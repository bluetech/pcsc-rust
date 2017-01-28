// Example of how to enumerate connected card readers.

extern crate pcsc;

use pcsc::*;

fn main() {
    // Get a context.
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    // First allocate the required buffer.
    let len = ctx.list_readers_len().expect("failed to list readers needed len");
    let mut readers_buf = vec![0; len];
    // Alternatively, we could have just used a sufficiently large
    // statically sized, stack allocated buffer instead, like we do in
    // other examples:
    // let mut readers_buf = [0; 2048];

    let names = ctx.list_readers(&mut readers_buf).expect("failed to list readers");
    for name in names {
        println!("{:?}", name);
    }
}
