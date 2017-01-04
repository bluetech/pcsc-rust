// Example of how to monitor card & card reader state changes.

extern crate pcsc;

use pcsc::*;

fn main() {
    let ctx = Context::establish(Scope::User).expect("failed to establish context");

    let mut readers_buf = [0; 2048];
    let mut reader_states = vec![
        // Listen for reader insertions/removals.
        ReaderState::new(PNP_NOTIFICATION(), STATE_UNAWARE),
    ];
    loop {
        // Remove dead readers.
        fn is_dead(rs: &ReaderState) -> bool {
            rs.event_state().intersects(STATE_UNKNOWN | STATE_IGNORE)
        }
        for rs in &reader_states {
            if is_dead(rs) {
                println!("Removing {:?}", rs.name());
            }
        }
        reader_states.retain(|ref rs| !is_dead(rs));

        // Add new readers.
        let names = ctx.list_readers(&mut readers_buf).expect("failed to list readers");
        for name in names {
            if !reader_states[1..].iter().any(|rs| rs.name() == name) {
                println!("Adding {:?}", name);
                reader_states.push(ReaderState::new(name, STATE_UNAWARE));
            }
        }

        // Update the view of the state to wait on.
        for rs in &mut reader_states {
            rs.sync_current_state();
        }

        // Wait until the state changes.
        ctx.get_status_change(None, &mut reader_states).expect("failed to get status change");

        // Print current state.
        println!();
        for rs in &reader_states[1..] {
            println!("{:?} {:?}", rs.name(), rs.event_state());
        }
    }
}
