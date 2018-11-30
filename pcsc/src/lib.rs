//! Communicate with smart cards using the PC/SC API.
//!
//! PC/SC (Personal Computer/Smart Card) is a standard API for
//! communicating with smart cards -- enumerating card readers, connecting
//! to smart cards, sending them commands, etc. See [Wikipedia][1] and
//! [PC/SC Workgroup][2] for more information.
//!
//! [1]: https://en.wikipedia.org/wiki/PC/SC
//! [2]: https://www.pcscworkgroup.com/
//!
//! This library is a safe and ergonomic FFI wrapper around the following
//! PC/SC implementations:
//!
//! - On Windows, the built-in `WinSCard.dll` library and "Smart Card"
//!   service. See [MSDN][3] for documentation of the implemented API.
//!
//! - On macOS, the built-in PCSC framework.
//!
//! - On Linux, BSDs and (hopefully) other systems, the PCSC lite library
//!   and pcscd daemon. See [pcsclite][4] for documentation of the
//!   implemented API.
//!
//! [3]: https://msdn.microsoft.com/EN-US/library/aa374731.aspx#smart_card_functions
//! [4]: https://pcsclite.alioth.debian.org/pcsclite.html
//!
//! ## Communicating with a smart card
//!
//! To communicate with a smart card, you send it APDU (Application
//! Protocol Data Unit) commands, and receive APDU responses.
//!
//! The format of these commands is described in the [ISO 7816 Part 4][5]
//! standard. The commands themselves vary based on the application on the
//! card.
//!
//! [5]: http://www.cardwerk.com/smartcards/smartcard_standard_ISO7816-4.aspx
//!
//! ## Note on portability
//!
//! The various implementations are not fully consistent with each other,
//! and some may also miss various features or exhibit various bugs.
//! Hence, you cannot assume that code which works on one platform will
//! behave the same in all other platforms - unfortunately, some
//! adjustments might be needed to reach a common base. See [pcsclite][4]
//! for a list of documented differences, and [Ludovic Rousseau's blog][6]
//! archive for many more details.
//!
//! [6]: https://ludovicrousseau.blogspot.com/
//!
//! Not all PC/SC functionality is covered yet; if you are missing
//! something, please open an issue.
//!
//! ## Note on strings
//!
//! The library uses C strings (`&CStr`) for all strings (e.g. card reader
//! names), to avoid any allocation and conversion overhead.
//!
//! In pcsclite and macOS, all strings are guaranteed to be UTF-8 encoded.
//!
//! In Windows, the API provides two variants of all functions dealing
//! with strings - ASCII and Unicode (in this case, meaning 16-bits wide
//! strings). For ease of implementation, this library wraps the ASCII
//! variants only. (If you require Unicode names in Windows, please open
//! an issue.)
//!
//! Since ASCII is a subset of UTF-8, you can thus safely use UTF-8
//! conversion functions such as `to_str()` to obtain an `&str`/`String`
//! from this library -- but don't do this if you don't need to ☺
//!
//! ## Note on thread safety and blocking operations
//!
//! A library context can be safely moved to another thread or used from
//! multiple threads.
//!
//! Operations on a given context are not performed concurrently. If one
//! thread performs a blocking operation on a context, such as
//! `get_status_change()`, then another operation on the context will
//! block until the ongoing operation finishes.
//!
//! An ongoing blocking operation on a context can be canceled from another
//! thread by calling the `cancel` function on the context.
//!
//! If you want to perform concurrent operations, for example, monitor
//! smart card reader changes in one thread, and send commands to cards in
//! another, create a different context for each thread.
//!
//! Note however, that if one context has an exclusive transaction with a
//! card, any operation on the same underlying card from not within the
//! transaction will block even across contexts.
//!
//! See [MSDN][7] for more details.
//!
//! [7]: https://msdn.microsoft.com/en-us/library/ms953432.aspx#smartcardcspcook_topic2

#[macro_use]
extern crate bitflags;
extern crate pcsc_sys as ffi;

use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::{forget, transmute};
use std::ops::Deref;
use std::os::raw::c_char;
use std::ptr::{null, null_mut};

use ffi::{DWORD, LONG};

// We use these instead of std::mem::uninitialized -- variables which are
// set to this are always overridden and the dummy values are never exposed.
const DUMMY_LONG: LONG = -1;
const DUMMY_DWORD: DWORD = 0xdead_beef;

// Note on potentially problematic casts (clippy lints `cast-sign-loss`,
// `cast-possible-truncation`): from my analysis they are all OK, for
// both 32bit and 64bit DWORD/LONG. But it is sketchy.

bitflags! {
    /// A mask of the state a card reader.
    pub struct State: DWORD {
        const UNAWARE = ffi::SCARD_STATE_UNAWARE;
        const IGNORE = ffi::SCARD_STATE_IGNORE;
        const CHANGED = ffi::SCARD_STATE_CHANGED;
        const UNKNOWN = ffi::SCARD_STATE_UNKNOWN;
        const UNAVAILABLE = ffi::SCARD_STATE_UNAVAILABLE;
        const EMPTY = ffi::SCARD_STATE_EMPTY;
        const PRESENT = ffi::SCARD_STATE_PRESENT;
        const ATRMATCH = ffi::SCARD_STATE_ATRMATCH;
        const EXCLUSIVE = ffi::SCARD_STATE_EXCLUSIVE;
        const INUSE = ffi::SCARD_STATE_INUSE;
        const MUTE = ffi::SCARD_STATE_MUTE;
        const UNPOWERED = ffi::SCARD_STATE_UNPOWERED;
    }
}

bitflags! {
    /// A mask of the status of a card in a card reader.
    pub struct Status: DWORD {
        const UNKNOWN = ffi::SCARD_UNKNOWN;
        const ABSENT = ffi::SCARD_ABSENT;
        const PRESENT = ffi::SCARD_PRESENT;
        const SWALLOWED = ffi::SCARD_SWALLOWED;
        const POWERED = ffi::SCARD_POWERED;
        const NEGOTIABLE = ffi::SCARD_NEGOTIABLE;
        const SPECIFIC = ffi::SCARD_SPECIFIC;
    }
}

