// Example of how to enumerate connected card readers.

extern crate pcsc;

use pcsc::*;

fn main() {
    // Get a context.
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    // Just list the readers.
    let mut readers_buf = [0; 2048];
    let names = ctx.list_readers(&mut readers_buf).expect("failed to list readers");
    for name in names {
        println!("{:?}", name);
    }
}
