//! Low level bindings to the PCSC C API.
//!
//! The following platforms are supported:
//!
//! - On Windows, the built-in `WinSCard.dll` library and "Smart Card"
//!   service. See [MSDN][1] for documentation of the implemented API.
//!
//! - On macOS, the built-in PCSC framework.
//!
//! - On Linux, BSDs and (hopefully) other systems, the PCSC lite library
//!   and pcscd daemon. See [pcsclite][2] for documentation of the
//!   implemented API.
//!
//!   pcsclite is detected at build time using pkg-config. See the
//!   [`pkg-config`][3] crate for more information.
//!
//!   If you do not want to use pkg-config, you may instead export the
//!   following environment variables when building the crate:
//!
//!   - `PCSC_LIB_DIR`: A directory in which to search for a dynamic
//!     library implementing the PCSC API.
//!   - `PCSC_LIB_NAME`: The name of the library. Defaults to `pcsclite`.
//!
//! [1]: https://msdn.microsoft.com/EN-US/library/aa374731.aspx#smart_card_functions
//! [2]: https://pcsclite.apdu.fr/
//! [3]: https://docs.rs/pkg-config/

#![allow(bad_style)]
// Needed for the errors, they are given in hex for some reason, but if
// LONG is i32 they are negative (which presumably was the intention).
#![allow(overflowing_literals)]

use std::os::raw::{c_char, c_void};
#[cfg(not(target_os = "macos"))]
use std::os::raw::{c_long, c_ulong};

#[cfg(not(target_os = "macos"))]
pub type DWORD = c_ulong;
#[cfg(not(target_os = "macos"))]
pub type LONG = c_long;
#[cfg(not(target_os = "macos"))]
pub type ULONG = c_ulong;

#[cfg(target_os = "macos")]
pub type DWORD = u32;
#[cfg(target_os = "macos")]
pub type LONG = i32;
#[cfg(target_os = "macos")]
pub type ULONG = u32;

#[cfg(target_os = "windows")]
pub type SCARDCONTEXT = usize;
#[cfg(target_os = "windows")]
pub type SCARDHANDLE = usize;

#[cfg(not(target_os = "windows"))]
pub type SCARDCONTEXT = LONG;
#[cfg(not(target_os = "windows"))]
pub type SCARDHANDLE = LONG;

