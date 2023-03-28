use clap::Args;
use mullvad_types::{
    location::{CityCode, CountryCode, Hostname},
    relay_constraints::{Constraint, LocationConstraint},
};

#[derive(Args, Debug, Clone)]
pub struct LocationArgs {
    /// A two-letter country code, or 'any'.
    pub country: CountryCode,
    /// A three-letter city code.
    pub city: Option<CityCode>,
    /// A host name, such as "se-got-wg-101".
    pub hostname: Option<Hostname>,
}

impl From<LocationArgs> for Constraint<LocationConstraint> {
    fn from(value: LocationArgs) -> Self {
        if value.country.eq_ignore_ascii_case("any") {
            return Constraint::Any;
        }

        match (value.country, value.city, value.hostname) {
            (country, None, None) => Constraint::Only(LocationConstraint::Country(country)),
            (country, Some(city), None) => {
                Constraint::Only(LocationConstraint::City(country, city))
            }
            (country, Some(city), Some(hostname)) => {
                Constraint::Only(LocationConstraint::Hostname(country, city, hostname))
            }
            _ => unreachable!("invalid location arguments"),
        }
    }
}
