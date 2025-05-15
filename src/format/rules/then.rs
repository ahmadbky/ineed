use crate::format::{FmtRule, Mergeable, Unwrappable};

pub struct ThenFmtRules<A, B> {
    pub a_rules: A,
    pub b_rules: B,
}

impl<A, B, R> From<R> for ThenFmtRules<A, B>
where
    A: From<R>,
    B: From<R>,
    R: FmtRule,
{
    fn from(value: R) -> Self {
        Self {
            a_rules: A::from(value),
            b_rules: B::from(value),
        }
    }
}

impl<A, B> Mergeable for ThenFmtRules<A, B>
where
    A: Mergeable,
    B: Mergeable,
{
    fn merge_with(&self, other: &Self) -> Self {
        Self {
            a_rules: self.a_rules.merge_with(&other.a_rules),
            b_rules: self.b_rules.merge_with(&other.b_rules),
        }
    }
}

impl<A, B> Unwrappable for ThenFmtRules<A, B> {
    type Unwrapped = ();
    fn unwrap(&self) -> Self::Unwrapped {}
}