pub const SCARD_S_SUCCESS: LONG = 0x0000_0000;
pub const SCARD_F_INTERNAL_ERROR: LONG = 0x8010_0001;
pub const SCARD_E_CANCELLED: LONG = 0x8010_0002;
pub const SCARD_E_INVALID_HANDLE: LONG = 0x8010_0003;
pub const SCARD_E_INVALID_PARAMETER: LONG = 0x8010_0004;
pub const SCARD_E_INVALID_TARGET: LONG = 0x8010_0005;
pub const SCARD_E_NO_MEMORY: LONG = 0x8010_0006;
pub const SCARD_F_WAITED_TOO_LONG: LONG = 0x8010_0007;
pub const SCARD_E_INSUFFICIENT_BUFFER: LONG = 0x8010_0008;
pub const SCARD_E_UNKNOWN_READER: LONG = 0x8010_0009;
pub const SCARD_E_TIMEOUT: LONG = 0x8010_000A;
pub const SCARD_E_SHARING_VIOLATION: LONG = 0x8010_000B;
pub const SCARD_E_NO_SMARTCARD: LONG = 0x8010_000C;
pub const SCARD_E_UNKNOWN_CARD: LONG = 0x8010_000D;
pub const SCARD_E_CANT_DISPOSE: LONG = 0x8010_000E;
pub const SCARD_E_PROTO_MISMATCH: LONG = 0x8010_000F;
pub const SCARD_E_NOT_READY: LONG = 0x8010_0010;
pub const SCARD_E_INVALID_VALUE: LONG = 0x8010_0011;
pub const SCARD_E_SYSTEM_CANCELLED: LONG = 0x8010_0012;
pub const SCARD_F_COMM_ERROR: LONG = 0x8010_0013;
pub const SCARD_F_UNKNOWN_ERROR: LONG = 0x8010_0014;
pub const SCARD_E_INVALID_ATR: LONG = 0x8010_0015;
pub const SCARD_E_NOT_TRANSACTED: LONG = 0x8010_0016;
pub const SCARD_E_READER_UNAVAILABLE: LONG = 0x8010_0017;
pub const SCARD_P_SHUTDOWN: LONG = 0x8010_0018;
pub const SCARD_E_PCI_TOO_SMALL: LONG = 0x8010_0019;
pub const SCARD_E_READER_UNSUPPORTED: LONG = 0x8010_001A;
pub const SCARD_E_DUPLICATE_READER: LONG = 0x8010_001B;
pub const SCARD_E_CARD_UNSUPPORTED: LONG = 0x8010_001C;
pub const SCARD_E_NO_SERVICE: LONG = 0x8010_001D;
pub const SCARD_E_SERVICE_STOPPED: LONG = 0x8010_001E;
pub const SCARD_E_UNEXPECTED: LONG = 0x8010_001F;
// See: https://pcsclite.apdu.fr/api/group__API.html#differences
#[cfg(not(target_os = "windows"))]
pub const SCARD_E_UNSUPPORTED_FEATURE: LONG = 0x8010_001F;
pub const SCARD_E_ICC_INSTALLATION: LONG = 0x8010_0020;
pub const SCARD_E_ICC_CREATEORDER: LONG = 0x8010_0021;
#[cfg(target_os = "windows")]
pub const SCARD_E_UNSUPPORTED_FEATURE: LONG = 0x8010_0022;
pub const SCARD_E_DIR_NOT_FOUND: LONG = 0x8010_0023;
pub const SCARD_E_FILE_NOT_FOUND: LONG = 0x8010_0024;
pub const SCARD_E_NO_DIR: LONG = 0x8010_0025;
pub const SCARD_E_NO_FILE: LONG = 0x8010_0026;
pub const SCARD_E_NO_ACCESS: LONG = 0x8010_0027;
pub const SCARD_E_WRITE_TOO_MANY: LONG = 0x8010_0028;
pub const SCARD_E_BAD_SEEK: LONG = 0x8010_0029;
pub const SCARD_E_INVALID_CHV: LONG = 0x8010_002A;
pub const SCARD_E_UNKNOWN_RES_MNG: LONG = 0x8010_002B;
pub const SCARD_E_NO_SUCH_CERTIFICATE: LONG = 0x8010_002C;
pub const SCARD_E_CERTIFICATE_UNAVAILABLE: LONG = 0x8010_002D;
pub const SCARD_E_NO_READERS_AVAILABLE: LONG = 0x8010_002E;
pub const SCARD_E_COMM_DATA_LOST: LONG = 0x8010_002F;
pub const SCARD_E_NO_KEY_CONTAINER: LONG = 0x8010_0030;
pub const SCARD_E_SERVER_TOO_BUSY: LONG = 0x8010_0031;

pub const SCARD_W_UNSUPPORTED_CARD: LONG = 0x8010_0065;
pub const SCARD_W_UNRESPONSIVE_CARD: LONG = 0x8010_0066;
pub const SCARD_W_UNPOWERED_CARD: LONG = 0x8010_0067;
pub const SCARD_W_RESET_CARD: LONG = 0x8010_0068;
pub const SCARD_W_REMOVED_CARD: LONG = 0x8010_0069;

pub const SCARD_W_SECURITY_VIOLATION: LONG = 0x8010_006A;
pub const SCARD_W_WRONG_CHV: LONG = 0x8010_006B;
pub const SCARD_W_CHV_BLOCKED: LONG = 0x8010_006C;
pub const SCARD_W_EOF: LONG = 0x8010_006D;
pub const SCARD_W_CANCELLED_BY_USER: LONG = 0x8010_006E;
pub const SCARD_W_CARD_NOT_AUTHENTICATED: LONG = 0x8010_006F;

