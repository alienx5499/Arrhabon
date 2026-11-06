//! The time (unix seconds) by which the visa outcome must be attested.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(i64);

impl Deadline {
    pub const fn new(unix: i64) -> Self {
        Deadline(unix)
    }

    pub const fn unix(self) -> i64 {
        self.0
    }

    /// Past from the deadline second onward, so "before" and "past" never overlap.
    pub const fn is_past(self, now: i64) -> bool {
        now >= self.0
    }
}