/// How a reader connection is shared.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShareMode {
    Exclusive = ffi::SCARD_SHARE_EXCLUSIVE as u32,
    Shared = ffi::SCARD_SHARE_SHARED as u32,
    Direct = ffi::SCARD_SHARE_DIRECT as u32,
}

/// A smart card communication protocol.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Protocol {
    T0 = ffi::SCARD_PROTOCOL_T0 as u32,
    T1 = ffi::SCARD_PROTOCOL_T1 as u32,
    RAW = ffi::SCARD_PROTOCOL_RAW as u32,
}

impl Protocol {
    fn from_raw(raw: DWORD) -> Protocol {
        match raw {
            ffi::SCARD_PROTOCOL_T0 => Protocol::T0,
            ffi::SCARD_PROTOCOL_T1 => Protocol::T1,
            ffi::SCARD_PROTOCOL_RAW => Protocol::RAW,
            // This should not be possible, since we only allow to select
            // from Protocol's variants. Hence, we can panic.
            _ => panic!("impossible protocol: {:#x}", raw),
        }
    }
}

bitflags! {
    /// A mask of possible communication protocols.
    pub struct Protocols: DWORD {
        const UNDEFINED = ffi::SCARD_PROTOCOL_UNDEFINED;
        const T0 = ffi::SCARD_PROTOCOL_T0;
        const T1 = ffi::SCARD_PROTOCOL_T1;
        const RAW = ffi::SCARD_PROTOCOL_RAW;
        const ANY = ffi::SCARD_PROTOCOL_ANY;
    }
}

/// Disposition method when disconnecting from a card reader.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Disposition {
    LeaveCard = ffi::SCARD_LEAVE_CARD as u32,
    ResetCard = ffi::SCARD_RESET_CARD as u32,
    UnpowerCard = ffi::SCARD_UNPOWER_CARD as u32,
    EjectCard = ffi::SCARD_EJECT_CARD as u32,
}

/// Possible library errors.
///
/// See [pcsclite][1], [MSDN][2].
///
/// [1]: https://pcsclite.alioth.debian.org/api/group__ErrorCodes.html
/// [2]: https://msdn.microsoft.com/en-us/library/windows/desktop/aa374738(v=vs.85).aspx#smart_card_return_values
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    // <contiguous block 1>
    InternalError = ffi::SCARD_F_INTERNAL_ERROR as u32,
    Cancelled = ffi::SCARD_E_CANCELLED as u32,
    InvalidHandle = ffi::SCARD_E_INVALID_HANDLE as u32,
    InvalidParameter = ffi::SCARD_E_INVALID_PARAMETER as u32,
    InvalidTarget = ffi::SCARD_E_INVALID_TARGET as u32,
    NoMemory = ffi::SCARD_E_NO_MEMORY as u32,
    WaitedTooLong = ffi::SCARD_F_WAITED_TOO_LONG as u32,
    InsufficientBuffer = ffi::SCARD_E_INSUFFICIENT_BUFFER as u32,
    UnknownReader = ffi::SCARD_E_UNKNOWN_READER as u32,
    Timeout = ffi::SCARD_E_TIMEOUT as u32,
    SharingViolation = ffi::SCARD_E_SHARING_VIOLATION as u32,
    NoSmartcard = ffi::SCARD_E_NO_SMARTCARD as u32,
    UnknownCard = ffi::SCARD_E_UNKNOWN_CARD as u32,
    CantDispose = ffi::SCARD_E_CANT_DISPOSE as u32,
    ProtoMismatch = ffi::SCARD_E_PROTO_MISMATCH as u32,
    NotReady = ffi::SCARD_E_NOT_READY as u32,
    InvalidValue = ffi::SCARD_E_INVALID_VALUE as u32,
    SystemCancelled = ffi::SCARD_E_SYSTEM_CANCELLED as u32,
    CommError = ffi::SCARD_F_COMM_ERROR as u32,
    UnknownError = ffi::SCARD_F_UNKNOWN_ERROR as u32,
    InvalidAtr = ffi::SCARD_E_INVALID_ATR as u32,
    NotTransacted = ffi::SCARD_E_NOT_TRANSACTED as u32,
    ReaderUnavailable = ffi::SCARD_E_READER_UNAVAILABLE as u32,
    Shutdown = ffi::SCARD_P_SHUTDOWN as u32,
    PciTooSmall = ffi::SCARD_E_PCI_TOO_SMALL as u32,
    ReaderUnsupported = ffi::SCARD_E_READER_UNSUPPORTED as u32,
    DuplicateReader = ffi::SCARD_E_DUPLICATE_READER as u32,
    CardUnsupported = ffi::SCARD_E_CARD_UNSUPPORTED as u32,
    NoService = ffi::SCARD_E_NO_SERVICE as u32,
    ServiceStopped = ffi::SCARD_E_SERVICE_STOPPED as u32,
    #[cfg(target_os = "windows")]
    Unexpected = ffi::SCARD_E_UNEXPECTED as u32,
    IccInstallation = ffi::SCARD_E_ICC_INSTALLATION as u32,
    IccCreateorder = ffi::SCARD_E_ICC_CREATEORDER as u32,
    UnsupportedFeature = ffi::SCARD_E_UNSUPPORTED_FEATURE as u32,
    DirNotFound = ffi::SCARD_E_DIR_NOT_FOUND as u32,
    FileNotFound = ffi::SCARD_E_FILE_NOT_FOUND as u32,
    NoDir = ffi::SCARD_E_NO_DIR as u32,
    NoFile = ffi::SCARD_E_NO_FILE as u32,
    NoAccess = ffi::SCARD_E_NO_ACCESS as u32,
    WriteTooMany = ffi::SCARD_E_WRITE_TOO_MANY as u32,
    BadSeek = ffi::SCARD_E_BAD_SEEK as u32,
    InvalidChv = ffi::SCARD_E_INVALID_CHV as u32,
    UnknownResMng = ffi::SCARD_E_UNKNOWN_RES_MNG as u32,
    NoSuchCertificate = ffi::SCARD_E_NO_SUCH_CERTIFICATE as u32,
    CertificateUnavailable = ffi::SCARD_E_CERTIFICATE_UNAVAILABLE as u32,
    NoReadersAvailable = ffi::SCARD_E_NO_READERS_AVAILABLE as u32,
    CommDataLost = ffi::SCARD_E_COMM_DATA_LOST as u32,
    NoKeyContainer = ffi::SCARD_E_NO_KEY_CONTAINER as u32,
    ServerTooBusy = ffi::SCARD_E_SERVER_TOO_BUSY as u32,
    // </contiguous block 1>

    // <contiguous block 2>
    UnsupportedCard = ffi::SCARD_W_UNSUPPORTED_CARD as u32,
    UnresponsiveCard = ffi::SCARD_W_UNRESPONSIVE_CARD as u32,
    UnpoweredCard = ffi::SCARD_W_UNPOWERED_CARD as u32,
    ResetCard = ffi::SCARD_W_RESET_CARD as u32,
    RemovedCard = ffi::SCARD_W_REMOVED_CARD as u32,

    SecurityViolation = ffi::SCARD_W_SECURITY_VIOLATION as u32,
    WrongChv = ffi::SCARD_W_WRONG_CHV as u32,
    ChvBlocked = ffi::SCARD_W_CHV_BLOCKED as u32,
    Eof = ffi::SCARD_W_EOF as u32,
    CancelledByUser = ffi::SCARD_W_CANCELLED_BY_USER as u32,
    CardNotAuthenticated = ffi::SCARD_W_CARD_NOT_AUTHENTICATED as u32,

    CacheItemNotFound = ffi::SCARD_W_CACHE_ITEM_NOT_FOUND as u32,
    CacheItemStale = ffi::SCARD_W_CACHE_ITEM_STALE as u32,
    CacheItemTooBig = ffi::SCARD_W_CACHE_ITEM_TOO_BIG as u32,
    // </contiguous block 2>
}