pub const SCARD_W_CACHE_ITEM_NOT_FOUND: LONG = 0x8010_0070;
pub const SCARD_W_CACHE_ITEM_STALE: LONG = 0x8010_0071;
pub const SCARD_W_CACHE_ITEM_TOO_BIG: LONG = 0x8010_0072;

pub const SCARD_SCOPE_USER: DWORD = 0x0000;
pub const SCARD_SCOPE_TERMINAL: DWORD = 0x0001;
pub const SCARD_SCOPE_SYSTEM: DWORD = 0x0002;
pub const SCARD_SCOPE_GLOBAL: DWORD = 0x0003;

pub const SCARD_PROTOCOL_UNDEFINED: DWORD = 0x0000_0000;
pub const SCARD_PROTOCOL_UNSET: DWORD = SCARD_PROTOCOL_UNDEFINED;
pub const SCARD_PROTOCOL_T0: DWORD = 0x0000_0001;
pub const SCARD_PROTOCOL_T1: DWORD = 0x0000_0002;
#[cfg(not(target_os = "windows"))]
pub const SCARD_PROTOCOL_RAW: DWORD = 0x0000_0004;
#[cfg(target_os = "windows")]
pub const SCARD_PROTOCOL_RAW: DWORD = 0x0001_0000;
pub const SCARD_PROTOCOL_T15: DWORD = 0x0000_0008;
pub const SCARD_PROTOCOL_ANY: DWORD = SCARD_PROTOCOL_T0 | SCARD_PROTOCOL_T1;

pub const SCARD_SHARE_EXCLUSIVE: DWORD = 0x0001;
pub const SCARD_SHARE_SHARED: DWORD = 0x0002;
pub const SCARD_SHARE_DIRECT: DWORD = 0x0003;

pub const SCARD_LEAVE_CARD: DWORD = 0x0000;
pub const SCARD_RESET_CARD: DWORD = 0x0001;
pub const SCARD_UNPOWER_CARD: DWORD = 0x0002;
pub const SCARD_EJECT_CARD: DWORD = 0x0003;

/* Non-Windows (bitmask) */
#[cfg(not(target_os = "windows"))]
pub const SCARD_UNKNOWN: DWORD = 0x0001;
#[cfg(not(target_os = "windows"))]
pub const SCARD_ABSENT: DWORD = 0x0002;
#[cfg(not(target_os = "windows"))]
pub const SCARD_PRESENT: DWORD = 0x0004;
#[cfg(not(target_os = "windows"))]
pub const SCARD_SWALLOWED: DWORD = 0x0008;
#[cfg(not(target_os = "windows"))]
pub const SCARD_POWERED: DWORD = 0x0010;
#[cfg(not(target_os = "windows"))]
pub const SCARD_NEGOTIABLE: DWORD = 0x0020;
#[cfg(not(target_os = "windows"))]
pub const SCARD_SPECIFIC: DWORD = 0x0040;
/* Windows (ordinal) */
#[cfg(target_os = "windows")]
pub const SCARD_UNKNOWN: DWORD = 0;
#[cfg(target_os = "windows")]
pub const SCARD_ABSENT: DWORD = 1;
#[cfg(target_os = "windows")]
pub const SCARD_PRESENT: DWORD = 2;
#[cfg(target_os = "windows")]
pub const SCARD_SWALLOWED: DWORD = 3;
#[cfg(target_os = "windows")]
pub const SCARD_POWERED: DWORD = 4;
#[cfg(target_os = "windows")]
pub const SCARD_NEGOTIABLE: DWORD = 5;
#[cfg(target_os = "windows")]
pub const SCARD_SPECIFIC: DWORD = 6;

