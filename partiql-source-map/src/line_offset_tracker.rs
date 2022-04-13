//! [`LineOffsetTracker`] and related types for mapping locations in source `str`s.

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
/// use partiql_source_map::line_offset_tracker::{LineOffsetError, LineOffsetTracker};
///
/// let source = "12345\n789012345\n789012345\n789012345";
/// let mut tracker = LineOffsetTracker::default();
/// tracker.record(6.into());
/// tracker.record(16.into());
/// tracker.record(26.into());
///
/// // We added 3 newlines, so there should be 4 lines of source
/// assert_eq!(tracker.num_lines(), 4);
/// assert_eq!(tracker.at(source, ByteOffset(0).into()), Ok(LineAndCharPosition::new(0,0)));
/// assert_eq!(tracker.at(source, ByteOffset(6).into()), Ok(LineAndCharPosition::new(1,0)));
/// assert_eq!(tracker.at(source, ByteOffset(30).into()), Ok(LineAndCharPosition::new(3,4)));
/// assert_eq!(tracker.at(source, ByteOffset(300).into()), Err(LineOffsetError::EndOfInput));
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

/// Errors that can be encountered when indexing by byte offset.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LineOffsetError {
    /// Requested `offset` is past end of input
    EndOfInput,
    /// Requested `offset` falls inside a unicode codepoint
    InsideUnicodeCodepoint,
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
    /// If `offset` is larger than `source.len()`, then [`LineOffsetError::EndOfInput`] is returned
    /// If `offset` is in the middle of a unicode codepoint, then [`LineOffsetError::InsideUnicodeCodepoint`] is returned
    pub fn at(
        &self,
        source: &str,
        BytePosition(offset): BytePosition,
    ) -> Result<LineAndCharPosition, LineOffsetError> {
        let full_len = source.len() as u32;
        match offset {
            ByteOffset(0) => Ok(LineAndCharPosition::new(0, 0)),
            ByteOffset(n) if n >= full_len => Err(LineOffsetError::EndOfInput),
            _ => {
                let line_num = self.line_num_from_byte_offset(offset);
                let line_span = self.byte_span_from_line_num(line_num, source.len().into());
                let limit = (offset - line_span.start).0 as usize;
                let line = &source[line_span.start.0 as usize..line_span.end.0 as usize];
                let column_num = line
                    .char_indices()
                    .enumerate()
                    .find(|(_i, (idx, _char))| idx == &limit);

                match column_num {
                    None => Err(LineOffsetError::InsideUnicodeCodepoint),
                    Some((column_num, (_idx, _char))) => {
                        Ok(LineAndCharPosition::new(line_num.to_usize(), column_num))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker_from_str(s: &str) -> LineOffsetTracker {
        let mut tracker = LineOffsetTracker::default();
        let mut start = 0;
        while let Some(l) = s[start..].find('\n') {
            let span = (start + l)..(start + l + 1);
            tracker.record(span.end.into());
            start += l + 1;
        }
        tracker
    }

    #[test]
    fn simple() {
        let s = "01\n345";
        let tracker = tracker_from_str(s);

        assert_eq!(tracker.num_lines(), 2);

        assert_eq!(&s[0..1], "0");
        assert_eq!(
            tracker.at(s, 0.into()).unwrap(),
            LineAndCharPosition::new(0, 0)
        );
        assert_eq!(&s[1..2], "1");
        assert_eq!(
            tracker.at(s, 1.into()).unwrap(),
            LineAndCharPosition::new(0, 1)
        );
        assert_eq!(&s[2..3], "\n");
        assert_eq!(
            tracker.at(s, 2.into()).unwrap(),
            LineAndCharPosition::new(0, 2)
        );
        assert_eq!(&s[3..4], "3");
        assert_eq!(
            tracker.at(s, 3.into()).unwrap(),
            LineAndCharPosition::new(1, 0)
        );
        assert_eq!(&s[4..5], "4");
        assert_eq!(
            tracker.at(s, 4.into()).unwrap(),
            LineAndCharPosition::new(1, 1)
        );
        assert_eq!(&s[5..6], "5");
        assert_eq!(
            tracker.at(s, 5.into()).unwrap(),
            LineAndCharPosition::new(1, 2)
        );
        assert_eq!(s.len(), 6);
        assert_eq!(tracker.at(s, 6.into()), Err(LineOffsetError::EndOfInput));
        assert_eq!(tracker.at(s, 7.into()), Err(LineOffsetError::EndOfInput));
    }

    #[test]
    fn append() {
        let s = "01234\nab`de\nqr`tu";
        let s1 = 0;
        let s2 = s.find('`').unwrap();
        let s3 = s.rfind('`').unwrap();
        let s4 = s.len();

        let mut tracker1 = tracker_from_str(&s[s1..s2]);
        let mut tracker2 = tracker_from_str(&s[s2..s3]);
        let tracker3 = tracker_from_str(&s[s3..s4]);

        assert_eq!(tracker1.num_lines(), 2);
        assert_eq!(tracker2.num_lines(), 2);
        assert_eq!(tracker3.num_lines(), 1);

        tracker2.append(&tracker3, (s3 - s2).into());
        tracker1.append(&tracker2, (s2 - s1).into());

        assert_eq!(tracker1.num_lines(), 3);
        assert_eq!(&s[9..10], "d");
        assert_eq!(
            tracker1.at(s, 9.into()).unwrap(),
            LineAndCharPosition::new(1, 3)
        );
        assert_eq!(&s[16..17], "u");
        assert_eq!(
            tracker1.at(s, 16.into()).unwrap(),
            LineAndCharPosition::new(2, 4)
        );
    }

    #[test]
    fn complex() {
        let s = "0123456789\n0123456789\n012345\n012345\n🤷\n\n";
        let tracker = tracker_from_str(s);

        assert_eq!(tracker.num_lines(), 7);

        // boundaries
        assert_eq!(
            tracker.at(s, 0.into()).unwrap(),
            LineAndCharPosition::new(0, 0)
        );
        assert_eq!(
            tracker.at(s, s.len().into()),
            Err(LineOffsetError::EndOfInput)
        );
        assert_eq!(
            tracker.at(s, (s.len() + 1).into()),
            Err(LineOffsetError::EndOfInput)
        );

        //lines
        let idx = s.find('2').unwrap();
        assert_eq!(&s[idx..idx + 1], "2");
        assert_eq!(
            tracker.at(s, idx.into()).unwrap(),
            LineAndCharPosition::new(0, 2)
        );

        let idx = 1 + idx + s[idx + 1..].find('2').unwrap();
        assert_eq!(&s[idx..idx + 1], "2");
        assert_eq!(
            tracker.at(s, idx.into()).unwrap(),
            LineAndCharPosition::new(1, 2)
        );

        let idx = 1 + idx + s[idx + 1..].find('2').unwrap();
        assert_eq!(&s[idx..idx + 1], "2");
        assert_eq!(
            tracker.at(s, idx.into()).unwrap(),
            LineAndCharPosition::new(2, 2)
        );

        let idx = 1 + idx + s[idx + 1..].find('2').unwrap();
        assert_eq!(&s[idx..idx + 1], "2");
        assert_eq!(
            tracker.at(s, idx.into()).unwrap(),
            LineAndCharPosition::new(3, 2)
        );

        let idx = s.find('🤷').unwrap();
        assert_eq!(&s[idx..idx + '🤷'.len_utf8()], "🤷");
        assert_eq!(
            tracker.at(s, idx.into()).unwrap(),
            LineAndCharPosition::new(4, 0)
        );
    }
}
