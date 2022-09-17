use crate::interner::{InternedString, Interner};

#[derive(Debug)]
pub struct Sequence<'a> {
    pub name: Option<String>,
    items: Vec<InternedString<'a>>,
}

/// Convert an iterable of stringy things into a vector of interned strings.
pub(crate) fn make_interned<Items, Item>(
    interner: &Interner<String>,
    items: Items,
) -> Result<Vec<InternedString>, Error>
where
    Items: IntoIterator<Item = Item>,
    Item: AsRef<str>,
{
    let mut interneds = Vec::new();
    for item in items.into_iter() {
        let item = item.as_ref();
        let interned = interner
            .get(item)
            .ok_or_else(|| Error::NotFound(item.to_owned()))?;
        interneds.push(interned);
    }
    Ok(interneds)
}

impl<'a> Sequence<'a> {
    pub fn new<Items, Item>(interner: &'a Interner<String>, items: Items) -> Result<Self, Error>
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
        // item 0: 1A. Index 0 - offset 0 means we check our sequence at position 0. Match, so we push `true`
        // Item 1: 2B. First, do index 1 - offset 1, so check sequence at position 0. No match, so push `false`.
        //              Next, do checks for offset in 0..(n-1):
        //                  index 1 - offset 0: match && true = true
        // Item 2: 3C. First, do index 2 - offset 2, so check sequence at position 0. No match, so push `false`.
        //              Next, do checks for offset in 0..(n-1):
        //                  index 2 - offset 0: no match (we want 1A again here); true && false = false
        //                  index 2 - offset 1: no match (we want 2B here); false && false = false
        // etc...
        let mut sequences = Vec::new();
        let mut item_count = 0;
        'outer: for (index, item) in iter.into_iter().enumerate() {
            item_count += 1;

            sequences.push(self.items[0] == item);

            // now do the updates
            for offset in 0..(sequences.len() - 1) {
                let check_idx = match index.checked_sub(offset) {
                    Some(idx) => idx,
                    None => continue 'outer,
                };
                if check_idx >= self.items.len() {
                    continue 'outer;
                }
                sequences[offset] = sequences[offset] && self.items[check_idx] == item;
            }
        }

        // we're not quite done: what if there was a partial match at the end,
        // but they didn't provide enough input?
        let desired_length = item_count - self.items.len() + 1;
        sequences.truncate(desired_length);

        // if any subseqence was true at this point, we've succeeded
        sequences.into_iter().any(std::convert::identity)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("item not found when constructing sequence: \"{0}\"")]
    NotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn make_interner() -> Interner<String> {
        let mut interner = Interner::new();
        interner.insert("1A".into());
        interner.insert("2B".into());
        interner.insert("3C".into());
        interner
    }

    #[rstest]
    #[case::bare("1A 2B 1A 3C", "1A 2B 3C 1A 2B 1A 3C", true)]
    #[case::incomplete("1A 2B 1A 3C", "1A 2B 3C 1A 2B 1A", false)]
    #[case::extra("1A 2B 1A 3C", "1A 2B 3C 1A 2B 1A 3C 2B 3C 1A", true)]
    #[case::exact("1A 2B", "1A 2B", true)]
    #[case::backwards("1A 2B 3C", "3C 2B 1A", false)]
    fn inner_example(#[case] sequence: &str, #[case] items: &str, #[case] expect_match: bool) {
        let interner = make_interner();
        let sequence = Sequence::new(&interner, sequence.split_ascii_whitespace()).unwrap();
        let items = make_interned(&interner, items.split_ascii_whitespace()).unwrap();
        assert_eq!(sequence.is_matched(items), expect_match);
    }
}
