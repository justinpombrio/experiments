struct Link<N> {
    node: Option<N>,
    prev_xor_next: *mut Link<N>,
}

impl<N> Link<N> {
    fn new(node: Option<N>, prev_xor_next: *mut Link<N>) -> *mut Link<N> {
        let link = Link {
            node,
            prev_xor_next,
        };
        Box::leak(Box::new(link))
    }

    fn xor(x: *mut Link<N>, y: *mut Link<N>) -> *mut Link<N> {
        ((x as usize) ^ (y as usize)) as *mut Link<N>
    }

    fn next(tail: *mut Link<N>, head: *mut Link<N>) -> *mut Link<N> {
        unsafe { Link::xor(tail, (*head).prev_xor_next) }
    }

    fn prev(tail: *mut Link<N>, head: *mut Link<N>) -> *mut Link<N> {
        unsafe { Link::xor((*tail).prev_xor_next, head) }
    }
}

pub struct CursorMut<N> {
    head: *mut Link<N>,
    tail: *mut Link<N>,
}

impl<N> CursorMut<N> {
    pub fn current(&mut self) -> Option<&mut N> {
        unsafe { (*self.head).node.as_mut() }
    }

    pub fn move_next(&mut self) {
        self.move_next_once();
        if unsafe { (*self.tail).node.is_none() && (*self.head).node.is_none() } {
            self.move_next_once();
        }
    }

    pub fn move_prev(&mut self) {
        self.move_prev_once();
        if unsafe { (*self.tail).node.is_none() && (*self.head).node.is_none() } {
            self.move_prev_once();
        }
    }

    fn move_next_once(&mut self) {
        *self = CursorMut {
            tail: self.head,
            head: Link::next(self.tail, self.head),
        }
    }

    fn move_prev_once(&mut self) {
        *self = CursorMut {
            tail: Link::prev(self.tail, self.head),
            head: self.tail,
        }
    }

    pub fn insert(&mut self, node: N) {
        let prev = Link::prev(self.tail, self.head);
        let next = Link::next(self.tail, self.head);
        let ptr = Link::new(Some(node), Link::xor(self.tail, self.head));
        unsafe {
            (*self.head).prev_xor_next = Link::xor(ptr, next);
            (*self.tail).prev_xor_next = Link::xor(prev, ptr);
            self.tail = ptr;
        }
    }

    pub fn remove(&mut self) -> Option<N> {
        if unsafe { (*self.tail).node.is_none() } {
            return None;
        }
        let next = Link::next(self.tail, self.head);
        let prev = Link::prev(self.tail, self.head);
        let prevprev = Link::prev(prev, self.tail);
        unsafe {
            let node = (*self.tail).node.take();
            (*self.head).prev_xor_next = Link::xor(prev, next);
            (*prev).prev_xor_next = Link::xor(prevprev, self.head);
            self.tail = prev;
            node
        }
    }
}

pub struct LinkedList<N> {
    back: *mut Link<N>,
    front: *mut Link<N>,
}

impl<N> LinkedList<N> {
    pub fn new() -> LinkedList<N> {
        LinkedList {
            back: Link::new(None, 0 as *mut Link<N>),
            front: Link::new(None, 0 as *mut Link<N>),
        }
    }

    pub fn cursor(&mut self) -> CursorMut<N> {
        CursorMut {
            tail: self.front,
            head: Link::next(self.back, self.front),
        }
    }
}

fn main() {
    let mut list = LinkedList::new();
    let mut cursor = list.cursor();
    for ch in "Hello, world".chars() {
        cursor.insert(ch);
    }
    for _ in 0..6 {
        cursor.move_next();
    }
    for _ in 0..4 {
        cursor.remove().unwrap();
    }
    *cursor.current().unwrap() = 'i';
    cursor.move_prev();
    for _ in 0..10 {
        println!("{}", cursor.current().unwrap());
        cursor.move_next();
    }
}
