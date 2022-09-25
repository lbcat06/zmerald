use std::cmp::Ordering;
use std::hash::{ Hasher, Hash };

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Number {
    Integer(i64),
    Float(Float),
}

#[derive(Copy, Clone, Debug)]
pub struct Float(f64);

impl Float {
    pub fn new(v: f64) -> Self {
        Float(v)
    }

    pub fn get(self) -> f64 {
        self.0
    }
}

impl Number {
    pub fn new(v: impl Into<Number>) -> Self {
        v.into()
    }

    pub fn into_f64(self) -> f64 {
        self.map_to(|i| i as f64, |f| f)
    }

    pub fn as_f64(self) -> Option<f64> {
        self.map_to(|_| None, Some)
    }

    pub fn as_i64(self) -> Option<i64> {
        self.map_to(Some, |_| None)
    }

    pub fn map_to<T>(self, integer_fn: impl FnOnce(i64) -> T, float_fn: impl FnOnce(f64) -> T) -> T {
        match self {
            Number::Integer(i) => integer_fn(i),
            Number::Float(Float(f)) => float_fn(f),
        }
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Number {
        Number::Float(Float(f))
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Number {
        Number::Integer(i)
    }
}

impl From<i32> for Number {
    fn from(i: i32) -> Number {
        Number::Integer(i64::from(i))
    }
}

// The following number conversion checks if the integer fits losslessly into an i64, before
// constructing a Number::Integer variant. If not, the conversion defaults to float.

impl From<u64> for Number {
    fn from(i: u64) -> Number {
        if i <= std::i64::MAX as u64 {
            Number::Integer(i as i64)
        } else {
            Number::new(i as f64)
        }
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.0.is_nan() && other.0.is_nan() || self.0 == other.0
    }
}

impl Eq for Float {}

impl Hash for Float {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64);
    }
}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            _ => self.0.partial_cmp(&other.0),
        }
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Bug: Contract violation")
    }
}