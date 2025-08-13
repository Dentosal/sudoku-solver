use std::ops;

use crate::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PossibleValues(u16);

impl PossibleValues {
    pub const EMPTY: Self = Self(0);
    pub const ANY: Self = Self(0b1_1111_1111);

    pub fn initial_state(value: Option<Number>) -> Self {
        if let Some(n) = value {
            Self::from(n)
        } else {
            Self(0b1_1111_1111)
        }
    }

    pub fn is_broken(&self) -> bool {
        self.0 == 0
    }

    pub fn count(&self) -> u8 {
        self.0.count_ones() as u8
    }

    pub fn contains(&self, value: Number) -> bool {
        !(*self & Self::from(value)).is_broken()
    }

    pub fn add(&mut self, value: Number) {
        *self |= Self::from(value);
    }

    pub fn remove(&mut self, value: Number) {
        self.0 &= !(1 << value.index());
    }

    pub fn determined(&self) -> Option<Number> {
        if self.count() == 1 {
            Some(Number(self.0.trailing_zeros() as u8 + 1))
        } else {
            None
        }
    }

    pub fn options(&self) -> Vec<Number> {
        let mut result = Vec::new();

        let mut tmp = self.0;
        let mut num = Number::MIN;
        while tmp != 0 {
            if tmp & 1 != 0 {
                result.push(num);
            }

            num = Number(num.0 + 1);
            tmp = tmp >> 1;
        }
        result
    }
}

impl ops::BitAnd for PossibleValues {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}
impl ops::BitAndAssign for PossibleValues {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl ops::BitOr for PossibleValues {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
impl ops::BitOrAssign for PossibleValues {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl From<Number> for PossibleValues {
    fn from(value: Number) -> Self {
        Self(1 << value.index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_possible_values_num() {
        let mut pv = PossibleValues::initial_state(Some(Number(5)));
        assert_eq!(pv.count(), 1);
        assert!(!pv.is_broken());
        pv.remove(Number(5));
        assert!(pv.is_broken());
    }

    #[test]
    fn test_possible_values_any() {
        let mut pv = PossibleValues::initial_state(None);
        assert_eq!(pv.count(), 9);
        assert!(!pv.is_broken());
        pv.remove(Number(1));
        assert_eq!(pv.count(), 8);
        assert!(!pv.is_broken());
    }

    #[test]
    fn test_possible_values_bitwise_ops() {
        let pv1 = PossibleValues::initial_state(Some(Number(1)));
        let pv2 = PossibleValues::initial_state(Some(Number(2)));
        let pv3 = PossibleValues::initial_state(Some(Number(3)));

        let and_result = pv1 & pv2;
        let or_result = pv1 | pv3;

        assert_eq!(and_result.count(), 0); // 1 and 2 have no overlap
        assert_eq!(or_result.count(), 2); // 1 and 3 are combined
        assert!(and_result.is_broken());
        assert!(!or_result.is_broken());

        assert!(pv1.contains(Number(1)));
        assert!(pv2.contains(Number(2)));
        assert!(!pv2.contains(Number(1)));
        assert!(!pv1.contains(Number(2)));

        assert_eq!(pv1.options(), vec![Number(1)]);
        assert_eq!(pv2.options(), vec![Number(2)]);
        assert_eq!((pv1 | pv2).options(), vec![Number(1), Number(2)]);

        assert_eq!(pv1.determined(), Some(Number(1)));
        assert_eq!(pv2.determined(), Some(Number(2)));
        assert_eq!((pv1 | pv2).determined(), None);
    }
}
