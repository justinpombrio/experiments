use std::fmt;

pub fn display_sep<T, I>(f: &mut fmt::Formatter, sep: &str, iter: I)
                         -> fmt::Result
    where I: Iterator<Item = T>, T: fmt::Display
{
    let mut iter = iter;
    match iter.next() {
        None => (),
        Some(thing) => {
            try!(write!(f, "{}", thing));
            for thing in iter {
                try!(write!(f, "{}", sep));
                try!(write!(f, "{}", thing));
            }
        }
    };
    write!(f, "")
}

pub fn take_vec<T>(vec: &mut Vec<T>) -> Vec<T> {
    let mut copy = vec!();
    copy.append(vec);
    copy
}
