//! The capability probe: one batch of queries, one pure reply parser.
//!
//! Unlike glyph coverage, pixel protocols are actively probeable. The batch
//! asks four questions and ends with a barrier: the kitty graphics query
//! (answered only by terminals that speak the protocol), XTVERSION (the
//! terminal's name — `TERM_PROGRAM` that survives ssh), XTSMGRAPHICS (sixel
//! color registers), `CSI 16 t` (cell size in device pixels, more reliable
//! than `TIOCGWINSZ`), and finally DA1 — which every terminal since the VT100
//! answers, and replies arrive in order, so its answer means every earlier
//! answer that was coming has arrived.
//!
//! Parsing is pure over the reply bytes: fixtures test it without a terminal.

/// Maximum aggregate reply bytes retained and parsed from one probe.
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) const MAX_REPLY_BYTES: usize = 4 * 1024;

/// The query batch, barrier last.
pub(crate) const QUERIES: &str = concat!(
    // Kitty graphics support probe from the protocol docs: a 1×1 query-action
    // transmission with a correlation id; terminals that speak the protocol
    // reply `OK`, everyone else silently discards the APC.
    "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\",
    // XTVERSION: terminal name and version as a DCS `>|…` report.
    "\x1b[>q",
    // XTSMGRAPHICS: read (action 1) the number of sixel color registers.
    "\x1b[?1;1;0S",
    // Cell size in device pixels: `CSI 6 ; height ; width t` comes back.
    "\x1b[16t",
    // DA1, the barrier; attribute 4 in the reply advertises sixel.
    "\x1b[c",
);

/// What the replies said. `answered` is the barrier: false means the probe
/// never reached a responding terminal and nothing here is evidence.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Report {
    pub kitty: bool,
    pub terminal: Option<String>,
    pub sixel: bool,
    pub cell_size: Option<(u16, u16)>,
    pub answered: bool,
}

/// Whether the DA1 barrier reply has arrived.
pub(crate) fn done(bytes: &[u8]) -> bool {
    sequences(bytes).iter().any(|s| {
        matches!(
            s,
            Sequence::Csi {
                private: true,
                terminator: 'c',
                ..
            }
        )
    })
}

/// Interprets the reply stream; unknown and partial sequences are skipped.
pub(crate) fn parse(bytes: &[u8]) -> Report {
    let mut report = Report::default();
    for sequence in sequences(bytes) {
        match sequence {
            Sequence::Apc(body) => {
                if body.contains("i=31") && body.contains("OK") {
                    report.kitty = true;
                }
            }
            Sequence::Dcs(body) => {
                if let Some(name) = body.strip_prefix(">|") {
                    report.terminal = Some(name.trim().to_string());
                }
            }
            Sequence::Csi {
                private: true,
                terminator: 'c',
                params,
            } => {
                report.answered = true;
                if params.contains(&4) {
                    report.sixel = true;
                }
            }
            Sequence::Csi {
                private: true,
                terminator: 'S',
                params,
            } => {
                // `? 1 ; status ; value S`: item 1 is color registers,
                // status 0 is success.
                if params.first() == Some(&1) && params.get(1) == Some(&0) {
                    report.sixel = true;
                }
            }
            Sequence::Csi {
                private: false,
                terminator: 't',
                params,
            } => {
                // `6 ; height ; width t` — sanity-gated like TIOCGWINSZ: a
                // hairline cell is a terminal reporting nonsense.
                if params.first() == Some(&6)
                    && let (Some(&height), Some(&width)) = (params.get(1), params.get(2))
                    && width >= 2
                    && height >= 4
                    && width <= u32::from(u16::MAX)
                    && height <= u32::from(u16::MAX)
                {
                    report.cell_size = Some((width as u16, height as u16));
                }
            }
            Sequence::Csi { .. } => {}
        }
    }
    report
}

/// One recognized escape reply.
enum Sequence {
    Csi {
        private: bool,
        params: Vec<u32>,
        terminator: char,
    },
    /// A DCS string, introducer stripped.
    Dcs(String),
    /// An APC string, introducer stripped.
    Apc(String),
}

/// Splits the byte stream into complete escape sequences, skipping anything
/// unrecognized or still partial (the reader calls this on a growing buffer).
fn sequences(bytes: &[u8]) -> Vec<Sequence> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }
        match bytes[i + 1] {
            b'[' => {
                let mut j = i + 2;
                while j < bytes.len() && !(0x40..=0x7E).contains(&bytes[j]) {
                    j += 1;
                }
                let Some(&terminator) = bytes.get(j) else {
                    break;
                };
                let body = &bytes[i + 2..j];
                let private = body.first() == Some(&b'?');
                let digits = if private { &body[1..] } else { body };
                let params = digits
                    .split(|&b| b == b';')
                    .filter_map(|segment| std::str::from_utf8(segment).ok()?.parse::<u32>().ok())
                    .collect();
                out.push(Sequence::Csi {
                    private,
                    params,
                    terminator: char::from(terminator),
                });
                i = j + 1;
            }
            kind @ (b'P' | b'_') => {
                // String sequences end at ST (ESC \).
                let mut j = i + 2;
                while j + 1 < bytes.len() && !(bytes[j] == 0x1b && bytes[j + 1] == b'\\') {
                    j += 1;
                }
                if j + 1 >= bytes.len() {
                    break;
                }
                let body = String::from_utf8_lossy(&bytes[i + 2..j]).into_owned();
                out.push(if kind == b'P' {
                    Sequence::Dcs(body)
                } else {
                    Sequence::Apc(body)
                });
                i = j + 2;
            }
            _ => i += 1,
        }
    }
    out
}

#[cfg(test)]
#[path = "tests/probe_tests.rs"]
mod tests;
