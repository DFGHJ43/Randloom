//! Graceful interrupt handling.
//!
//! On Unix, SIGINT/SIGTERM set an atomic flag that the event loop polls, so
//! the terminal is restored on every exit path. On Windows, Ctrl+C is
//! delivered by crossterm as an ordinary key event once raw mode is active;
//! there is no SIGTERM equivalent to install.

#[cfg(unix)]
mod imp {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, OnceLock};

    use signal_hook::consts::{SIGINT, SIGTERM};
    use signal_hook::flag;

    static INTERRUPTED: OnceLock<Arc<AtomicBool>> = OnceLock::new();

    pub fn install() {
        let flag = INTERRUPTED.get_or_init(|| Arc::new(AtomicBool::new(false)));
        for signal in [SIGINT, SIGTERM] {
            let _ = flag::register(signal, Arc::clone(flag));
        }
    }

    pub fn interrupted() -> bool {
        INTERRUPTED
            .get()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
    }
}

#[cfg(not(unix))]
mod imp {
    pub fn install() {}

    pub fn interrupted() -> bool {
        false
    }
}

pub use imp::{install, interrupted};
