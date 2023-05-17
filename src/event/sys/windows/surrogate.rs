use std::fmt;

use crate::event::{KeyEventKind, KeyEventState, KeyModifiers};

pub(crate) const MIN_HIGH_SURROGATE: u16 = 0xD800;
pub(crate) const MAX_HIGH_SURROGATE: u16 = 0xDBFF;
pub(crate) const MIN_LOW_SURROGATE: u16 = 0xDC00;
pub(crate) const MAX_LOW_SURROGATE: u16 = 0xDFFF;

/// Platform-specific error type representing an illegal surrogate code points.
///
/// In windows, the [crate::event::read] is receives character in encoded UTF-16.
/// In UTF-16, code points greater than BMP are divided into surrogate pairs and
/// decoded to a single supplementary code point, but illegal surrogate pairs may be apper.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IllegalSurrogate(String);

impl fmt::Display for IllegalSurrogate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl std::error::Error for IllegalSurrogate {}

pub(crate) struct HighSurrogate {
    pub(crate) high: u16,
    pub(crate) modifiers: KeyModifiers,
    pub(crate) key_event_kind: KeyEventKind,
    pub(crate) key_event_state: KeyEventState,
}
pub(crate) struct LowSurrogate {
    pub(crate) low: u16,
    pub(crate) key_event_kind: KeyEventKind,
}

/// Convert surrogate pair to supplementary code point.
///
/// `high` and `low` pass UTF-16 encoded surrogate pair.
pub(crate) fn to_supplementary_code_point(high: u16, low: u16) -> u32 {
    // https://unicode.org/faq/utf_bom.html
    (((high & 0x3ff) as u32) << 10 | (low & 0x3ff) as u32) + 0x10000
}