impl Error {
    fn from_raw(raw: LONG) -> Error {
        unsafe {
            // The ranges here are the "blocks" above.
            if ffi::SCARD_F_INTERNAL_ERROR <= raw && raw <= ffi::SCARD_E_SERVER_TOO_BUSY
                || ffi::SCARD_W_UNSUPPORTED_CARD <= raw && raw <= ffi::SCARD_W_CACHE_ITEM_TOO_BIG
            {
                transmute::<u32, Error>(raw as u32)
            } else {
                debug_assert!(false, format!("unknown PCSC error code: {:#x}", raw));
                // We mask unknown error codes here; this is not very nice,
                // but seems better than panicking.
                Error::UnknownError
            }
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        // The descriptions are from MSDN.
        match *self {
            Error::InternalError => "An internal consistency check failed",
            Error::Cancelled => "The action was cancelled by an SCardCancel request",
            Error::InvalidHandle => "The supplied handle was invalid",
            Error::InvalidParameter => "One or more of the supplied parameters could not be properly interpreted",
            Error::InvalidTarget => "Registry startup information is missing or invalid",
            Error::NoMemory => "Not enough memory available to complete this command",
            Error::WaitedTooLong => "An internal consistency timer has expired",
            Error::InsufficientBuffer => "The data buffer to receive returned data is too small for the returned data",
            Error::UnknownReader => "The specified reader name is not recognized",
            Error::Timeout => "The user-specified timeout value has expired",
            Error::SharingViolation => "The smart card cannot be accessed because of other connections outstanding",
            Error::NoSmartcard => "The operation requires a Smart Card, but no Smart Card is currently in the device",
            Error::UnknownCard => "The specified smart card name is not recognized",
            Error::CantDispose => "The system could not dispose of the media in the requested manner",
            Error::ProtoMismatch => "The requested protocols are incompatible with the protocol currently in use with the smart card",
            Error::NotReady => "The reader or smart card is not ready to accept commands",
            Error::InvalidValue => "One or more of the supplied parameters values could not be properly interpreted",
            Error::SystemCancelled => "The action was cancelled by the system, presumably to log off or shut down",
            Error::CommError => "An internal communications error has been detected",
            Error::UnknownError => "An internal error has been detected, but the source is unknown",
            Error::InvalidAtr => "An ATR obtained from the registry is not a valid ATR string",
            Error::NotTransacted => "An attempt was made to end a non-existent transaction",
            Error::ReaderUnavailable => "The specified reader is not currently available for use",
            Error::Shutdown => "The operation has been aborted to allow the server application to exit",
            Error::PciTooSmall => "The PCI Receive buffer was too small",
            Error::ReaderUnsupported => "The reader driver does not meet minimal requirements for support",
            Error::DuplicateReader => "The reader driver did not produce a unique reader name",
            Error::CardUnsupported => "The smart card does not meet minimal requirements for support",
            Error::NoService => "The Smart card resource manager is not running",
            Error::ServiceStopped => "The Smart card resource manager has shut down",
            #[cfg(target_os = "windows")]
            Error::Unexpected => "An unexpected card error has occurred",
            Error::UnsupportedFeature => "This smart card does not support the requested feature",
            Error::IccInstallation => "No primary provider can be found for the smart card",
            Error::IccCreateorder => "The requested order of object creation is not supported",
            Error::DirNotFound => "The identified directory does not exist in the smart card",
            Error::FileNotFound => "The identified file does not exist in the smart card",
            Error::NoDir => "The supplied path does not represent a smart card directory",
            Error::NoFile => "The supplied path does not represent a smart card file",
            Error::NoAccess => "Access is denied to this file",
            Error::WriteTooMany => "The smart card does not have enough memory to store the information",
            Error::BadSeek => "There was an error trying to set the smart card file object pointer",
            Error::InvalidChv => "The supplied PIN is incorrect",
            Error::UnknownResMng => "An unrecognized error code was returned from a layered component",
            Error::NoSuchCertificate => "The requested certificate does not exist",
            Error::CertificateUnavailable => "The requested certificate could not be obtained",
            Error::NoReadersAvailable => "Cannot find a smart card reader",
            Error::CommDataLost => "A communications error with the smart card has been detected. Retry the operation",
            Error::NoKeyContainer => "The requested key container does not exist on the smart card",
            Error::ServerTooBusy => "The smart card resource manager is too busy to complete this operation",
            Error::UnsupportedCard => "The reader cannot communicate with the card, due to ATR string configuration conflicts",
            Error::UnresponsiveCard => "The smart card is not responding to a reset",
            Error::UnpoweredCard => "Power has been removed from the smart card, so that further communication is not possible",
            Error::ResetCard => "The smart card has been reset, so any shared state information is invalid",
            Error::RemovedCard => "The smart card has been removed, so further communication is not possible",
            Error::SecurityViolation => "Access was denied because of a security violation",
            Error::WrongChv => "The card cannot be accessed because the wrong PIN was presented",
            Error::ChvBlocked => "The card cannot be accessed because the maximum number of PIN entry attempts has been reached",
            Error::Eof => "The end of the smart card file has been reached",
            Error::CancelledByUser => r#"The user pressed "Cancel" on a Smart Card Selection Dialog"#,
            Error::CardNotAuthenticated => "No PIN was presented to the smart card",
            Error::CacheItemNotFound => "The requested item could not be found in the cache",
            Error::CacheItemStale => "The requested cache item is too old and was deleted from the cache",
            Error::CacheItemTooBig => "The new cache item exceeds the maximum per-item size defined for the cache",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(std::error::Error::description(self))
    }
}

macro_rules! try_pcsc {
    ($e:expr) => (match $e {
        ffi::SCARD_S_SUCCESS => (),
        err => return Err(Error::from_raw(err)),
    });
}

/// Scope of a context.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    User = ffi::SCARD_SCOPE_USER as u32,
    Terminal = ffi::SCARD_SCOPE_TERMINAL as u32,
    System = ffi::SCARD_SCOPE_SYSTEM as u32,
    Global = ffi::SCARD_SCOPE_GLOBAL as u32,
}

/// A class of Attributes.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeClass {
    VendorInfo = ffi::SCARD_CLASS_VENDOR_INFO as u32,
    Communications = ffi::SCARD_CLASS_COMMUNICATIONS as u32,
    Protocol = ffi::SCARD_CLASS_PROTOCOL as u32,
    PowerMgmt = ffi::SCARD_CLASS_POWER_MGMT as u32,
    Security = ffi::SCARD_CLASS_SECURITY as u32,
    Mechanical = ffi::SCARD_CLASS_MECHANICAL as u32,
    VendorDefined = ffi::SCARD_CLASS_VENDOR_DEFINED as u32,
    IfdProtocol = ffi::SCARD_CLASS_IFD_PROTOCOL as u32,
    IccState = ffi::SCARD_CLASS_ICC_STATE as u32,
    System = ffi::SCARD_CLASS_SYSTEM as u32,
}

