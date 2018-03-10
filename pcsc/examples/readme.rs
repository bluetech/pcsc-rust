extern crate pcsc;

use pcsc::*;

fn main() {
    // Establish a PC/SC context.
    let ctx = Context::establish(Scope::User)
        .expect("failed to establish context");

    // List available readers.
    let mut readers_buf = [0; 2048];
    let mut readers = ctx.list_readers(&mut readers_buf)
        .expect("failed to list readers");

    // Use the first reader.
    let reader = readers.next().ok_or(())
        .expect("no readers are connected");
    println!("Using reader: {:?}", reader);

    // Connect to the card.
    let card = ctx.connect(reader, ShareMode::Shared, Protocols::ANY)
        .expect("failed to connect to card");

    // Send an APDU command.
    let apdu = b"\x00\xa4\x04\x00\x0A\xA0\x00\x00\x00\x62\x03\x01\x0C\x06\x01";
    let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
    let rapdu = card.transmit(apdu, &mut rapdu_buf)
        .expect("failed to transmit APDU to card");
    println!("{:?}", rapdu);
}
