//! USDC amounts in micro-units (6 decimals) with checked arithmetic only.

use crate::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Usdc(u64);

impl Usdc {
    pub const ZERO: Usdc = Usdc(0);

    pub const fn new(micro: u64) -> Self {
        Usdc(micro)
    }

    pub const fn micro(self) -> u64 {
        self.0
    }

    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn checked_add(self, other: Usdc) -> Result<Usdc, DomainError> {
        self.0
            .checked_add(other.0)
            .map(Usdc)
            .ok_or(DomainError::Overflow)
    }

    pub fn checked_sub(self, other: Usdc) -> Result<Usdc, DomainError> {
        self.0
            .checked_sub(other.0)
            .map(Usdc)
            .ok_or(DomainError::Underflow)
    }
}