/// Card reader attribute types.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Attribute {
    VendorName = ffi::SCARD_ATTR_VENDOR_NAME as u32,
    VendorIfdType = ffi::SCARD_ATTR_VENDOR_IFD_TYPE as u32,
    VendorIfdVersion = ffi::SCARD_ATTR_VENDOR_IFD_VERSION as u32,
    VendorIfdSerialNo = ffi::SCARD_ATTR_VENDOR_IFD_SERIAL_NO as u32,
    ChannelId = ffi::SCARD_ATTR_CHANNEL_ID as u32,
    AsyncProtocolTypes = ffi::SCARD_ATTR_ASYNC_PROTOCOL_TYPES as u32,
    DefaultClk = ffi::SCARD_ATTR_DEFAULT_CLK as u32,
    MaxClk = ffi::SCARD_ATTR_MAX_CLK as u32,
    DefaultDataRate = ffi::SCARD_ATTR_DEFAULT_DATA_RATE as u32,
    MaxDataRate = ffi::SCARD_ATTR_MAX_DATA_RATE as u32,
    MaxIfsd = ffi::SCARD_ATTR_MAX_IFSD as u32,
    SyncProtocolTypes = ffi::SCARD_ATTR_SYNC_PROTOCOL_TYPES as u32,
    PowerMgmtSupport = ffi::SCARD_ATTR_POWER_MGMT_SUPPORT as u32,
    UserToCardAuthDevice = ffi::SCARD_ATTR_USER_TO_CARD_AUTH_DEVICE as u32,
    UserAuthInputDevice = ffi::SCARD_ATTR_USER_AUTH_INPUT_DEVICE as u32,
    Characteristics = ffi::SCARD_ATTR_CHARACTERISTICS as u32,

    CurrentProtocolType = ffi::SCARD_ATTR_CURRENT_PROTOCOL_TYPE as u32,
    CurrentClk = ffi::SCARD_ATTR_CURRENT_CLK as u32,
    CurrentF = ffi::SCARD_ATTR_CURRENT_F as u32,
    CurrentD = ffi::SCARD_ATTR_CURRENT_D as u32,
    CurrentN = ffi::SCARD_ATTR_CURRENT_N as u32,
    CurrentW = ffi::SCARD_ATTR_CURRENT_W as u32,
    CurrentIfsc = ffi::SCARD_ATTR_CURRENT_IFSC as u32,
    CurrentIfsd = ffi::SCARD_ATTR_CURRENT_IFSD as u32,
    CurrentBwt = ffi::SCARD_ATTR_CURRENT_BWT as u32,
    CurrentCwt = ffi::SCARD_ATTR_CURRENT_CWT as u32,
    CurrentEbcEncoding = ffi::SCARD_ATTR_CURRENT_EBC_ENCODING as u32,
    ExtendedBwt = ffi::SCARD_ATTR_EXTENDED_BWT as u32,

    IccPresence = ffi::SCARD_ATTR_ICC_PRESENCE as u32,
    IccInterfaceStatus = ffi::SCARD_ATTR_ICC_INTERFACE_STATUS as u32,
    CurrentIoState = ffi::SCARD_ATTR_CURRENT_IO_STATE as u32,
    AtrString = ffi::SCARD_ATTR_ATR_STRING as u32,
    IccTypePerAtr = ffi::SCARD_ATTR_ICC_TYPE_PER_ATR as u32,

    EscReset = ffi::SCARD_ATTR_ESC_RESET as u32,
    EscCancel = ffi::SCARD_ATTR_ESC_CANCEL as u32,
    EscAuthrequest = ffi::SCARD_ATTR_ESC_AUTHREQUEST as u32,
    Maxinput = ffi::SCARD_ATTR_MAXINPUT as u32,

    DeviceUnit = ffi::SCARD_ATTR_DEVICE_UNIT as u32,
    DeviceInUse = ffi::SCARD_ATTR_DEVICE_IN_USE as u32,
    DeviceFriendlyName = ffi::SCARD_ATTR_DEVICE_FRIENDLY_NAME as u32,
    DeviceSystemName = ffi::SCARD_ATTR_DEVICE_SYSTEM_NAME as u32,
    SupressT1IfsRequest = ffi::SCARD_ATTR_SUPRESS_T1_IFS_REQUEST as u32,
}

