pub trait FromDefaults<Rule: FmtRule>: Sized {
    fn from_defaults() -> Self;
}

pub trait FmtRule: std::fmt::Debug + Copy + Eq + Sized {
    type Defaults: IntoIterator<Item = Self>;
    type Struct: FromDefaults<Self>;

    fn defaults() -> Self::Defaults;
    fn fill(acc: Self::Struct, this: Self) -> Self::Struct;
}

#[macro_export]
macro_rules! fmt_rules {
    (enum $EnumName:ident $(<$($gens:tt),*>)? for struct $StructName:ident {$(
        $Variant:ident($var_ty:ty = $default_val:literal) for $struct_field:ident
    ),* $(,)?}) => {
        pub struct $StructName $(<$($gens)*>)? {$(
            $struct_field: $var_ty
        ),*}

        #[derive(Debug, Clone, Copy, Eq)]
        pub enum $EnumName $(<$($gens)*>)? {$(
            $Variant($var_ty)
        ),*}

        #[automatically_derived]
        impl $(<$($gens)*>)? PartialEq for $EnumName $(<$($gens)*>)? {
            fn eq(&self, other: &Self) -> bool {
                $crate::__private::discriminant(self) == $crate::__private::discriminant(other)
            }
        }

        const _: () = {
            $(
                #[allow(non_camel_case_types)]
                struct $struct_field;
            )*

            #[automatically_derived]
            impl $(<$($gens)*>)? $crate::format::FmtRule for $EnumName $(<$($gens)*>)? {
                type Defaults = [Self; $(<$struct_field as $crate::__private::_1>::_1 +)* 0];
                type Struct = $StructName $(<$($gens)*>)?;

                fn defaults() -> Self::Defaults {
                    [$(
                        Self::$Variant($default_val)
                    ),*]
                }

                #[allow(clippy::needless_update)]
                fn fill(acc: Self::Struct, this: Self) -> Self::Struct {
                    match this {$(
                        Self::$Variant(val) => $StructName {
                            $struct_field: val,
                            ..acc
                        }
                    ),*}
                }
            }
        };

        const _: () = {
            #[automatically_derived]
            impl $(<$($gens)*>)? $crate::format::FromDefaults<$EnumName $(<$($gens)*>)?> for $StructName $(<$($gens)*>)? {
                fn from_defaults() -> Self {
                    let defaults = <$EnumName $(<$($gens)*>)? as $crate::format::FmtRule>::defaults();

                    $(
                        let mut $struct_field = None;
                    )*

                    for d in defaults {
                        match d {$(
                            $EnumName::$Variant(val) => {
                                $struct_field.replace(val);
                            }
                        )*}
                    }

                    Self {$(
                        // SAFETY: at this point, thanks to the macro usage, this can't be None
                        $struct_field: unsafe { $struct_field.unwrap_unchecked() }
                    ),*}
                }
            }
        };
    }
}

#[derive(Debug)]
pub struct PromptFormat<R> {
    pub rules: Vec<R>,
}

impl<R: FmtRule> Extend<R> for PromptFormat<R> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = R>,
    {
        let fmt = PromptFormat {
            rules: FromIterator::from_iter(iter),
        };
        *self = self.merge_with(&fmt);
    }
}

impl<R> FromIterator<R> for PromptFormat<R> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = R>,
    {
        Self {
            rules: FromIterator::from_iter(iter),
        }
    }
}

impl<R: FmtRule> Default for PromptFormat<R> {
    fn default() -> Self {
        Self {
            rules: <R as FmtRule>::defaults().into_iter().collect(),
        }
    }
}

impl<R: FmtRule> PromptFormat<R> {
    pub fn to_struct(&self) -> <R as FmtRule>::Struct {
        self.rules.iter().copied().fold(
            <R as FmtRule>::Struct::from_defaults(),
            <R as FmtRule>::fill,
        )
    }

    pub fn merge_with(&self, other: &PromptFormat<R>) -> PromptFormat<R> {
        self.rules
            .clone()
            .into_iter()
            .chain(
                other
                    .rules
                    .iter()
                    .filter(|rule| !self.rules.contains(rule))
                    .cloned(),
            )
            .collect()
    }
}
