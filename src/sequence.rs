use std::ops::Deref;

use crate::interner::{Interned, InternedString, Interner};

#[derive(Debug)]
pub struct Sequence<'a> {
    name: Option<String>,
    items: Vec<InternedString<'a>>,
}

/// Convert an iterable of stringy things into a vector of interned strings.
///
/// This will prove useful for testing.
fn make_interned<'a, Items, Item>(
    interner: &'a Interner<String>,
    items: Items,
) -> Result<Vec<InternedString<'a>>, Error>
where
    Items: IntoIterator<Item = Item>,
    Item: AsRef<str>,
{
    let mut interneds = Vec::new();
    for item in items.into_iter() {
        let item = item.as_ref();
        let interned = interner.get(item).ok_or(Error::NotFound(item.to_owned()))?;
        interneds.push(interned);
    }
    Ok(interneds)
}

impl<'a> Sequence<'a> {
    fn new<Items, Item>(interner: &'a Interner<String>, items: Items) -> Result<Self, Error>
    where
        Items: IntoIterator<Item = Item>,
        Item: AsRef<str>,
    {
        let items = make_interned(interner, items)?;
        Ok(Self { name: None, items })
    }

    /// `true` when this sequence matches some subset if the iterable.
    pub fn is_matched(&self, iter: impl IntoIterator<Item = InternedString<'a>>) -> bool {
        // the basic strategy here is to construct a vector of booleans.
        // each bool in the vec is true if and only if this sequence matches at that offset.
        //
        // example: our desired sequence is 1A 2B 1A 3C
        // iterator items are 1A 2B 3C 1A 2B 1A 3C
        //
        // item 0: 1A. Index 0 - sequence 0 means we check our sequence at position 0. Match, so we push `true`
        // Item 1: 2B. First, do index 1 - sequence 1, so check sequence at position 0. No match, so push `false`.
        //              Next, do checks for sequence in 0..(n-1):
        //                  index 1 - sequence 0: match && true = true
        // Item 2: 3C. First, do index 2 - sequence 2, so check sequence at position 0. No match, so push `false`.
        //              Next, do checks for sequence in 0..(n-1):
        //                  index 2 - sequence 0: no match (we want 1A again here); true && false = false
        //                  index 2 - sequence 1: no match (we want 2B here); false && false = false
        // etc...
        let mut sequences = Vec::new();
        let mut item_count = 0;
        for (index, item) in iter.into_iter().enumerate() {
            item_count += 1;

            sequences.push(self.items[0] == item);

            // now do the updates
            // TODO: there is a panic here, the math is not right
            for sequence in 0..(sequences.len() - 1) {
                sequences[sequence] = sequences[sequence] && self.items[index - sequence] == item;
            }
        }

        dbg!(&sequences);

        // we're not quite done: what if there was a partial match, but they didn't provide enough input?
        let desired_length = dbg!(dbg!(item_count) - dbg!(self.items.len()) + 1);
        sequences.truncate(desired_length);

        dbg!(&sequences);

        // if any subseqence was true at this point, we've succeeded
        sequences.into_iter().any(std::convert::identity)
    }
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("item not found hwen constructing sequence: \"{0}\"")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interner() -> Interner<String> {
        let mut interner = Interner::new();
        interner.insert("1A".into());
        interner.insert("2B".into());
        interner.insert("3C".into());
        interner
    }

    #[test]
    fn inner_example_bare() {
        let interner = &make_interner();
        let sequence = Sequence::new(&interner, "1A 2B 1A 3C".split_ascii_whitespace()).unwrap();
        let items =
            make_interned(&interner, "1A 2B 3C 1A 2B 1A 3C".split_ascii_whitespace()).unwrap();
        assert!(sequence.is_matched(items));
    }

    #[test]
    fn inner_example_incomplete() {
        let interner = &make_interner();
        let sequence = Sequence::new(&interner, "1A 2B 1A 3C".split_ascii_whitespace()).unwrap();
        let items = make_interned(&interner, "1A 2B 3C 1A 2B 1A".split_ascii_whitespace()).unwrap();
        assert!(!sequence.is_matched(items));
    }

    #[test]
    fn inner_example_too_long() {
        let interner = &make_interner();
        let sequence = Sequence::new(&interner, "1A 2B 1A 3C".split_ascii_whitespace()).unwrap();
        let items = make_interned(
            &interner,
            "1A 2B 3C 1A 2B 1A 3C 2B 3C 1A".split_ascii_whitespace(),
        )
        .unwrap();
        assert!(sequence.is_matched(items));
    }
}
