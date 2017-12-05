// Example of communication with a smart card.

extern crate pcsc;

use pcsc::*;

fn main() {
    // Get a context.
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    // List connected readers.
    let mut readers_buf = [0; 2048];
    let readers = ctx.list_readers(&mut readers_buf).expect("failed to list readers").collect::<Vec<_>>();
    println!("Readers: {:?}", readers);

    if readers.is_empty() {
        return;
    }

    {
        // Try to connect to a card in the first reader.
        let mut card = ctx.connect(readers[0], ShareMode::Exclusive, Protocols::ANY).expect("failed to connect to card");

        {
            // Start an exclusive transaction (not required -- can work on card directly).
            let tx = card.transaction().expect("failed to begin card transaction");

            // Get the card status.
            let (status, protocol) = tx.status().expect("failed to get card status");
            println!("Status: {:?}", status);
            println!("Protocol: {:?}", protocol);

            // Send some harmless APDU to the card.
            let apdu = b"\x00\xa4\x04\x00\x08\x31\x54\x49\x43\x2e\x49\x43\x41";
            let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
            let rapdu = tx.transmit(apdu, &mut rapdu_buf).expect("failed to transmit APDU to card");
            println!("RAPDU: {:?}", rapdu);

            // Get the card's ATR.
            let mut atr_buf = [0; MAX_ATR_SIZE];
            let atr = tx.get_attribute(Attribute::AtrString, &mut atr_buf).expect("failed to get ATR attribute");
            println!("ATR: {:?}", atr);

            // Get some attribute.
            let mut ifd_version_buf = [0; 4];
            let ifd_version = tx.get_attribute(Attribute::VendorIfdVersion, &mut ifd_version_buf).expect("failed to get vendor IFD version attribute");
            println!("Vendor IFD version: {:?}", ifd_version);

            // Get some other attribute.
            // This time we allocate a buffer of the needed length.
            let vendor_name_len = tx.get_attribute_len(Attribute::VendorName).expect("failed to get the vendor name attribute length");
            let mut vendor_name_buf = vec![0; vendor_name_len];
            let vendor_name = tx.get_attribute(Attribute::VendorName, &mut vendor_name_buf).expect("failed to get vendor name attribute");
            println!("Vendor name: {}", std::str::from_utf8(vendor_name).unwrap());

            // Can either end explicity, which allows error handling,
            // and setting the disposition method, or leave it to drop, which
            // swallows any error and hardcodes LeaveCard.
            tx.end(Disposition::LeaveCard).map_err(|(_, err)| err).expect("failed to end transaction");
        }

        // Can either disconnect explicity, which allows error handling,
        // and setting the disposition method, or leave it to drop, which
        // swallows any error and hardcodes ResetCard.
        card.disconnect(Disposition::ResetCard).map_err(|(_, err)| err).expect("failed to disconnect from card");
    }

    // Can either release explicity, which allows error handling,
    // or leave it to drop, which swallows any error.
    ctx.release().map_err(|(_, err)| err).expect("failed to release context");
}
