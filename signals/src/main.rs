use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, SigSet, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

// We must use a global static because C-function pointers cannot capture environment variables.
static RUNNING: AtomicBool = AtomicBool::new(true);

// The actual signal handler must be an `extern "C"` function.
// WARNING: Do not use println!, allocate memory, or lock mutexes in here!
extern "C" fn handle_sigint(_sig: libc::c_int) {
    // Safely tell the main thread to stop
    RUNNING.store(false, Ordering::Relaxed);

    println!("Hello, world!");
}

fn main() {
    nix::unistd::write(std::io::stdout(), "Hello, world!\n".as_bytes()).unwrap();

    // 1. Define the handler function
    let handler = SigHandler::Handler(handle_sigint);

    // 2. Configure the signal action
    let action = SigAction::new(handler, SaFlags::empty(), SigSet::empty());

    // 3. Register the signal handler for SIGINT (Ctrl-C)
    // This is unsafe because setting a signal handler can interrupt normal control flow
    unsafe {
        signal::sigaction(Signal::SIGINT, &action).expect("Failed to register signal handler");
    }

    println!("Process is running. Press Ctrl-C to stop.");

    // 4. Main loop checks the atomic flag
    while RUNNING.load(Ordering::Relaxed) {
        println!("inside the loop");
        thread::sleep(Duration::from_millis(100));
    }

    println!("Graceful shutdown complete.");
}
