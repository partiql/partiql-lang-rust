// Copyright Amazon.com, Inc. or its affiliates.

//! Types representing positions, spans, locations, etc of parsed PartiQL text.

use std::fmt;
use std::fmt::{Debug, Formatter};
use std::num::NonZeroUsize;
use std::ops::{Add, Range, Sub};

macro_rules! impl_pos {
    ($pos_type:ident, $primitive:ty) => {
        impl Add for $pos_type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
        impl Add<$primitive> for $pos_type {
            type Output = Self;

            fn add(self, rhs: $primitive) -> Self::Output {
                Self(self.0 + rhs)
            }
        }
        impl Sub for $pos_type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }
        impl Sub<$primitive> for $pos_type {
            type Output = Self;

            fn sub(self, rhs: $primitive) -> Self::Output {
                Self(self.0 - rhs)
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

/// A 0-indexed byte offset, relative to some other position.
///
/// This type is small (u32 currently) to allow it to be included in ASTs and other
/// data structures.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct ByteOffset(pub u32);
impl_pos!(ByteOffset, u32);

/// A 0-indexed line offset, relative to some other position.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LineOffset(pub u32);
impl_pos!(LineOffset, u32);

/// A 0-indexed char offset, relative to some other position.
///
/// This value represents the number of unicode codepoints seen, so will differ
/// from [`ByteOffset`] for a given location in a &str if the string contains
/// non-ASCII unicode characters
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct CharOffset(pub u32);
impl_pos!(CharOffset, u32);

/// A 0-indexed byte absolute position (i.e., relative to the start of a &str)
///
/// This type is small (u16 currently) to allow it to be included in ASTs and other
/// data structures.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct BytePosition(pub ByteOffset);

impl From<ByteOffset> for BytePosition {
    fn from(offset: ByteOffset) -> Self {
        Self(offset)
    }
}

impl From<usize> for BytePosition {
    fn from(offset: usize) -> Self {
        Self(offset.into())
    }
}

/// A 0-indexed line & char absolute position (i.e., relative to the start of a &str)
///
/// ## Example
/// ```
/// # use partiql_source_map::location::LineAndCharPosition;
/// println!("Beginning of &str: {:?}", LineAndCharPosition::new(0, 0));
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LineAndCharPosition {
    pub line: LineOffset,
    pub char: CharOffset,
}
impl LineAndCharPosition {
    /// Constructs at [`LineAndCharPosition`]
    #[inline]
    pub fn new(line: usize, char: usize) -> Self {
        Self {
            line: LineOffset::from_usize(line),
            char: CharOffset::from_usize(char),
        }
    }
}

/// A line and column location intended for usage in errors/warnings/lints/etc.
///
/// Both line and column are 1-indexed, as that is how most people think of lines and columns.
///
/// ## Example
/// ```
/// # use partiql_source_map::location::LineAndColumn;
/// println!("Beginning of a document: {}", LineAndColumn::new(1, 1).unwrap());
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct LineAndColumn {
    pub line: NonZeroUsize,
    pub column: NonZeroUsize,
}

impl LineAndColumn {
    /// Constructs at [`LineAndColumn`] if non-zero-index invariants, else [`None`]
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

impl From<LineAndCharPosition> for LineAndColumn {
    fn from(LineAndCharPosition { line, char }: LineAndCharPosition) -> Self {
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

/// A wrapper type that holds an `inner` value and a `location` for it
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Located<T, Loc: Debug> {
    // TODO this should be `Loc: Display`
    /// The item that has a location attached
    pub inner: T,
    /// The location of the error
    pub location: Range<Loc>,
}

/// Trait adding a `to_located` method to ease construction of [`Located`] from its inner value.
///
/// ## Example
///
/// ```rust
/// use partiql_source_map::location::{Located, ToLocated};
/// assert_eq!("blah".to_string().to_located(5..10),
///             Located{inner: "blah".to_string(), location: 5..10});
/// ```
pub trait ToLocated<Loc: Debug>: Sized {
    /// Create a [`Located`] from its inner value.
    fn to_located(self, location: Range<Loc>) -> Located<Self, Loc> {
        let inner = self;
        Located { inner, location }
    }
}

// "Blanket" impl of `ToLocated` for all `T`
// See https://doc.rust-lang.org/book/ch10-02-traits.html#using-trait-bounds-to-conditionally-implement-methods
impl<T, Loc: Debug> ToLocated<Loc> for T {}

impl<T, Loc: Debug> Located<T, Loc> {
    /// Maps an `Located<T, Loc>` to `Located<T, Loc2>` by applying a function to the contained
    /// location and moving the contained `inner`
    ///
    /// ## Example
    ///
    /// ```rust
    /// use partiql_source_map::location::{Located, ToLocated};
    /// assert_eq!("blah".to_string().to_located(5..10).map_loc(|l| l+5),
    ///             Located{inner: "blah".to_string(), location: 10..15});
    /// ```
    pub fn map_loc<F, Loc2>(self, tx: F) -> Located<T, Loc2>
    where
        Loc2: Debug,
        F: Fn(Loc) -> Loc2,
    {
        let Located { inner, location } = self;
        let location = Range {
            start: tx(location.start),
            end: tx(location.end),
        };
        Located { inner, location }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    use crate::location::Located;
    use crate::location::{ByteOffset, BytePosition};

    #[test]
    fn located() {
        let l1: Located<String, ByteOffset> =
            "test".to_string().to_located(ByteOffset(0)..ByteOffset(42));

        assert_eq!(l1.inner, "test");
        assert_eq!(l1.location.start.0, 0);
        assert_eq!(l1.location.end.0, 42);

        let l1c = l1.clone();
        assert!(matches!(
            l1c,
            Located {
                inner: s,
                location: std::ops::Range {
                    start:ByteOffset(0),
                    end: ByteOffset(42)
                }
            } if s == "test"
        ));

        let l2 = l1.map_loc(BytePosition);

        assert!(matches!(
            l2.location,
            std::ops::Range {
                start: BytePosition(ByteOffset(0)),
                end: BytePosition(ByteOffset(42))
            }
        ));
    }

    #[test]
    fn byteoff() {
        let offset1 = ByteOffset(5);
        let offset2 = ByteOffset::from_usize(15);

        assert_eq!(20, (offset1 + offset2).to_usize());
        assert_eq!(10, (offset2 - offset1).to_usize());
        assert_eq!(ByteOffset(10), offset2 - 5);
        assert_eq!(ByteOffset(20), offset2 + 5);
    }

    #[test]
    fn lineoff() {
        let offset1 = LineOffset(5);
        let offset2 = LineOffset::from_usize(15);

        assert_eq!(20, (offset1 + offset2).to_usize());
        assert_eq!(10, (offset2 - offset1).to_usize());
        assert_eq!(LineOffset(10), offset2 - 5);
        assert_eq!(LineOffset(20), offset2 + 5);
    }

    #[test]
    fn charoff() {
        let offset1 = CharOffset(5);
        let offset2 = CharOffset::from_usize(15);

        assert_eq!(20, (offset1 + offset2).to_usize());
        assert_eq!(10, (offset2 - offset1).to_usize());
        assert_eq!(CharOffset(10), offset2 - 5);
        assert_eq!(CharOffset(20), offset2 + 5);
    }

    #[test]
    fn positions() {
        assert_eq!(BytePosition(ByteOffset(15)), BytePosition(15.into()));
        assert_eq!(BytePosition(ByteOffset(5)), ByteOffset(5).into());
        assert_eq!(BytePosition(ByteOffset(25)), 25.into());

        let loc = LineAndCharPosition::new(13, 42);
        assert_eq!(
            LineAndCharPosition {
                line: LineOffset(13),
                char: CharOffset(42)
            },
            loc
        );
        let display = LineAndColumn {
            line: unsafe { NonZeroUsize::new_unchecked(14) },
            column: unsafe { NonZeroUsize::new_unchecked(43) },
        };

        assert_eq!(display, loc.into());
        assert_eq!(display, unsafe { LineAndColumn::new_unchecked(14, 43) });
        assert_eq!(display, LineAndColumn::new(14, 43).unwrap());
        assert_eq!("line 14, column 43", format!("{}", display))
    }
}
