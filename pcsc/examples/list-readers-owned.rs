// Example of how to enumerate connected card readers.
use pcsc::*;

fn main() {
    // Get a context.
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    // Instead of manually allocating the buffer for the reader names, we let pcsc take care of
    // that
    let names = ctx.list_readers_owned().expect("failed to list readers");
    for name in names {
        println!("{:?}", name);
    }
}
