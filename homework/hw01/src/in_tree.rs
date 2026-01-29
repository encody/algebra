use std::{collections::HashMap, hash::Hash};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Entry<T> {
    pub parent_index: usize,
    pub len: usize,
    pub value: T,
}

#[derive(Debug)]
pub struct InTree<T> {
    pub lookup: HashMap<Entry<T>, usize>,
    pub entries: Vec<Option<Entry<T>>>,
}

impl<T> Default for InTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InTree<T> {
    pub fn new() -> Self {
        Self {
            lookup: HashMap::new(),
            entries: vec![None],
        }
    }

    pub fn create(&mut self, parent_index: usize, value: T) -> usize
    where
        T: Clone + Hash + Eq,
    {
        let len = match &self.entries[parent_index] {
            Some(p) => p.len + 1,
            None => 1,
        };
        let e = Entry {
            parent_index,
            value,
            len,
        };

        if let Some(i) = self.lookup.get(&e) {
            *i
        } else {
            let i = self.entries.len();
            self.lookup.insert(e.clone(), i);
            self.entries.push(Some(e.clone()));
            i
        }
    }

    pub fn len(&self, index: usize) -> usize {
        match &self.entries[index] {
            Some(e) => e.len,
            None => 0,
        }
    }

    pub fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.entries[index].as_ref()
    }

    pub fn print(&self, index: usize)
    where
        T: std::fmt::Debug,
    {
        let mut entries = vec![];

        let mut current = &self.entries[index];

        while let Some(entry) = current {
            entries.push(&entry.value);

            current = &self.entries[entry.parent_index];
        }

        for e in entries.iter().rev() {
            eprintln!("{e:?}");
        }
    }

    pub fn traverse(&self, index: usize, nth_in_path: usize) -> Option<&Entry<T>> {
        let mut current = self.entries[index].as_ref()?;

        for i in (0..current.len).rev() {
            if i == nth_in_path {
                return Some(current);
            }

            current = self.entries[current.parent_index].as_ref()?;
        }

        None
    }

    pub fn parent(&self, index: usize) -> Option<usize> {
        let current = self.entries[index].as_ref()?;
        Some(current.parent_index)
    }

    pub fn resolve(&self, from_index: usize, where_fn: impl Fn(&T) -> bool) -> Option<&T> {
        let mut d = &self.entries[from_index];

        while let Some(Entry {
            parent_index,
            value,
            ..
        }) = d
        {
            if where_fn(value) {
                return Some(value);
            }

            d = &self.entries[*parent_index];
        }

        None
    }
}
