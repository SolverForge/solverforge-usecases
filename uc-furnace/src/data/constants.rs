use rand::rngs::SmallRng;
use rand::RngExt;

pub const CUSTOMERS: &[&str] = &[
    "Brembo",
    "Dana",
    "SKF",
    "Interpump",
    "Carraro",
    "Bondioli & Pavesi",
    "Comer Industries",
    "Lombardini",
    "Bonfiglioli",
    "Dana Graziano",
];

pub fn rand_range(rng: &mut SmallRng, lo: u32, hi: u32) -> u32 {
    rng.random_range(lo..=hi)
}

pub fn pick<'a, T>(rng: &mut SmallRng, slice: &'a [T]) -> &'a T {
    let idx = rng.random_range(0..slice.len());
    &slice[idx]
}
