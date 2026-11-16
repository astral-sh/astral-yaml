use crate::{Error, Span};
use std::cell::Cell;

/// A remaining allowance for work or bytes.
///
/// Borrow the budget to share its balance. Cloning it creates an independent
/// budget with the same remaining allowance.
#[derive(Clone, Debug)]
pub struct Budget {
    remaining: Cell<usize>,
}

impl Budget {
    /// Start with `limit` units available.
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self {
            remaining: Cell::new(limit),
        }
    }

    /// Return the number of units still available.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.remaining.get()
    }

    /// Deduct `amount`, returning `None` and preserving the balance if it exceeds
    /// the remaining allowance.
    #[inline]
    pub fn try_charge(&self, amount: usize) -> Option<()> {
        self.remaining.set(self.remaining().checked_sub(amount)?);
        Some(())
    }

    /// Deduct `amount`, reporting exhaustion at `span`.
    ///
    /// # Errors
    /// If the charge exceeds the remaining allowance, return `message` at `span`
    /// and leave the balance unchanged.
    #[inline]
    pub fn charge_at(&self, amount: usize, span: Span, message: &'static str) -> Result<(), Error> {
        self.try_charge(amount).ok_or(Error { span, message })
    }
}

#[cfg(test)]
mod tests {
    use super::Budget;
    use crate::{Error, Span};

    #[test]
    fn a_failed_charge_preserves_the_balance() {
        let budget = Budget::new(3);
        assert_eq!(budget.try_charge(4), None);
        assert_eq!(budget.remaining(), 3);
        assert_eq!(budget.try_charge(3), Some(()));
        assert_eq!(budget.try_charge(0), Some(()));
        assert_eq!(budget.try_charge(1), None);
    }

    #[test]
    fn charge_at_reports_the_callers_span_and_message() {
        let budget = Budget::new(1);
        let span = Span::new(4, 7).unwrap();
        let message = "item limit exceeded";

        budget.charge_at(1, span, message).unwrap();
        assert_eq!(
            budget.charge_at(1, span, message),
            Err(Error { span, message })
        );
    }
}
