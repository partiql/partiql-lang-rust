//! Identifiers as written in a query, the canonical names a catalog resolves them to, and the
//! namespaces and paths those names live in.

use std::fmt;
use std::hash::{Hash, Hasher};

/// Whether `a` and `b` are equal ignoring case, by per-character simple case mapping.
///
/// The strings are compared character by character, without full Unicode case folding: two
/// characters are equal if they are the same, if their uppercase forms are the same, or if the
/// lowercase forms of their uppercase forms are the same. So `"Ä"` matches `"ä"`, and the
/// dotless `"ı"` matches `"I"`, but `"ß"` does not match `"SS"`. Characters are compared as
/// Unicode scalar values (a supplementary character is one character, not two UTF-16 code units),
/// using the case mappings of the Unicode version Rust's standard library implements.
#[must_use]
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    let mut a = a.chars();
    let mut b = b.chars();
    loop {
        match (a.next(), b.next()) {
            (None, None) => return true,
            (Some(c1), Some(c2)) => {
                if c1 == c2 {
                    continue;
                }
                let u1 = simple_upper(c1);
                let u2 = simple_upper(c2);
                if u1 == u2 || simple_lower(u1) == simple_lower(u2) {
                    continue;
                }
                return false;
            }
            _ => return false,
        }
    }
}

/// The simple (single-character) uppercase mapping. Rust's `char::to_uppercase` is the full
/// mapping; where that is several characters (e.g. `ß` to `SS`) the character is kept
/// unchanged.
fn simple_upper(c: char) -> char {
    let mut up = c.to_uppercase();
    match (up.next(), up.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

/// The simple (single-character) lowercase mapping. The only
/// character whose full lowercase mapping is several characters is U+0130 (LATIN CAPITAL LETTER
/// I WITH DOT ABOVE), whose single-character mapping is `i`.
fn simple_lower(c: char) -> char {
    if c == '\u{130}' {
        return 'i';
    }
    let mut low = c.to_lowercase();
    match (low.next(), low.next()) {
        (Some(l), None) => l,
        _ => c,
    }
}

/// One part of an [`Identifier`]: its text and whether it was written regular (unquoted) or
/// delimited (double-quoted).
///
/// Equality and hashing use only the text, not whether the part is regular: `a` and `"a"` are
/// equal.
#[derive(Debug, Clone, Eq)]
pub struct IdentPart {
    text: String,
    regular: bool,
}

impl IdentPart {
    /// A regular (unquoted) part, which matches ignoring case.
    pub fn regular(text: impl Into<String>) -> Self {
        IdentPart {
            text: text.into(),
            regular: true,
        }
    }

    /// A delimited (double-quoted) part, which matches exactly.
    pub fn delimited(text: impl Into<String>) -> Self {
        IdentPart {
            text: text.into(),
            regular: false,
        }
    }

    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn is_regular(&self) -> bool {
        self.regular
    }

    /// Whether this part matches `other`: ignoring case (see [`equals_ignore_case`]) if this part
    /// is regular, exactly if it is delimited.
    #[must_use]
    pub fn matches(&self, other: &str) -> bool {
        if self.regular {
            equals_ignore_case(&self.text, other)
        } else {
            self.text == other
        }
    }
}

impl PartialEq for IdentPart {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Hash for IdentPart {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

impl fmt::Display for IdentPart {
    /// A regular part as its text, a delimited part in double quotes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.regular {
            write!(f, "{}", self.text)
        } else {
            write!(f, "\"{}\"", self.text)
        }
    }
}

/// An identifier as written in a query: one or more [`IdentPart`]s, the last of which is the
/// identifier itself and the others its qualifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    qualifier: Vec<IdentPart>,
    identifier: IdentPart,
}

impl Identifier {
    /// The identifier `identifier` qualified by `qualifier`.
    #[must_use]
    pub fn new(qualifier: Vec<IdentPart>, identifier: IdentPart) -> Self {
        Identifier {
            qualifier,
            identifier,
        }
    }

    /// The identifier with the given parts, or `None` if there are none.
    pub fn of(parts: impl IntoIterator<Item = IdentPart>) -> Option<Self> {
        let mut qualifier: Vec<IdentPart> = parts.into_iter().collect();
        let identifier = qualifier.pop()?;
        Some(Identifier::new(qualifier, identifier))
    }

    /// A single regular part.
    pub fn regular(text: impl Into<String>) -> Self {
        Identifier::new(vec![], IdentPart::regular(text))
    }

    /// A single delimited part.
    pub fn delimited(text: impl Into<String>) -> Self {
        Identifier::new(vec![], IdentPart::delimited(text))
    }

    /// Regular parts, or `None` if there are none.
    pub fn regular_parts<S: Into<String>>(parts: impl IntoIterator<Item = S>) -> Option<Self> {
        Identifier::of(parts.into_iter().map(IdentPart::regular))
    }

    /// Delimited parts, or `None` if there are none.
    pub fn delimited_parts<S: Into<String>>(parts: impl IntoIterator<Item = S>) -> Option<Self> {
        Identifier::of(parts.into_iter().map(IdentPart::delimited))
    }

    /// The last part.
    #[must_use]
    pub fn identifier(&self) -> &IdentPart {
        &self.identifier
    }

    /// The parts before the last.
    #[must_use]
    pub fn qualifier(&self) -> &[IdentPart] {
        &self.qualifier
    }

    #[must_use]
    pub fn has_qualifier(&self) -> bool {
        !self.qualifier.is_empty()
    }

    /// All the parts, in order.
    pub fn parts(&self) -> impl Iterator<Item = &IdentPart> {
        self.qualifier
            .iter()
            .chain(std::iter::once(&self.identifier))
    }

    /// The number of parts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.qualifier.len() + 1
    }

    /// Always `false`: an identifier has at least one part.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// This identifier followed by the parts of `other`.
    #[must_use]
    pub fn append(&self, other: &Identifier) -> Identifier {
        let parts: Vec<IdentPart> = self.parts().chain(other.parts()).cloned().collect();
        Identifier::of(parts).expect("an identifier has at least one part")
    }

    /// Whether this unqualified identifier matches `other`: exactly, or, if `ignore_case`,
    /// as its part matches (see [`IdentPart::matches`]). A qualified identifier matches nothing.
    #[must_use]
    pub fn matches(&self, other: &str, ignore_case: bool) -> bool {
        if self.has_qualifier() {
            return false;
        }
        if ignore_case {
            self.identifier.matches(other)
        } else {
            self.identifier.text == other
        }
    }
}