pub const SCARD_STATE_UNAWARE: DWORD = 0x0000;
pub const SCARD_STATE_IGNORE: DWORD = 0x0001;
pub const SCARD_STATE_CHANGED: DWORD = 0x0002;
pub const SCARD_STATE_UNKNOWN: DWORD = 0x0004;
pub const SCARD_STATE_UNAVAILABLE: DWORD = 0x0008;
pub const SCARD_STATE_EMPTY: DWORD = 0x0010;
pub const SCARD_STATE_PRESENT: DWORD = 0x0020;
pub const SCARD_STATE_ATRMATCH: DWORD = 0x0040;
pub const SCARD_STATE_EXCLUSIVE: DWORD = 0x0080;
pub const SCARD_STATE_INUSE: DWORD = 0x0100;
pub const SCARD_STATE_MUTE: DWORD = 0x0200;
pub const SCARD_STATE_UNPOWERED: DWORD = 0x0400;
pub const SCARD_AUTOALLOCATE: DWORD = !0;

pub const INFINITE: DWORD = 0xFFFF_FFFF;

pub const MAX_ATR_SIZE: usize = 33;
pub const MAX_BUFFER_SIZE: usize = 264;
pub const MAX_BUFFER_SIZE_EXTENDED: usize = 4 + 3 + (1 << 16) + 3 + 2;

#[cfg_attr(not(target_os = "macos"), repr(C))]
#[cfg_attr(target_os = "macos", repr(C, packed))]
pub struct SCARD_IO_REQUEST {
    pub dwProtocol: DWORD,
    pub cbPciLength: DWORD,
}

#[cfg(not(target_os = "windows"))]
pub const ATR_BUFFER_SIZE: usize = MAX_ATR_SIZE;
#[cfg(target_os = "windows")]
pub const ATR_BUFFER_SIZE: usize = 36;

#[cfg_attr(not(target_os = "macos"), repr(C))]
#[cfg_attr(target_os = "macos", repr(C, packed))]
pub struct SCARD_READERSTATE {
    pub szReader: *const c_char,
    pub pvUserData: *mut c_void,
    pub dwCurrentState: DWORD,
    pub dwEventState: DWORD,
    pub cbAtr: DWORD,
    pub rgbAtr: [u8; ATR_BUFFER_SIZE],
}

pub const SCARD_CLASS_VENDOR_INFO: ULONG = 1;
pub const SCARD_CLASS_COMMUNICATIONS: ULONG = 2;
pub const SCARD_CLASS_PROTOCOL: ULONG = 3;
pub const SCARD_CLASS_POWER_MGMT: ULONG = 4;
pub const SCARD_CLASS_SECURITY: ULONG = 5;
pub const SCARD_CLASS_MECHANICAL: ULONG = 6;
pub const SCARD_CLASS_VENDOR_DEFINED: ULONG = 7;
pub const SCARD_CLASS_IFD_PROTOCOL: ULONG = 8;
pub const SCARD_CLASS_ICC_STATE: ULONG = 9;
pub const SCARD_CLASS_SYSTEM: ULONG = 0;

macro_rules! SCARD_ATTR_VALUE {
    ($class:expr, $value:expr) => (
        ($class << 16) | $value
    )
}

pub const SCARD_ATTR_VENDOR_NAME: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_INFO, 0x0100);
pub const SCARD_ATTR_VENDOR_IFD_TYPE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_INFO, 0x0101);
pub const SCARD_ATTR_VENDOR_IFD_VERSION: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_INFO, 0x0102);
pub const SCARD_ATTR_VENDOR_IFD_SERIAL_NO: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_INFO, 0x0103);
pub const SCARD_ATTR_CHANNEL_ID: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_COMMUNICATIONS, 0x0110);
pub const SCARD_ATTR_ASYNC_PROTOCOL_TYPES: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_PROTOCOL, 0x0120);
pub const SCARD_ATTR_DEFAULT_CLK: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_PROTOCOL, 0x0121);
pub const SCARD_ATTR_MAX_CLK: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_PROTOCOL, 0x0122);
pub const SCARD_ATTR_DEFAULT_DATA_RATE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_PROTOCOL, 0x0123);
pub const SCARD_ATTR_MAX_DATA_RATE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_PROTOCOL, 0x0124);
pub const SCARD_ATTR_MAX_IFSD: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_PROTOCOL, 0x0125);
pub const SCARD_ATTR_SYNC_PROTOCOL_TYPES: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_PROTOCOL, 0x0126);
pub const SCARD_ATTR_POWER_MGMT_SUPPORT: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_POWER_MGMT, 0x0131);
pub const SCARD_ATTR_USER_TO_CARD_AUTH_DEVICE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SECURITY, 0x0140);
pub const SCARD_ATTR_USER_AUTH_INPUT_DEVICE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SECURITY, 0x0142);
pub const SCARD_ATTR_CHARACTERISTICS: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_MECHANICAL, 0x0150);