/// Maximum amount of bytes in an ATR.
pub const MAX_ATR_SIZE: usize = ffi::MAX_ATR_SIZE;
/// Maximum amount of bytes in a short APDU command or response.
pub const MAX_BUFFER_SIZE: usize = ffi::MAX_BUFFER_SIZE;
/// Maximum amount of bytes in an extended APDU command or response.
pub const MAX_BUFFER_SIZE_EXTENDED: usize = ffi::MAX_BUFFER_SIZE_EXTENDED;

/// A special reader name for detecting card reader insertions and removals.
///
/// # Note
///
/// This function is a wrapper around a constant, and is intended to be
/// used as such.
#[allow(non_snake_case)]
// We can't have a const &CStr yet, so we simulate it with a function.
pub fn PNP_NOTIFICATION() -> &'static CStr {
    unsafe { CStr::from_bytes_with_nul_unchecked(b"\\\\?PnP?\\Notification\0") }
}

/// A structure for tracking the current state of card readers and cards.
///
/// This structure wraps `SCARD_READERSTATE` ([pcsclite][1], [MSDN][2]).
///
/// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga33247d5d1257d59e55647c3bb717db24
/// [2]: https://msdn.microsoft.com/en-us/library/aa379808.aspx
#[repr(C)]
pub struct ReaderState {
    // Note: must be directly transmutable to SCARD_READERSTATE.
    inner: ffi::SCARD_READERSTATE,
}

// For some reason, linking in windows fails if we put these directly
// in statics. This is why we have this function instead of the
// SCARD_PCI_* defines from the C API.
fn get_protocol_pci(protocol: Protocol) -> &'static ffi::SCARD_IO_REQUEST {
    unsafe {
        match protocol {
            Protocol::T0 => &ffi::g_rgSCardT0Pci,
            Protocol::T1 => &ffi::g_rgSCardT1Pci,
            Protocol::RAW => &ffi::g_rgSCardRawPci,
        }
    }
}

/// Library context to the PCSC service.
///
/// This structure wraps `SCARDCONTEXT`.
pub struct Context {
    handle: ffi::SCARDCONTEXT,
}

/// A connection to a smart card.
///
/// This structure wraps `SCARDHANDLE`.
pub struct Card<'ctx> {
    _context: PhantomData<&'ctx Context>,
    handle: ffi::SCARDHANDLE,
    active_protocol: Protocol,
}

/// An exclusive transaction with a card.
///
/// A transaction ensures uninterrupted access to the card for its
/// duration. All other operations performed on the same underlying
/// card (even from other processes) will block until the transaction
/// is finished.
// By taking a mut reference to the card we statically enforce that:
// - There can only be one active transaction at a time.
// - All operations on the card must be performed through the transaction
//   for the duration of the transaction's lifetime.
pub struct Transaction<'tx, 'card: 'tx> {
    card: &'tx mut Card<'card>,
}

/// An iterator over card reader names.
///
/// The iterator does not perform any copying or allocations; this is left
/// to the caller's discretion. It is therefore tied to the underlying
/// buffer.
#[derive(Clone)]
pub struct ReaderNames<'buf> {
    buf: &'buf [u8],
    pos: usize,
}

impl<'buf> Iterator for ReaderNames<'buf> {
    type Item = &'buf CStr;

    fn next(&mut self) -> Option<&'buf CStr> {
        match self.buf[self.pos..].iter().position(|&c| c == 0) {
            None | Some(0) => None,
            Some(len) => {
                let old_pos = self.pos;
                self.pos += len + 1;
                // The panic can't happen, but we avoid unsafe.
                Some(CStr::from_bytes_with_nul(&self.buf[old_pos..self.pos]).unwrap())
            }
        }
    }
}

impl Context {
    /// Establish a new context.
    ///
    /// This function wraps `SCardEstablishContext` ([pcsclite][1],
    /// [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gaa1b8970169fd4883a6dc4a8f43f19b67
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379479.aspx
    pub fn establish(
        scope: Scope,
    ) -> Result<Context, Error> {
        unsafe {
            let mut ctx: ffi::SCARDCONTEXT = DUMMY_LONG as ffi::SCARDCONTEXT;

            try_pcsc!(ffi::SCardEstablishContext(
                scope as DWORD,
                null(),
                null(),
                &mut ctx,
            ));

            Ok(Context {
                handle: ctx,
            })
        }
    }

    /// Release the context.
    ///
    /// In case of error, ownership of the context is returned to the
    /// caller.
    ///
    /// This function wraps `SCardReleaseContext` ([pcsclite][1],
    /// [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga6aabcba7744c5c9419fdd6404f73a934
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379798.aspx
    ///
    /// ## Note
    ///
    /// `Context` implements `Drop` which automatically releases the
    /// context; you only need to call this function if you want to handle
    /// errors.
    pub fn release(
        self
    ) -> Result<(), (Context, Error)> {
        unsafe {
            let err = ffi::SCardReleaseContext(
                self.handle,
            );
            if err != ffi::SCARD_S_SUCCESS {
                return Err((self, Error::from_raw(err)));
            }

            // Skip the drop, we did it "manually".
            forget(self);

            Ok(())
        }
    }

