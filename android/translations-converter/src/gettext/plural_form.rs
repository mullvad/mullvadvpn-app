use derive_more::{Display, Error};
use std::str::FromStr;

/// Known plural forms.
#[derive(Clone, Copy, Debug)]
pub enum PluralForm {
    Single,
    SingularForOne,
    SingularForZeroAndOne,
    Polish,
    Russian,
}

impl PluralForm {
    /// Obtain an instance based on a known plural formula.
    ///
    /// Plural variants need to be obtained using a formula. However, some locales have known
    /// formulas, so they can be represented as a known plural form. This constructor can return a
    /// plural form based on the formulas that are known to be used in the project.
    pub fn from_formula(formula: &str) -> Option<Self> {
        match formula {
            "nplurals=1; plural=0" => Some(PluralForm::Single),
            "nplurals=2; plural=(n != 1)" => Some(PluralForm::SingularForOne),
            "nplurals=2; plural=(n > 1)" => Some(PluralForm::SingularForZeroAndOne),
            "nplurals=4; plural=(n==1 ? 0 : (n%10>=2 && n%10<=4) && (n%100<12 || n%100>14) ? 1 : n!=1 && (n%10>=0 && n%10<=1) || (n%10>=5 && n%10<=9) || (n%100>=12 && n%100<=14) ? 2 : 3)" => {
                Some(PluralForm::Polish)
            }
            "nplurals=4; plural=((n%10==1 && n%100!=11) ? 0 : ((n%10 >= 2 && n%10 <=4 && (n%100 < 12 || n%100 > 14)) ? 1 : ((n%10 == 0 || (n%10 >= 5 && n%10 <=9)) || (n%100 >= 11 && n%100 <= 14)) ? 2 : 3))" => {
                Some(PluralForm::Russian)
            }
            _ => None
        }
    }
}

impl FromStr for PluralForm {
    type Err = UnsupportedPluralFormulaError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        PluralForm::from_formula(string)
            .ok_or_else(|| UnsupportedPluralFormulaError(string.to_owned()))
    }
}

/// Failed to create [`PluralForm`] from specified plural formula.
///
/// The formula could be an invalid formula, or support for it hasn't been added yet.
#[derive(Clone, Debug, Display, Error)]
#[display(fmt = "Unsupported plural formula: {_0}")]
pub struct UnsupportedPluralFormulaError(#[error(not(source))] String);
