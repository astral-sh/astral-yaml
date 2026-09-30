use std::{collections::HashSet, ops::Range};

use crate::{Error, de};

#[derive(Default)]
pub(super) struct Keys(HashSet<String>);

impl Keys {
    pub(super) fn insert(&mut self, output: &str, range: Range<usize>) -> Result<(), Error> {
        let source = &output[range];
        let identity = if source.starts_with('"') {
            // Quoted strings already have canonical spelling and need no parsing limit.
            source.to_owned()
        } else {
            de::canonical_key(source).map_err(|error| Error::new(error.message))?
        };

        if !self.0.insert(identity) {
            return Err(Error::new("duplicate mapping key"));
        }

        Ok(())
    }
}
