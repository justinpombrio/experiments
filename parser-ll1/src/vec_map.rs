#[derive(Debug, Clone)]
pub struct VecMap<T>(Vec<Option<T>>);

impl<T> VecMap<T> {
    pub fn new() -> VecMap<T> {
        VecMap(Vec::new())
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        match self.0.get(index) {
            None | Some(None) => None,
            Some(Some(value)) => Some(value),
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.0.resize_with(index + 1, || None);
        self.0[index] = Some(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.0.iter().enumerate().filter_map(|(i, opt)| match opt {
            None => None,
            Some(val) => Some((i, val)),
        })
    }

    pub fn into_iter(self) -> impl Iterator<Item = (usize, T)> {
        self.0
            .into_iter()
            .enumerate()
            .filter_map(|(i, opt)| match opt {
                None => None,
                Some(val) => Some((i, val)),
            })
    }
}
