use std::{num::NonZeroUsize, ops::Range};

use crate::{Error, de};

const LINEAR_KEYS: usize = 32;

/// Key storage shared by active mappings. Completed mappings release their
/// entries while retaining capacity for the next mapping.
#[derive(Default)]
pub(super) struct Keys {
    entries: Vec<Key>,
    canonical: Vec<Canonical>,
}

/// Duplicate checks are unnecessary when every key clears a different bit. A collision
/// only requests an exact comparison. It never establishes equality.
#[derive(Clone, Copy)]
pub(super) struct Scope {
    start: usize,
    // A collision clears the entire mask, keeping exact comparisons enabled
    // for the rest of the mapping without a separate flag.
    available: u64,
}

impl Scope {
    pub(super) fn insert(&mut self, signature: u64) {
        // Mix the prefix and length before selecting one of 64 bits.
        let hash = signature ^ (signature >> 32);
        let bit = 1 << (hash.wrapping_mul(0x9e37_79b9_7f4a_7c15) >> 58);
        self.available = if self.available & bit == 0 {
            0
        } else {
            self.available & !bit
        };
    }
}

#[derive(Clone, Copy)]
struct Key {
    identity: Identity,
    signature: u64,
}

#[derive(Clone, Copy)]
enum Identity {
    // Quoted keys are nonempty. The nonzero end lets Rust store both variants
    // in two words without a separate discriminant.
    Output { start: usize, end: NonZeroUsize },
    Canonical(usize),
}

struct Canonical {
    entry: usize,
    identity: String,
}

impl Key {
    #[inline]
    fn new(identity: Identity, bytes: &[u8]) -> Self {
        // For most quoted keys, skipping the opening quote avoids reading
        // across separately written bytes.
        let prefix = bytes
            .get(1..)
            .and_then(<[u8]>::first_chunk)
            .or_else(|| bytes.first_chunk())
            .copied()
            .map_or(0, u32::from_ne_bytes);
        // This only filters candidates: equal signatures still compare every byte.
        let signature = (u64::from(prefix) << 32) ^ bytes.len() as u64;
        Self {
            identity,
            signature,
        }
    }

    fn identity<'a>(&self, output: &'a str, canonical: &'a [Canonical]) -> &'a [u8] {
        match &self.identity {
            Identity::Output { start, end } => &output.as_bytes()[*start..end.get()],
            Identity::Canonical(index) => canonical[*index].identity.as_bytes(),
        }
    }
}

impl Keys {
    pub(super) fn scope(&self) -> Scope {
        Scope {
            start: self.entries.len(),
            available: u64::MAX,
        }
    }

    #[inline]
    pub(super) fn insert(&mut self, output: &str, range: Range<usize>) -> Result<u64, Error> {
        let source = &output.as_bytes()[range.clone()];
        if let (Some(b'"'), Some(end)) = (source.first(), NonZeroUsize::new(range.end)) {
            let identity = Identity::Output {
                start: range.start,
                end,
            };
            let key = Key::new(identity, source);
            self.push(key);
            Ok(key.signature)
        } else {
            self.insert_canonical(&output[range])
        }
    }

    #[cold]
    fn insert_canonical(&mut self, source: &str) -> Result<u64, Error> {
        let identity = de::canonical_key(source).map_err(|error| Error::new(error.message))?;
        let key = Key::new(
            Identity::Canonical(self.canonical.len()),
            identity.as_bytes(),
        );
        self.canonical.push(Canonical {
            entry: self.entries.len(),
            identity,
        });
        self.push(key);
        Ok(key.signature)
    }

    #[inline]
    fn push(&mut self, key: Key) {
        if self.entries.capacity() == 0 {
            self.entries.reserve(32);
        }
        self.entries.push(key);
    }

    pub(super) fn finish(&mut self, output: &str, scope: Scope) -> Result<(), Error> {
        let start = scope.start;
        if self.entries.len() == start {
            return Ok(());
        }

        let duplicate = if scope.available == 0 {
            let entries = &mut self.entries[start..];
            Self::duplicates(entries, output, &self.canonical)
        } else {
            false
        };

        self.entries.truncate(start);
        while self.canonical.last().is_some_and(|key| key.entry >= start) {
            self.canonical.pop();
        }

        if duplicate {
            return Err(Error::new("duplicate mapping key"));
        }

        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn duplicates(entries: &mut [Key], output: &str, canonical: &[Canonical]) -> bool {
        if entries.len() <= LINEAR_KEYS {
            Self::duplicates_in_small_mapping(entries, output, canonical)
        } else {
            Self::duplicates_in_large_mapping(entries, output, canonical)
        }
    }

    #[inline]
    fn duplicates_in_small_mapping(
        mut entries: &[Key],
        output: &str,
        canonical: &[Canonical],
    ) -> bool {
        while let Some((key, rest)) = entries.split_first() {
            if rest.iter().any(|previous| {
                previous.signature == key.signature
                    && previous.identity(output, canonical) == key.identity(output, canonical)
            }) {
                return true;
            }

            entries = rest;
        }

        false
    }

    #[cold]
    fn duplicates_in_large_mapping(
        entries: &mut [Key],
        output: &str,
        canonical: &[Canonical],
    ) -> bool {
        // Sorting bounds the work even when many keys share a signature.
        entries.sort_unstable_by(|left, right| {
            left.signature.cmp(&right.signature).then_with(|| {
                left.identity(output, canonical)
                    .cmp(right.identity(output, canonical))
            })
        });
        entries.windows(2).any(|pair| {
            pair[0].signature == pair[1].signature
                && pair[0].identity(output, canonical) == pair[1].identity(output, canonical)
        })
    }
}
