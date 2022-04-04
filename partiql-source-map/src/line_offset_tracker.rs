use crate::location::{ByteOffset, BytePosition, LineAndCharPosition, LineOffset};
use smallvec::{smallvec, SmallVec};
use std::ops::Range;

/// Keeps track of source offsets of newlines for the purposes of later calculating
/// line and column information
///
///
/// ## Example
///
/// ```rust
/// use partiql_source_map::location::{ByteOffset, LineAndCharPosition};
/// use partiql_source_map::line_offset_tracker::LineOffsetTracker;
///
/// let source = "12345\n789012345\n789012345\n789012345";
/// let mut tracker = LineOffsetTracker::default();
/// tracker.record(6.into());
/// tracker.record(16.into());
/// tracker.record(26.into());
///
/// // We added 3 newlines, so there should be 4 lines of source
/// assert_eq!(tracker.num_lines(), 4);
/// assert_eq!(tracker.at(source, ByteOffset(0).into()), LineAndCharPosition::new(0,0));
/// assert_eq!(tracker.at(source, ByteOffset(6).into()), LineAndCharPosition::new(1,0));
/// assert_eq!(tracker.at(source, ByteOffset(30).into()), LineAndCharPosition::new(3,4));
/// ```
pub struct LineOffsetTracker {
    line_starts: SmallVec<[ByteOffset; 16]>,
}

impl Default for LineOffsetTracker {
    fn default() -> Self {
        LineOffsetTracker {
            line_starts: smallvec![ByteOffset(0)], // line 1 starts at offset `0`
        }
    }
}

impl LineOffsetTracker {
    /// Record a newline at `span` in the source
    #[inline(always)]
    pub fn record(&mut self, line_start: ByteOffset) {
        self.line_starts.push(line_start);
    }

    /// Append the line starts from another [`LineOffsetTracker`] to this one, adding `offset` to each.
    #[inline(always)]
    pub fn append(&mut self, other: &LineOffsetTracker, offset: ByteOffset) {
        // skip the first offset in `other`; it is the `0` added by `LineOffsetTracker::default()`
        for start in &other.line_starts[1..] {
            self.record(offset + *start);
        }
    }

    /// Calculate the number of lines of source seen so far.
    #[inline(always)]
    pub fn num_lines(&self) -> usize {
        self.line_starts.len()
    }

    /// Calculates the byte offset span ([`Range`]) of a line.
    ///
    /// `num` is the line number (0-indexed) for which to  calculate the span
    /// `max` is the largest value allowable in the returned [`Range's end`](core::ops::Range)
    #[inline(always)]
    fn byte_span_from_line_num(&self, num: LineOffset, max: ByteOffset) -> Range<ByteOffset> {
        let start = self.line_starts[num.to_usize()];
        let end = self
            .line_starts
            .get((num + 1).to_usize())
            .unwrap_or(&max)
            .min(&max);
        start..*end
    }

    /// Calculates the line number (0-indexed) in which a byte offset is contained.
    ///
    /// `offset` is the byte offset
    #[inline(always)]
    fn line_num_from_byte_offset(&self, offset: ByteOffset) -> LineOffset {
        match self.line_starts.binary_search(&offset) {
            Err(i) => i - 1,
            Ok(i) => i,
        }
        .into()
    }

    /// Calculates a [`LineAndCharPosition`] for a byte offset from the given `&str`
    ///
    /// `source` is source `&str` into which the byte offset applies
    /// `offset` is the byte offset for which to find the [`LineAndCharPosition`]
    ///
    /// If `offset` is larger than `source.len()`, then a position one beyond the
    /// last character is returned.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    ///  - `offset` falls inside a unicode codepoint
    pub fn at(&self, source: &str, BytePosition(offset): BytePosition) -> LineAndCharPosition {
        match offset {
            ByteOffset(0) => LineAndCharPosition::new(0, 0),
            ByteOffset(n) if n >= source.len() as u32 => {
                let LineAndCharPosition { line, char } =
                    self.at(source, BytePosition((source.len() - 1).into()));
                let char = char + 1;
                LineAndCharPosition { line, char }
            }
            _ => {
                let line_num = self.line_num_from_byte_offset(offset);
                let line_span = self.byte_span_from_line_num(line_num, source.len().into());
                let span = line_span.start.to_usize()..=offset.to_usize();
                let column_num = source[span].chars().count();

                LineAndCharPosition::new(line_num.to_usize(), column_num - 1)
            }
        }
    }
}