    /// Check whether the Context is still valid.
    ///
    /// This function wraps `SCardIsValidContext` ([pcsclite][1],
    /// [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga722eb66bcc44d391f700ff9065cc080b
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379788.aspx
    pub fn is_valid(
        &self
    ) -> Result<(), Error> {
        unsafe {
            try_pcsc!(ffi::SCardIsValidContext(
                self.handle,
            ));

            Ok(())
        }
    }

    /// Cancel any ongoing blocking operation in the Context.
    ///
    /// See the `cancel.rs` example program.
    ///
    /// This function wraps `SCardCancel` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gaacbbc0c6d6c0cbbeb4f4debf6fbeeee6
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379470.aspx
    pub fn cancel(
        &self,
    ) -> Result<(), Error> {
        unsafe {
            try_pcsc!(ffi::SCardCancel(
                self.handle,
            ));

            Ok(())
        }
    }

    /// List all connected card readers.
    ///
    /// `buffer` is a buffer that should be large enough to hold all of
    /// the connected reader names. The function `list_readers_len` can be
    /// used to find the exact required length.
    ///
    /// Returns an iterator over the reader names. The iterator yields
    /// values directly from `buffer`.
    ///
    /// If the buffer is not large enough to hold all of the names,
    /// `Error::InsufficientBuffer` is returned.
    ///
    /// This function wraps `SCardListReaders` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga93b07815789b3cf2629d439ecf20f0d9
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379793.aspx
    pub fn list_readers<'buf>(
        &self,
        buffer: &'buf mut [u8],
    ) -> Result<ReaderNames<'buf>, Error> {
        unsafe {
            let mut buflen = buffer.len() as DWORD;

            let err = ffi::SCardListReaders(
                self.handle,
                null(),
                buffer.as_mut_ptr() as *mut c_char,
                &mut buflen,
            );
            if err == Error::NoReadersAvailable as LONG {
                return Ok(ReaderNames {
                    buf: b"\0",
                    pos: 0,
                });
            }
            if err != ffi::SCARD_S_SUCCESS {
                return Err(Error::from_raw(err));
            }

            Ok(ReaderNames {
                buf: &buffer[..buflen as usize],
                pos: 0,
            })
        }
    }

    /// Get the needed length of a buffer to be passed to `list_readers`.
    ///
    /// This function wraps `SCardListReaders` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga93b07815789b3cf2629d439ecf20f0d9
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379793.aspx
    pub fn list_readers_len(
        &self,
    ) -> Result<usize, Error> {
        unsafe {
            let mut buflen = DUMMY_DWORD;

            let err = ffi::SCardListReaders(
                self.handle,
                null(),
                null_mut(),
                &mut buflen,
            );
            if err == Error::NoReadersAvailable as LONG {
                return Ok(0);
            }
            if err != ffi::SCARD_S_SUCCESS {
                return Err(Error::from_raw(err));
            }

            Ok(buflen as usize)
        }
    }

    /// Connect to a card which is present in a reader.
    ///
    /// See the `connect.rs` example program.
    ///
    /// This function wraps `SCardConnect` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga4e515829752e0a8dbc4d630696a8d6a5
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379473.aspx
    pub fn connect(
        &self,
        reader: &CStr,
        share_mode: ShareMode,
        preferred_protocols: Protocols,
    ) -> Result<Card, Error> {
        unsafe {
            let mut handle: ffi::SCARDHANDLE = DUMMY_LONG as ffi::SCARDHANDLE;
            let mut raw_active_protocol: DWORD = DUMMY_DWORD;

            try_pcsc!(ffi::SCardConnect(
                self.handle,
                reader.as_ptr(),
                share_mode as DWORD,
                preferred_protocols.bits(),
                &mut handle,
                &mut raw_active_protocol,
            ));

            let active_protocol = Protocol::from_raw(raw_active_protocol);

            Ok(Card {
                _context: PhantomData,
                handle,
                active_protocol,
            })
        }
    }

    /// Wait for card and card reader state changes.
    ///
    /// The function blocks until the state of one of the readers changes
    /// from corresponding passed-in `ReaderState`. The `ReaderState`s are
    /// updated to report the new state.
    ///
    /// A special reader name, `\\?PnP?\Notification`, can be used to
    /// detect card reader insertions and removals, as opposed to state
    /// changes of a specific reader. Use `PNP_NOTIFICATION()` to easily
    /// obtain a static reference to this name.
    ///
    /// See the `monitor.rs` example program.
    ///
    /// This function wraps `SCardGetStatusChange` ([pcsclite][1],
    /// [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga33247d5d1257d59e55647c3bb717db24
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379773.aspx
    pub fn get_status_change<D>(
        &self,
        timeout: D,
        readers: &mut [ReaderState],
    ) -> Result<(), Error>
        where D: Into<Option<std::time::Duration>> {
        let timeout_ms = match timeout.into() {
            Some(duration) => {
                let timeout_ms_u64 = duration.as_secs()
                    .saturating_mul(1000)
                    .saturating_add(u64::from(duration.subsec_nanos()) / 1_000_000);
                std::cmp::min(ffi::INFINITE, timeout_ms_u64 as DWORD)
            },
            None => ffi::INFINITE
        };

        unsafe {
            try_pcsc!(ffi::SCardGetStatusChange(
                self.handle,
                timeout_ms,
                readers.as_mut_ptr() as *mut ffi::SCARD_READERSTATE,
                readers.len() as DWORD,
            ));

            Ok(())
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            // Error is ignored here; to do proper error handling,
            // release() should be called manually.
            let _err = ffi::SCardReleaseContext(
                self.handle,
            );
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl ReaderState {
    /// Create a ReaderState for a card reader with a given presumed
    /// state.
    // TODO: Support ATR fields.
    pub fn new<T: Into<CString>>(
        name: T,
        current_state: State,
    ) -> ReaderState {
        ReaderState {
            inner: ffi::SCARD_READERSTATE {
                szReader: name.into().into_raw(),
                // This seems useless to expose.
                pvUserData: null_mut(),
                dwCurrentState: current_state.bits(),
                dwEventState: State::UNAWARE.bits(),
                cbAtr: 0,
                rgbAtr: [0; ffi::ATR_BUFFER_SIZE],
            },
        }
    }

    /// The name of the card reader.
    pub fn name(&self) -> &CStr {
        // Lifetime elision assigns this the same lifetime as &self; this
        // is what we want, and is safe.
        unsafe { CStr::from_ptr(self.inner.szReader) }
    }

    /// The last reported state.
    pub fn event_state(&self) -> State {
        State::from_bits_truncate(self.inner.dwEventState)
    }

    /// The card event count.
    ///
    /// The count is incremented for each card insertion or removal in the
    /// reader. This can be used to detect a card removal/insertion
    /// between two calls to `Context::get_status_change()`.
    pub fn event_count(&self) -> u32 {
        ((self.inner.dwEventState & 0xFFFF_0000) >> 16) as u32
    }

    /// Sync the currently-known state to the last reported state.
    pub fn sync_current_state(&mut self) {
        // In windows it is important that the event count is included;
        // otherwise PNP_NOTIFICATION is always reported as changed:
        // https://stackoverflow.com/a/16467368
        self.inner.dwCurrentState = self.inner.dwEventState;
    }
}

impl Drop for ReaderState {
    fn drop(&mut self) {
        // Reclaim the name and drop it immediately.
        unsafe { CString::from_raw(self.inner.szReader as *mut c_char) };
    }
}

impl<'ctx> Card<'ctx> {
    /// Start a new exclusive transaction with the card.
    ///
    /// Operations on the card for the duration of the transaction
    /// can only be performed through the returned `Transaction`.
    ///
    /// This function wraps `SCardBeginTransaction` ([pcsclite][1],
    /// [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gaddb835dce01a0da1d6ca02d33ee7d861
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379469.aspx
    pub fn transaction<'tx>(
        &'tx mut self,
    ) -> Result<Transaction<'tx, 'ctx>, Error> {
        unsafe {
            try_pcsc!(ffi::SCardBeginTransaction(
                self.handle,
            ));

            Ok(Transaction {
                card: self,
            })
        }
    }

    /// Reconnect to the card.
    ///
    /// This function wraps `SCardReconnect` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gad5d4393ca8c470112ad9468c44ed8940
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379797.aspx
    pub fn reconnect(
        &mut self,
        share_mode: ShareMode,
        preferred_protocols: Protocols,
        initialization: Disposition,
    ) -> Result<(), Error> {
        unsafe {
            let mut raw_active_protocol: DWORD = DUMMY_DWORD;

            try_pcsc!(ffi::SCardReconnect(
                self.handle,
                share_mode as DWORD,
                preferred_protocols.bits(),
                initialization as DWORD,
                &mut raw_active_protocol,
            ));

            self.active_protocol = Protocol::from_raw(raw_active_protocol);

            Ok(())
        }
    }

    /// Disconnect from the card.
    ///
    /// In case of error, ownership of the card is returned to the caller.
    ///
    /// This function wraps `SCardDisconnect` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga4be198045c73ec0deb79e66c0ca1738a
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379475.aspx
    ///
    /// ## Note
    ///
    /// `Card` implements `Drop` which automatically disconnects the card
    /// using `Disposition::ResetCard`; you only need to call this
    /// function if you want to handle errors or use a different
    /// disposition method.
    pub fn disconnect(
        self,
        disposition: Disposition,
    ) -> Result<(), (Card<'ctx>, Error)> {
        unsafe {
            let err = ffi::SCardDisconnect(
                self.handle,
                disposition as DWORD,
            );
            if err != ffi::SCARD_S_SUCCESS {
                return Err((self, Error::from_raw(err)));
            }

            // Skip the drop, we did it "manually".
            forget(self);

            Ok(())
        }
    }

    /// Get current info on the card.
    ///
    /// This function wraps `SCardStatus` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gae49c3c894ad7ac12a5b896bde70d0382
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379803.aspx
    // TODO: Missing return values: reader names and ATR.
    pub fn status(
        &self,
    ) -> Result<(Status, Protocol), Error> {
        unsafe {
            let mut raw_status: DWORD = DUMMY_DWORD;
            let mut raw_protocol: DWORD = DUMMY_DWORD;

            try_pcsc!(ffi::SCardStatus(
                self.handle,
                null_mut(),
                null_mut(),
                &mut raw_status,
                &mut raw_protocol,
                null_mut(),
                null_mut(),
            ));

            let status = Status::from_bits_truncate(raw_status);
            let protocol = Protocol::from_raw(raw_protocol);

            Ok((status, protocol))
        }
    }

    /// Get an attribute of the card or card reader.
    ///
    /// `buffer` is a buffer that should be large enough for the attribute
    /// data. The function `get_attribute_len` can be used to find the
    /// exact required length.
    ///
    /// Returns a slice into `buffer` containing the attribute data.
    ///
    /// If the buffer is not large enough, `Error::InsufficientBuffer` is
    /// returned.
    ///
    /// This function wraps `SCardGetAttrib` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gaacfec51917255b7a25b94c5104961602
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379559.aspx
    pub fn get_attribute<'buf>(
        &self,
        attribute: Attribute,
        buffer: &'buf mut [u8],
    ) -> Result<&'buf [u8], Error> {
        unsafe {
            let mut attribute_len = buffer.len() as DWORD;

            try_pcsc!(ffi::SCardGetAttrib(
                self.handle,
                attribute as DWORD,
                buffer.as_mut_ptr(),
                &mut attribute_len,
            ));

            Ok(&buffer[0..attribute_len as usize])
        }
    }

    /// Get the needed length of a buffer to be passed to `get_attribute`.
    ///
    /// This function wraps `SCardGetAttrib` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gaacfec51917255b7a25b94c5104961602
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379559.aspx
    pub fn get_attribute_len(
        &self,
        attribute: Attribute,
    ) -> Result<usize, Error> {
        unsafe {
            let mut attribute_len = DUMMY_DWORD;

            try_pcsc!(ffi::SCardGetAttrib(
                self.handle,
                attribute as DWORD,
                null_mut(),
                &mut attribute_len,
            ));

            Ok(attribute_len as usize)
        }
    }

    /// Set an attribute of the card or card reader.
    ///
    /// This function wraps `SCardSetAttrib` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga060f0038a4ddfd5dd2b8fadf3c3a2e4f
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379801.aspx
    pub fn set_attribute(
        &self,
        attribute: Attribute,
        attribute_data: &[u8],
    ) -> Result<(), Error> {
        unsafe {
            try_pcsc!(ffi::SCardSetAttrib(
                self.handle,
                attribute as DWORD,
                attribute_data.as_ptr(),
                attribute_data.len() as DWORD,
            ));

            Ok(())
        }
    }

    /// Transmit an APDU command to the card.
    ///
    /// `receive_buffer` is a buffer that should be large enough to hold
    /// the APDU response.
    ///
    /// Returns a slice into `receive_buffer` containing the APDU
    /// response.
    ///
    /// If `receive_buffer` is not large enough to hold the APDU response,
    /// `Error::InsufficientBuffer` is returned.
    ///
    /// This function wraps `SCardTransmit` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#ga9a2d77242a271310269065e64633ab99
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379804.aspx
    pub fn transmit<'buf>(
        &self,
        send_buffer: &[u8],
        receive_buffer: &'buf mut [u8],
    ) -> Result<&'buf [u8], Error> {
        let send_pci = get_protocol_pci(self.active_protocol);
        let recv_pci = null_mut();
        let mut receive_len = receive_buffer.len() as DWORD;

        unsafe {
            try_pcsc!(ffi::SCardTransmit(
                self.handle,
                send_pci,
                send_buffer.as_ptr(),
                send_buffer.len() as DWORD,
                recv_pci,
                receive_buffer.as_mut_ptr(),
                &mut receive_len,
            ));

            Ok(&receive_buffer[0..receive_len as usize])
        }
    }

    /// Sends a command directly to the reader (driver).
    ///
    /// `receive_buffer` is a buffer that should be large enough to hold
    /// the response.
    ///
    /// Returns a slice into `receive_buffer` containing the response.
    ///
    /// If `receive_buffer` is not large enough to hold the response,
    /// `Error::InsufficientBuffer` is returned.
    ///
    /// This function wraps `SCardControl` ([pcsclite][1], [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gac3454d4657110fd7f753b2d3d8f4e32f
    /// [2]: https://msdn.microsoft.com/en-us/library/windows/desktop/aa379474(v=vs.85).aspx
    pub fn control<'buf>(
        &self,
        control_code: DWORD,
        send_buffer: &[u8],
        receive_buffer: &'buf mut [u8],
    ) -> Result<&'buf [u8], Error> {
        let mut receive_len: DWORD = DUMMY_DWORD;

        unsafe {
            try_pcsc!(ffi::SCardControl(
                self.handle,
                control_code,
                send_buffer.as_ptr(),
                send_buffer.len() as DWORD,
                receive_buffer.as_mut_ptr(),
                receive_buffer.len() as DWORD,
                &mut receive_len,
            ));

            Ok(&receive_buffer[0..receive_len as usize])
        }
    }
}

