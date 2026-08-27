use std::ops::{Index, Range};

use crate::error_handler::SourceId;

/// Represents a section of source code. These can be used to display where
/// an error came from. Please note that each source in an error handler has
/// its own offset, which is added to both the `start` and `end` fields, and
/// can be obtained using [`ErrorHandler::get_source_offset`]. These are used
/// to determine which source a span came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Span {
    /// The byte index at which the span starts.
    pub start: u32,
    /// The byte index at which the span ends.
    pub end: u32,
    /// This field exists to prevent people from creating their own spans
    /// without adding a source offset.
    _marker: (),
}

impl Span {
    /// Create a new span that points to a byte range in `source`.
    pub fn new(range: Range<u32>, source: SourceId) -> Self {
        Self {
            start: range.start + source.span_offset,
            end: range.end + source.span_offset,
            _marker: (),
        }
    }

    /// Create a new span that points to a single byte in `source`.
    pub fn char(pos: u32, source: SourceId) -> Self {
        Self::new(pos..pos + 1, source)
    }
}

impl From<Range<Span>> for Span {
    fn from(
        Range {
            start: Span { start, .. },
            end: Span { end, .. },
        }: Range<Span>,
    ) -> Self {
        Self {
            start,
            end,
            _marker: (),
        }
    }
}

/// Internal version of the [`Span`]. This struct should not take into account
/// a source's offset, and should instead be used in contexts where string
/// indexing is involved.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct RawSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl From<Range<usize>> for RawSpan {
    fn from(Range { start, end }: Range<usize>) -> Self {
        Self { start, end }
    }
}

impl RawSpan {
    pub(crate) fn char(pos: usize) -> Self {
        Self::from(pos..pos + 1)
    }
}

impl Index<RawSpan> for str {
    type Output = str;

    fn index(&self, index: RawSpan) -> &Self::Output {
        &self[index.start..index.end]
    }
}
