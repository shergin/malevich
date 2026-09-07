//! The raw-mode terminal round trip: write a query batch, read replies.
//!
//! The one place the crate talks *to* a terminal rather than into a string.
//! I/O goes through `/dev/tty` (correct even when stdout or stdin is
//! redirected), in raw non-echoing mode so replies are neither displayed nor
//! line-buffered, with the previous settings restored on every path. Reads are
//! paced by `VTIME` (100 ms of silence per attempt) under a total budget, so a
//! terminal that answers nothing costs a fraction of a second exactly once —
//! the caller caches. Replies are capped because the four expected reports are
//! small and no terminal-controlled byte stream should grow memory without bound.

use super::probe::MAX_REPLY_BYTES;

/// Appends the prefix that fits and reports whether the reply buffer is full.
/// Compiled on every target so Windows tests the cap; `exchange` is Unix-only.
#[cfg_attr(not(unix), allow(dead_code))]
fn append_reply(replies: &mut Vec<u8>, chunk: &[u8]) -> bool {
    let remaining = MAX_REPLY_BYTES.saturating_sub(replies.len());
    replies.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
    replies.len() >= MAX_REPLY_BYTES
}

#[cfg(unix)]
pub(crate) fn exchange(
    queries: &str,
    budget: std::time::Duration,
    done: impl Fn(&[u8]) -> bool,
) -> Option<Vec<u8>> {
    use std::io::{Read, Write};

    use rustix::termios::{self, OptionalActions, SpecialCodeIndex};

    /// Restores the saved termios when the exchange ends, however it ends.
    struct Restore<'t> {
        tty: &'t std::fs::File,
        saved: termios::Termios,
    }
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            let _ = termios::tcsetattr(self.tty, OptionalActions::Now, &self.saved);
        }
    }

    let tty = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .ok()?;
    let saved = termios::tcgetattr(&tty).ok()?;
    let mut raw = saved.clone();
    raw.make_raw();
    // VMIN = 0, VTIME = 1: each read returns what is available, or empty after
    // 100 ms of silence — the polling clock, without a poll syscall.
    raw.special_codes[SpecialCodeIndex::VMIN] = 0;
    raw.special_codes[SpecialCodeIndex::VTIME] = 1;
    termios::tcsetattr(&tty, OptionalActions::Now, &raw).ok()?;
    let restore = Restore { tty: &tty, saved };

    (&tty).write_all(queries.as_bytes()).ok()?;
    (&tty).flush().ok()?;

    let start = std::time::Instant::now();
    let mut replies = Vec::new();
    let mut chunk = [0u8; 512];
    while start.elapsed() < budget && !done(&replies) {
        let remaining = MAX_REPLY_BYTES.saturating_sub(replies.len());
        if remaining == 0 {
            break;
        }
        let read_len = remaining.min(chunk.len());
        match (&tty).read(&mut chunk[..read_len]) {
            Ok(0) => continue,
            Ok(count) if append_reply(&mut replies, &chunk[..count]) => break,
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    drop(restore);
    Some(replies)
}

#[cfg(not(unix))]
pub(crate) fn exchange(
    _queries: &str,
    _budget: std::time::Duration,
    _done: impl Fn(&[u8]) -> bool,
) -> Option<Vec<u8>> {
    None
}

#[cfg(test)]
#[path = "tests/query_tests.rs"]
mod tests;
