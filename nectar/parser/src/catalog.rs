use std::fmt;
use std::ops;
use std::hash;
use std::iter;
use std::marker::PhantomData;


pub struct Catalog<T> {
    entries: Vec<T>
}
impl<T> Catalog<T> {
    pub fn new() -> Catalog<T> {
        Catalog{
            entries: vec!()
        }
    }

    pub fn add<'a>(&mut self, entry: T) -> Index<T> {
        let index = self.entries.len();
        self.entries.push(entry);
        Index::new(index)
    }

    pub fn lookup<'a>(&'a self, index: Index<T>) -> Indexed<'a, T> {
        self.__lookup(index.index)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter{
            table: self,
            index: 0
        }
    }

    fn __lookup<'a>(&'a self, index: usize) -> Indexed<'a, T> {
        let entry = self.entries.get(index).expect("index.rs: lookup failed");
        Indexed::new(index, entry)
    }
}

pub struct Iter<'a, T> where T: 'a {
    table: &'a Catalog<T>,
    index: usize
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = Indexed<'a, T>;
    fn next(&mut self) -> Option<Indexed<'a, T>> {
        match self.table.entries.get(self.index) {
            None => None,
            Some(entry) => {
                let index = self.index;
                self.index += 1;
                Some(Indexed::new(index, entry))
            }
        }
    }
}


pub struct Indexed<'a, T> where T : 'a {
    index: usize,
    pub data: &'a T
}
impl<'a, T> Indexed<'a, T> {
    fn new(index: usize, data: &'a T) -> Indexed<'a, T> {
        Indexed{
            index: index,
            data:  data
        }
    }

    pub fn index(&self) -> Index<T> {
        Index::new(self.index)
    }
}
impl<'a, T> Clone for Indexed<'a, T> {
    fn clone(&self) -> Indexed<'a, T> {
        Indexed::new(self.index, self.data)
    }
}
impl<'a, T> Copy for Indexed<'a, T> {}
impl<'a, T> ops::Deref for Indexed<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.data
    }
}
impl<'a, T> PartialEq for Indexed<'a, T> {
    fn eq(&self, other: &Indexed<'a, T>) -> bool {
        self.index == other.index
    }
}
impl<'a, T> Eq for Indexed<'a, T> {}
impl<'a, T> hash::Hash for Indexed<'a, T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
impl<'a, T> fmt::Debug for Indexed<'a, T> where T : fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
impl<'a, T> fmt::Display for Indexed<'a, T> where T : fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}


pub struct Index<T> {
    index: usize,
    phantom: PhantomData<T>
}
impl<T> Index<T> {
    fn new(index: usize) -> Index<T> {
        Index{
            index: index,
            phantom: PhantomData
        }
    }
}
impl<T> Clone for Index<T> {
    fn clone(&self) -> Index<T> {
        Index::new(self.index)
    }
}
impl<T> Copy for Index<T> {}
impl<T> hash::Hash for Index<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Index<T>) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for Index<T> {}
impl<T> fmt::Debug for Index<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}
