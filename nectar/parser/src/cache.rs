 use std::fmt;
use std::ops;
use std::hash;
use std::marker::PhantomData;


pub struct CacheId<T> {
    index: usize,
    phantom: PhantomData<T>
}
impl<T> CacheId<T> {
    fn new(index: usize) -> CacheId<T> {
        CacheId{
            index: index,
            phantom: PhantomData
        }
    }
}
impl<T> Clone for CacheId<T> {
    fn clone(&self) -> CacheId<T> {
        CacheId::new(self.index)
    }
}
impl<T> Copy for CacheId<T> {}
impl<T> hash::Hash for CacheId<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
impl<T> PartialEq for CacheId<T> {
    fn eq(&self, other: &CacheId<T>) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for CacheId<T> {}
impl<T> fmt::Debug for CacheId<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.index)
    }
}


pub struct CacheEntry<T> {
    pub index: CacheId<T>,
    pub data: T
}
impl<T> CacheEntry<T> {
    fn new(index: CacheId<T>, data: T) -> CacheEntry<T> {
        CacheEntry{
            index: index,
            data:  data
        }
    }
}
impl<T> ops::Deref for CacheEntry<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.data
    }
}
impl<T> PartialEq for CacheEntry<T> {
    fn eq(&self, other: &CacheEntry<T>) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for CacheEntry<T> {}
impl<T> hash::Hash for CacheEntry<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
impl<T: Clone> Clone for CacheEntry<T> {
    fn clone(&self) -> CacheEntry<T> {
        CacheEntry::new(self.index, self.data.clone())
    }
}
impl<T: Copy> Copy for CacheEntry<T> {}
impl<T> fmt::Debug for CacheEntry<T> where T : fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.data)
    }
}
impl<T> fmt::Display for CacheEntry<T> where T : fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}


pub struct Cache<T> {
    pub items: Vec<CacheEntry<T>>
}
impl<T> Cache<T> {
    pub fn new() -> Cache<T> {
        Cache{
            items: vec!()
        }
    }

    pub fn add(&mut self, data: T) -> CacheId<T> {
        let index = CacheId::new(self.items.len());
        let item  = CacheEntry::new(index, data);
        self.items.push(item);
        index
    }

    pub fn lookup(&self, index: CacheId<T>) -> &CacheEntry<T> {
        &self.items.get(index.index).expect("cache.rs: lookup failed")
    }
}

