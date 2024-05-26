use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

use indexmap::IndexSet;

use super::{Device, Range};
use crate::Word;

#[derive(Debug, Default)]
pub(super) struct Map(BTreeMap<Word, IndexSet<Entry>>);

impl Map {
    /// Constructs a new `Map`.
    #[allow(unused)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears the map, removing all elements.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Map the device to a given range.
    pub fn map(&mut self, range: Range, entry: Device) {
        let new = Entry::new(range, entry);
        let set = self.0.entry(new.base()).or_default();
        set.insert(new);
        set.sort();
    }

    /// Unmaps the device across all ranges.
    pub fn unmap(&mut self, entry: &Device) -> bool {
        let mut unmapped = false;
        for set in self.0.values_mut() {
            let len = set.len();
            set.retain(|it| !Rc::ptr_eq(&it.entry, entry));
            unmapped |= len > set.len();
        }
        unmapped
    }

    /// Select all devices for a given address.
    pub fn select(&self, addr: Word) -> impl Iterator<Item = &Entry> {
        self.0
            .range(..=addr)
            .rev()
            .flat_map(|(_, set)| set.iter())
            .filter(move |it| it.contains(&addr))
    }

    /// Find the entry for a given device.
    #[allow(unused)]
    pub fn find(&self, entry: &Device) -> Option<&Entry> {
        self.0
            .iter()
            .flat_map(|(_, set)| set.iter())
            .find(|it| Rc::ptr_eq(&it.entry, entry))
    }

    /// Gets an iterator over the entries of the map.
    #[allow(unused)]
    pub fn iter(&self) -> impl Iterator + '_ {
        self.0.iter().flat_map(|(_, set)| set.iter())
    }
}

#[derive(Clone, Debug)]
pub(super) struct Entry {
    pub range: Range,
    pub entry: Device,
}

impl Entry {
    fn new(range: Range, entry: Device) -> Self {
        Self { range, entry }
    }

    pub fn base(&self) -> Word {
        *self.range.start()
    }

    fn span(&self) -> Word {
        *self.range.end() - *self.range.start()
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn contains(&self, addr: &Word) -> bool {
        self.range.contains(addr)
    }
}

impl Eq for Entry {}

impl Hash for Entry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.range.hash(state);
        self.entry.as_ptr().hash(state);
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.base(), self.span()).cmp(&(other.base(), other.span()))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.entry, &other.entry)
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
