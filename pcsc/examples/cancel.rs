// Example of how to cancel a blocking Context operation from another
// thread.
use std::time::Duration;

use pcsc::*;

fn wait_for_enter_keypress() {
    use std::io::Read;
    let mut stdin = std::io::stdin();
    let _ = stdin.read(&mut [0]).expect("failed to read key");
}

fn main() {
    // Get a context.
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    // Spawn a thread which waits for a key-press then cancels the operation.
    {
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            wait_for_enter_keypress();
            ctx.cancel().expect("failed to cancel");
        });
    }

    // Set up the blocking call, and wait for cancel or timeout.
    println!("Entering blocking call; press Enter to cancel");
    let mut reader_states = vec![ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE)];
    match ctx.get_status_change(Duration::from_secs(5), &mut reader_states) {
        Ok(()) => {
            println!("Blocking call exited normally");
        }
        Err(Error::Cancelled) => {
            println!("Blocking call canceled");
        }
        Err(Error::Timeout) => {
            println!("Blocking call timed out");
        }
        Err(error) => {
            panic!("failed to get status changes: {:?}", error);
        }
    }
}
