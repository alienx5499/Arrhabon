use arrhabon_domain::{Deadline, DomainError, Usdc};

#[test]
fn usdc_checked_add_and_sub() {
    let a = Usdc::new(1_500_000);
    let b = Usdc::new(500_000);
    assert_eq!(a.checked_add(b), Ok(Usdc::new(2_000_000)));
    assert_eq!(a.checked_sub(b), Ok(Usdc::new(1_000_000)));
}

#[test]
fn usdc_add_overflow_is_an_error() {
    assert_eq!(
        Usdc::new(u64::MAX).checked_add(Usdc::new(1)),
        Err(DomainError::Overflow)
    );
}

#[test]
fn usdc_sub_underflow_is_an_error() {
    assert_eq!(
        Usdc::new(1).checked_sub(Usdc::new(2)),
        Err(DomainError::Underflow)
    );
}

#[test]
fn usdc_zero() {
    assert!(Usdc::ZERO.is_zero());
    assert_eq!(Usdc::new(7).micro(), 7);
}

#[test]
fn deadline_is_past_at_and_after_its_second() {
    let d = Deadline::new(100);
    assert!(!d.is_past(99));
    assert!(d.is_past(100));
    assert!(d.is_past(101));
    assert_eq!(d.unix(), 100);
}
