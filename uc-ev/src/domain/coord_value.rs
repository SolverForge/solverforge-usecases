//! Bit-exact coordinate wrapper.
//!
//! SolverForge derives `Eq` and `Hash` on facts and planning entities so it can
//! compare and fingerprint them, and plain `f64` implements neither. A
//! coordinate here is a fixed input value, so comparing its exact bits is the
//! right equality: two stops are the same place only when their coordinates
//! are bit-for-bit identical.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Latitude or longitude in decimal degrees, serialized as a bare number.
#[derive(Copy, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CoordValue(pub f64);

impl CoordValue {
    /// The wrapped degrees value.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl From<f64> for CoordValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl fmt::Debug for CoordValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for CoordValue {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for CoordValue {}

impl Hash for CoordValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}