pub const SCARD_ATTR_CURRENT_PROTOCOL_TYPE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0201);
pub const SCARD_ATTR_CURRENT_CLK: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0202);
pub const SCARD_ATTR_CURRENT_F: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0203);
pub const SCARD_ATTR_CURRENT_D: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0204);
pub const SCARD_ATTR_CURRENT_N: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0205);
pub const SCARD_ATTR_CURRENT_W: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0206);
pub const SCARD_ATTR_CURRENT_IFSC: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0207);
pub const SCARD_ATTR_CURRENT_IFSD: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0208);
pub const SCARD_ATTR_CURRENT_BWT: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x0209);
pub const SCARD_ATTR_CURRENT_CWT: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x020a);
pub const SCARD_ATTR_CURRENT_EBC_ENCODING: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x020b);
pub const SCARD_ATTR_EXTENDED_BWT: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_IFD_PROTOCOL, 0x020c);

pub const SCARD_ATTR_ICC_PRESENCE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_ICC_STATE, 0x0300);
pub const SCARD_ATTR_ICC_INTERFACE_STATUS: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_ICC_STATE, 0x0301);
pub const SCARD_ATTR_CURRENT_IO_STATE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_ICC_STATE, 0x0302);
pub const SCARD_ATTR_ATR_STRING: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_ICC_STATE, 0x0303);
pub const SCARD_ATTR_ICC_TYPE_PER_ATR: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_ICC_STATE, 0x0304);

pub const SCARD_ATTR_ESC_RESET: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_DEFINED, 0xA000);
pub const SCARD_ATTR_ESC_CANCEL: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_DEFINED, 0xA003);
pub const SCARD_ATTR_ESC_AUTHREQUEST: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_DEFINED, 0xA005);
pub const SCARD_ATTR_MAXINPUT: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_VENDOR_DEFINED, 0xA007);

pub const SCARD_ATTR_DEVICE_UNIT: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SYSTEM, 0x0001);
pub const SCARD_ATTR_DEVICE_IN_USE: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SYSTEM, 0x0002);
pub const SCARD_ATTR_DEVICE_FRIENDLY_NAME_A: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SYSTEM, 0x0003);
pub const SCARD_ATTR_DEVICE_SYSTEM_NAME_A: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SYSTEM, 0x0004);
pub const SCARD_ATTR_DEVICE_FRIENDLY_NAME_W: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SYSTEM, 0x0005);
pub const SCARD_ATTR_DEVICE_SYSTEM_NAME_W: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SYSTEM, 0x0006);
pub const SCARD_ATTR_SUPRESS_T1_IFS_REQUEST: DWORD = SCARD_ATTR_VALUE!(SCARD_CLASS_SYSTEM, 0x0007);

// Assumes ASCII.
pub const SCARD_ATTR_DEVICE_FRIENDLY_NAME: DWORD = SCARD_ATTR_DEVICE_FRIENDLY_NAME_A;
pub const SCARD_ATTR_DEVICE_SYSTEM_NAME: DWORD = SCARD_ATTR_DEVICE_SYSTEM_NAME_A;

#[cfg(target_os = "windows")]
pub fn SCARD_CTL_CODE(code: DWORD) -> DWORD {
    0x0031_0000 | (code << 2)
}
#[cfg(not(target_os = "windows"))]
pub fn SCARD_CTL_CODE(code: DWORD) -> DWORD {
    0x4200_0000 + code
}

