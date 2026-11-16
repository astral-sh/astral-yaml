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
}

#[cfg(test)]
mod tests {
    use super::Budget;

    #[test]
    fn a_failed_charge_preserves_the_balance() {
        let budget = Budget::new(3);
        assert_eq!(budget.try_charge(4), None);
        assert_eq!(budget.remaining(), 3);
        assert_eq!(budget.try_charge(3), Some(()));
        assert_eq!(budget.try_charge(0), Some(()));
        assert_eq!(budget.try_charge(1), None);
    }
}