impl fmt::Display for Identifier {
    /// The parts joined with `.`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in &self.qualifier {
            write!(f, "{part}.")?;
        }
        write!(f, "{}", self.identifier)
    }
}

/// A list of names that qualify a [`Name`], e.g. a schema in a catalog.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Namespace {
    levels: Vec<String>,
}

impl Namespace {
    #[must_use]
    pub fn empty() -> Self {
        Namespace::default()
    }

    pub fn of<S: Into<String>>(levels: impl IntoIterator<Item = S>) -> Self {
        Namespace {
            levels: levels.into_iter().map(Into::into).collect(),
        }
    }

    #[must_use]
    pub fn levels(&self) -> &[String] {
        &self.levels
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.levels.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    /// This namespace followed by `levels`.
    #[must_use]
    pub fn append<S: Into<String>>(&self, levels: impl IntoIterator<Item = S>) -> Self {
        let mut out = self.clone();
        out.levels.extend(levels.into_iter().map(Into::into));
        out
    }

    /// The levels as delimited identifier parts, or `None` for the empty namespace.
    #[must_use]
    pub fn as_identifier(&self) -> Option<Identifier> {
        Identifier::delimited_parts(self.levels.iter().cloned())
    }
}

impl fmt::Display for Namespace {
    /// Empty for the empty namespace, otherwise the levels as delimited parts joined with `.`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_identifier() {
            Some(id) => write!(f, "{id}"),
            None => Ok(()),
        }
    }
}

/// The canonical, exact name of a catalog object: a [`Namespace`] and a name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    namespace: Namespace,
    name: String,
}

impl Name {
    #[must_use]
    pub fn new(namespace: Namespace, name: impl Into<String>) -> Self {
        Name {
            namespace,
            name: name.into(),
        }
    }

    /// The name with the given parts, the last of which is the name and the others its
    /// namespace, or `None` if there are none.
    pub fn of<S: Into<String>>(parts: impl IntoIterator<Item = S>) -> Option<Self> {
        let mut levels: Vec<String> = parts.into_iter().map(Into::into).collect();
        let name = levels.pop()?;
        Some(Name::new(Namespace { levels }, name))
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    #[must_use]
    pub fn has_namespace(&self) -> bool {
        !self.namespace.is_empty()
    }

    /// The namespace levels followed by the name.
    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.namespace
            .levels
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.name.as_str()))
    }

    /// The number of parts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.namespace.len() + 1
    }

    /// Always `false`: a name has at least one part.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }
}

impl fmt::Display for Name {
    /// The parts as delimited identifier parts joined with `.`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for level in &self.namespace.levels {
            write!(f, "{}.", IdentPart::delimited(level.as_str()))?;
        }
        write!(f, "{}", IdentPart::delimited(self.name.as_str()))
    }
}

/// The ordered list of namespaces that unqualified routine names are searched in.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Path {
    namespaces: Vec<Namespace>,
}

/// The error of [`Path::promote`] for a namespace that is not in the path.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Cannot promote a path entry that is not present: {0}")]
pub struct PathEntryNotPresent(Namespace);

impl PathEntryNotPresent {
    /// The namespace that is not in the path.
    #[must_use]
    pub fn entry(&self) -> &Namespace {
        &self.0
    }
}

impl Path {
    pub fn of(namespaces: impl IntoIterator<Item = Namespace>) -> Self {
        Path {
            namespaces: namespaces.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn namespaces(&self) -> &[Namespace] {
        &self.namespaces
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.namespaces.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.namespaces.is_empty()
    }

    #[must_use]
    pub fn contains(&self, entry: &Namespace) -> bool {
        self.namespaces.contains(entry)
    }

    /// This path with the first occurrence of `entry` moved to the front.
    pub fn promote(&self, entry: &Namespace) -> Result<Path, PathEntryNotPresent> {
        let index = self
            .namespaces
            .iter()
            .position(|ns| ns == entry)
            .ok_or_else(|| PathEntryNotPresent(entry.clone()))?;
        let mut namespaces = self.namespaces.clone();
        let entry = namespaces.remove(index);
        namespaces.insert(0, entry);
        Ok(Path { namespaces })
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PATH: (")?;
        for (i, ns) in self.namespaces.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{ns}")?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_mappings_are_single_character() {
        assert_eq!(simple_upper('a'), 'A');
        assert_eq!(simple_upper('\u{DF}'), '\u{DF}'); // ß has no single-character uppercase
        assert_eq!(simple_lower('\u{130}'), 'i');
        assert_eq!(simple_lower('A'), 'a');
    }
}