// The g_* statics only link if this is applied, even though the link
// is already specified in the build script. No idea why; oh well.
#[cfg_attr(target_os = "windows", link(name = "winscard"))]
extern "system" {
    pub static g_rgSCardT0Pci: SCARD_IO_REQUEST;
    pub static g_rgSCardT1Pci: SCARD_IO_REQUEST;
    pub static g_rgSCardRawPci: SCARD_IO_REQUEST;

    pub fn SCardEstablishContext(
        dwScope: DWORD,
        pvReserved1: *const c_void,
        pvReserved2: *const c_void,
        phContext: *mut SCARDCONTEXT,
    ) -> LONG;

    pub fn SCardReleaseContext(
        hContext: SCARDCONTEXT,
    ) -> LONG;

    pub fn SCardIsValidContext(
        hContext: SCARDCONTEXT,
    ) -> LONG;

    pub fn SCardCancel(
        hContext: SCARDCONTEXT,
    ) -> LONG;

    #[cfg_attr(target_os = "windows", link_name = "SCardConnectA")]
    pub fn SCardConnect(
        hContext: SCARDCONTEXT,
        szReader: *const c_char,
        dwShareMode: DWORD,
        dwPreferredProtocols: DWORD,
        phCard: *mut SCARDHANDLE,
        pdwActiveProtocol: *mut DWORD,
    ) -> LONG;

    pub fn SCardReconnect(
        hCard: SCARDHANDLE,
        dwShareMode: DWORD,
        dwPreferredProtocols: DWORD,
        dwInitialization: DWORD,
        pdwActiveProtocol: *mut DWORD,
    ) -> LONG;

    pub fn SCardDisconnect(
        hCard: SCARDHANDLE,
        dwDisposition: DWORD,
    ) -> LONG;

    #[cfg_attr(target_os = "windows", link_name = "SCardGetStatusChangeA")]
    pub fn SCardGetStatusChange(
        hContext: SCARDCONTEXT,
        dwTimeout: DWORD,
        rgReaderStates: *mut SCARD_READERSTATE,
        cReaders: DWORD,
    ) -> LONG;

    #[cfg_attr(target_os = "windows", link_name = "SCardListReadersA")]
    pub fn SCardListReaders(
        hContext: SCARDCONTEXT,
        mszGroups: *const c_char,
        mszReaders: *mut c_char,
        pcchReaders: *mut DWORD,
    ) -> LONG;

    pub fn SCardBeginTransaction(
        hCard: SCARDHANDLE,
    ) -> LONG;

    pub fn SCardEndTransaction(
        hCard: SCARDHANDLE,
        dwDisposition: DWORD,
    ) -> LONG;

    #[cfg_attr(target_os = "windows", link_name = "SCardStatusA")]
    pub fn SCardStatus(
        hCard: SCARDHANDLE,
        szReaderName: *mut c_char,
        pcchReaderLen: *mut DWORD,
        pdwState: *mut DWORD,
        pdwProtocol: *mut DWORD,
        pbAtr: *mut u8,
        pcbAtrLen: *mut DWORD,
    ) -> LONG;

    pub fn SCardGetAttrib(
        hCard: SCARDHANDLE,
        dwAttrId: DWORD,
        pbAttr: *mut u8,
        pcbAttrLen: *mut DWORD,
    ) -> LONG;

    pub fn SCardSetAttrib(
        hCard: SCARDHANDLE,
        dwAttrId: DWORD,
        pbAttr: *const u8,
        pcbAttrLen: DWORD,
    ) -> LONG;

    pub fn SCardTransmit(
        hCard: SCARDHANDLE,
        pioSendPci: *const SCARD_IO_REQUEST,
        pbSendBuffer: *const u8,
        cbSendLength: DWORD,
        pioRecvPci: *mut SCARD_IO_REQUEST,
        pbRecvBuffer: *mut u8,
        pcbRecvLength: *mut DWORD,
    ) -> LONG;

    pub fn SCardControl(
        hCard: SCARDHANDLE,
        dwControlCode: DWORD,
        pbSendBuffer: *const u8,
        cbSendLength: DWORD,
        pbRecvBuffer: *mut u8,
        cbRecvLength: DWORD,
        lpBytesReturned: *mut DWORD,
    ) -> LONG;
}
