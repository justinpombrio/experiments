use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

// TODO: Should normalize Flows, removing extremely small rates that might crop up from rounding
// errors. Ideally, the public interface should never expose small rates.

pub type ItemName = &'static str;

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub name: ItemName,
    pub rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    /// Items with negative rates are inputs; those with positive rates are ouputs. Rates are in
    /// units per second.
    pub items: HashMap<ItemName, f64>,
}

impl Flow {
    pub fn new() -> Flow {
        Flow {
            items: HashMap::new(),
        }
    }
}

impl fmt::Display for Flow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.items.is_empty() {
            return write!(f, "      (none)");
        }
        let mut inputs: Vec<(ItemName, f64)> = Vec::new();
        let mut outputs = Vec::new();
        for (item, rate) in &self.items {
            if rate.abs() < 0.000000001 {
                ()
            } else if *rate < 0.0 {
                inputs.push((*item, *rate));
            } else {
                outputs.push((*item, *rate));
            }
        }
        inputs.sort_unstable_by_key(|(item, _)| *item);
        outputs.sort_unstable_by_key(|(item, _)| *item);
        write!(f, "       ")?;
        for (i, (item, rate)) in inputs.iter().enumerate() {
            write!(f, "{} {}", -rate, item)?;
            if i + 1 < inputs.len() {
                write!(f, ", ")?;
            }
        }
        write!(f, "\n    -> ")?;
        for (i, (item, rate)) in outputs.iter().enumerate() {
            write!(f, "{} {}", rate, item)?;
            if i + 1 < outputs.len() {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}

impl Add for Item {
    type Output = Item;
    fn add(self, other: Item) -> Item {
        assert_eq!(self.name, other.name, "Cannot add items of different types");
        Item {
            name: self.name,
            rate: self.rate + other.rate,
        }
    }
}

impl Sub for Item {
    type Output = Item;
    fn sub(self, other: Item) -> Item {
        assert_eq!(self.name, other.name, "Cannot sub items of different types");
        self + (other * -1.0)
    }
}

impl Mul<f64> for Item {
    type Output = Item;
    fn mul(mut self, x: f64) -> Item {
        self.rate *= x;
        self
    }
}

impl AddAssign<Item> for Flow {
    fn add_assign(&mut self, item: Item) {
        *self.items.entry(item.name).or_default() += item.rate;
    }
}

impl SubAssign<Item> for Flow {
    fn sub_assign(&mut self, item: Item) {
        *self.items.entry(item.name).or_default() -= item.rate;
    }
}

impl AddAssign<Flow> for Flow {
    fn add_assign(&mut self, flow: Flow) {
        for (name, rate) in flow.items {
            *self += Item {
                name: name,
                rate: rate,
            };
        }
    }
}

impl SubAssign<Flow> for Flow {
    fn sub_assign(&mut self, flow: Flow) {
        for (name, rate) in flow.items {
            *self -= Item {
                name: name,
                rate: rate,
            };
        }
    }
}

impl MulAssign<f64> for Flow {
    fn mul_assign(&mut self, multiplier: f64) {
        for rate in self.items.values_mut() {
            *rate *= multiplier;
        }
    }
}
