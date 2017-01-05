// Example of how to cancel a blocking Context operation from another
// thread.

extern crate pcsc;

use pcsc::*;
use std::time::Duration;

fn wait_for_enter_keypress() {
    use std::io::Read;
    let mut stdin = std::io::stdin();
    let _ = stdin.read(&mut [0]).expect("failed to read key");
}

fn main() {
    // Get a context.
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    // Get a "canceler" that can be passed to another thread.
    let canceler = ctx.get_canceler();

    // Spawn a thread which wants the ability the cancel a blocking call;
    // let it have the canceler.
    std::thread::spawn(move || {
        wait_for_enter_keypress();
        canceler.cancel().expect("failed to cancel");
    });

    // Set up the blocking call, and wait for cancel or timeout.
    println!("Entering blocking call; press Enter to cancel");
    let mut reader_states = vec![
        ReaderState::new(
            PNP_NOTIFICATION(),
            STATE_UNAWARE,
        ),
    ];
    match ctx.get_status_change(Duration::from_secs(5), &mut reader_states) {
        Ok(()) => {
            println!("Blocking call exited normally");
        },
        Err(Error::Cancelled) => {
            println!("Blocking call canceled");
        },
        Err(Error::Timeout) => {
            println!("Blocking call timed out");
        },
        Err(error) => {
            panic!("failed to get status changes: {:?}", error);
        },
    }
}
