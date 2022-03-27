// Copyright Amazon.com, Inc. or its affiliates.

//! Types representing positions, spans, locations, etc of parsed PartiQL text.

use std::fmt;
use std::fmt::Formatter;
use std::num::NonZeroUsize;
use std::ops::{Add, Sub};

macro_rules! impl_pos {
    ($pos_type:ident, $primitive:ty) => {
        impl Add for $pos_type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
        impl Sub for $pos_type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }
        impl $pos_type {
            #[inline(always)]
            pub fn from_usize(n: usize) -> Self {
                Self(n as $primitive)
            }

            #[inline(always)]
            pub fn to_usize(&self) -> usize {
                self.0 as usize
            }
        }
        impl From<usize> for $pos_type {
            fn from(n: usize) -> Self {
                Self::from_usize(n)
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct BytePos(pub u16);
impl_pos!(BytePos, u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LinePos(pub u16);
impl_pos!(LinePos, u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct CharPos(pub u16);
impl_pos!(CharPos, u16);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ByteOffset(pub BytePos);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LineAndColOffset {
    pub line: LinePos,
    pub char: CharPos,
}
impl LineAndColOffset {
    /// Constructs at [`LineAndColOffset`]
    #[inline]
    pub fn new(line: usize, char: usize) -> Self {
        Self {
            line: LinePos::from_usize(line),
            char: CharPos::from_usize(char),
        }
    }
}

/// A line and column location.
///
/// This value is one-based, as that is how most people think of lines and columns.
///
/// ## Example
/// ```
/// # use partiql_parser::location::LineAndColumn;
/// println!("Beginning of a document: {}", LineAndColumn::new(1, 1).unwrap());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LineAndColumn {
    pub line: NonZeroUsize,
    pub column: NonZeroUsize,
}

impl LineAndColumn {
    /// Constructs at [`LineAndColumn`], verifying 1-position invariant.
    #[inline]
    pub fn new(line: usize, column: usize) -> Option<Self> {
        Some(Self {
            line: NonZeroUsize::new(line)?,
            column: NonZeroUsize::new(column)?,
        })
    }

    /// Constructs at [`LineAndColumn`] without verifying 1-indexed invariant (i.e. nonzero).
    /// This results in undefined behaviour if either `line` or `column` is zero.
    ///
    /// # Safety
    ///
    /// Both `line` and `column` values must not be zero.
    #[inline]
    pub const unsafe fn new_unchecked(line: usize, column: usize) -> Self {
        Self {
            line: NonZeroUsize::new_unchecked(line),
            column: NonZeroUsize::new_unchecked(column),
        }
    }
}

impl From<LineAndColOffset> for LineAndColumn {
    fn from(LineAndColOffset { line, char }: LineAndColOffset) -> Self {
        let line = line.to_usize() + 1;
        let column = char.to_usize() + 1;
        // SAFETY: +1 is added to each of line and char after upcasting from a smaller integer
        unsafe { LineAndColumn::new_unchecked(line, column) }
    }
}

impl fmt::Display for LineAndColumn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// A possible position in the source.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Position {
    /// Variant indicating that there *is no* known location in source for some context.
    Unknown,
    /// Variant indicating that there *is* a known location in source for some context.
    At(LineAndColumn),
}

impl From<Option<LineAndColumn>> for Position {
    fn from(loc: Option<LineAndColumn>) -> Self {
        loc.map_or_else(|| Position::Unknown, LineAndColumn::into)
    }
}

impl From<LineAndColumn> for Position {
    fn from(loc: LineAndColumn) -> Self {
        Position::At(loc)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Position::Unknown => write!(f, "unknown position"),
            Position::At(location) => {
                write!(f, "{}", location)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn bytepos() {
        let bp1 = BytePos(5);
        let bp2 = BytePos::from_usize(15);

        assert_eq!(20, (bp1 + bp2).to_usize());
        assert_eq!(BytePos(10), bp2 - 5.into());
    }

    #[test]
    fn linepos() {
        let lp1 = LinePos(5);
        let lp2 = LinePos::from_usize(15);

        assert_eq!(20, (lp1 + lp2).to_usize());
        assert_eq!(LinePos(10), lp2 - 5.into());
    }

    #[test]
    fn charpos() {
        let cp1 = CharPos(5);
        let cp2 = CharPos::from_usize(15);

        assert_eq!(20, (cp1 + cp2).to_usize());
        assert_eq!(CharPos(10), cp2 - 5.into());
    }

    #[test]
    fn offset() {
        assert_eq!(ByteOffset(BytePos(5)), ByteOffset(5.into()));

        let loc = LineAndColOffset::new(13, 42);
        assert_eq!(
            LineAndColOffset {
                line: LinePos(13),
                char: CharPos(42)
            },
            loc
        );
        let display = LineAndColumn {
            line: unsafe { NonZeroUsize::new_unchecked(14) },
            column: unsafe { NonZeroUsize::new_unchecked(43) },
        };

        assert_eq!(display, loc.into());
        assert_eq!(display, unsafe { LineAndColumn::new_unchecked(14, 43) });
        assert_eq!("line 14, column 43", format!("{}", display))
    }

    #[test]
    fn position() {
        assert_eq!(Position::Unknown, None.into());
        assert_eq!(Position::Unknown, LineAndColumn::new(0, 0).into());
        let lac = LineAndColumn {
            line: unsafe { NonZeroUsize::new_unchecked(5) },
            column: unsafe { NonZeroUsize::new_unchecked(6) },
        };
        assert_eq!(Position::At(lac), LineAndColumn::new(5, 6).into());
        assert_eq!("unknown position", format!("{}", Position::Unknown));
        assert_eq!(
            "line 4, column 5",
            format!("{}", Position::At(LineAndColOffset::new(3, 4).into()))
        );
    }
}
