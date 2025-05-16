use crate::format::{ConstDefault, FmtRule, Mergeable, Partial};

/// The set of rules accepted by chained prompts (i.e. with
/// [`Promptable::then`](crate::Promptable::then)).
///
/// See the [module documentation](crate::format) for more information.
#[derive(Default)]
pub struct ThenFmtRules<A, B> {
    /// The rules of the first prompt.
    pub a_rules: A,
    /// The rules of the second prompt.
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

/// The expanded version of [`ThenFmtRules`].
pub struct ExpandedThenFmtRules<A, B> {
    /// The expanded version of the first prompt set of rules.
    pub a_rules: A,
    /// The expanded version of the second prompt set of rules.
    pub b_rules: B,
}

impl<A, B> ConstDefault for ExpandedThenFmtRules<A, B>
where
    A: ConstDefault,
    B: ConstDefault,
{
    const DEFAULT: Self = Self {
        a_rules: A::DEFAULT,
        b_rules: B::DEFAULT,
    };
}

impl<A, B> Partial for ThenFmtRules<A, B>
where
    A: Partial,
    B: Partial,
{
    type Expanded = ExpandedThenFmtRules<<A as Partial>::Expanded, <B as Partial>::Expanded>;
    fn expand(&self) -> Self::Expanded {
        ExpandedThenFmtRules {
            a_rules: self.a_rules.expand(),
            b_rules: self.b_rules.expand(),
        }
    }
}