impl<'ctx> Drop for Card<'ctx> {
    fn drop(&mut self) {
        unsafe {
            // Error is ignored here; to do proper error handling,
            // disconnect() should be called manually.
            //
            // Disposition is hard-coded to ResetCard here; to use
            // another method, disconnect() should be called manually.
            let _err = ffi::SCardDisconnect(
                self.handle,
                Disposition::ResetCard as DWORD,
            );
        }
    }
}

unsafe impl<'ctx> Send for Card<'ctx> {}
unsafe impl<'ctx> Sync for Card<'ctx> {}

impl<'tx, 'card: 'tx> Transaction<'tx, 'card> {
    /// End the transaction.
    ///
    /// In case of error, ownership of the transaction is returned to the
    /// caller.
    ///
    /// This function wraps `SCardEndTransaction` ([pcsclite][1],
    /// [MSDN][2]).
    ///
    /// [1]: https://pcsclite.alioth.debian.org/api/group__API.html#gae8742473b404363e5c587f570d7e2f3b
    /// [2]: https://msdn.microsoft.com/en-us/library/aa379477.aspx
    ///
    /// ## Note
    ///
    /// `Transaction` implements `Drop` which automatically ends the
    /// transaction using `Disposition::LeaveCard`; you only need to call
    /// this function if you want to handle errors or use a different
    /// disposition method.
    pub fn end(
        self,
        disposition: Disposition,
    ) -> Result<(), (Transaction<'tx, 'card>, Error)> {
        unsafe {
            let err = ffi::SCardEndTransaction(
                self.card.handle,
                disposition as DWORD,
            );
            if err != 0 {
                return Err((self, Error::from_raw(err)));
            }

            // Skip the drop, we did it "manually".
            forget(self);

            Ok(())
        }
    }
}

impl<'tx, 'card: 'tx> Drop for Transaction<'tx, 'card> {
    fn drop(&mut self) {
        unsafe {
            // Error is ignored here; to do proper error handling,
            // end() should be called manually.
            //
            // Disposition is hard-coded to LeaveCard here; to use
            // another method, end() should be called manually.
            let _err = ffi::SCardEndTransaction(
                self.card.handle,
                Disposition::LeaveCard as DWORD,
            );
        }
    }
}

impl<'tx, 'card: 'tx> Deref for Transaction<'tx, 'card> {
    type Target = Card<'card>;

    fn deref(&self) -> &Card<'card> {
        self.card
    }
}
