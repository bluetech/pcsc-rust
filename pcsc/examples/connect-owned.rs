// Example of communication with a smart card.

extern crate pcsc;

use pcsc::*;

fn main() {
    // Get a context.
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    // List connected readers.
    let readers = ctx.list_readers_owned().expect("failed to list readers");
    println!("Readers: {:?}", readers);

    if readers.is_empty() {
        return;
    }

    {
        // Try to connect to a card in the first reader.
        let mut card = ctx.connect(&readers[0], ShareMode::Shared, Protocols::ANY).expect("failed to connect to card");

        {
            // Start an exclusive transaction (not required -- can work on card directly).
            let tx = card.transaction().expect("failed to begin card transaction");

            // Get the card status.
            let status = tx.status2_owned().expect("failed to get card status");
            println!("Reader names from status: {:?}", status.reader_names());
            if let Some(protocol) = status.protocol2() {
                println!("Protocol from status: {:?}", protocol);
            } else {
                println!("Protocol from status: directly connected");
            }
            println!("ATR from status: {:?}", status.atr());

            // Send some harmless APDU to the card.
            if let Some(_) = status.protocol2() {
                let apdu = b"\x00\xa4\x04\x00\x08\x31\x54\x49\x43\x2e\x49\x43\x41";
                let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
                let rapdu = tx.transmit(apdu, &mut rapdu_buf).expect("failed to transmit APDU to card");
                println!("RAPDU: {:?}", rapdu);
            }

            // Get the card's ATR.
            let atr = tx.get_attribute_owned(Attribute::AtrString).expect("failed to get ATR attribute");
            println!("ATR from attribute: {:?}", atr);

            // Get some attribute.
            let ifd_version = tx.get_attribute_owned(Attribute::VendorIfdVersion).expect("failed to get vendor IFD version attribute");
            println!("Vendor IFD version: {:?}", ifd_version);

            // Get some other attribute.
            let vendor_name = tx.get_attribute_owned(Attribute::VendorName).expect("failed to get vendor name attribute");
            println!("Vendor name: {}", String::from_utf8(vendor_name).unwrap());

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
    // The function fails if there are any live clones.
    ctx.release().map_err(|(_, err)| err).expect("failed to release context");
}
