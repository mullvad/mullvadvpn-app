use clap::Args;
use mullvad_types::{
    constraints::Constraint,
    location::{CityCode, CountryCode, Hostname},
    relay_constraints::{GeographicLocationConstraint, LocationConstraint},
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

impl From<LocationArgs> for Constraint<GeographicLocationConstraint> {
    fn from(value: LocationArgs) -> Self {
        if value.country.eq_ignore_ascii_case("any") {
            return Constraint::Any;
        }

        Constraint::Only(match (value.country, value.city, value.hostname) {
            (country, None, None) => GeographicLocationConstraint::Country(country),
            (country, Some(city), None) => GeographicLocationConstraint::City(country, city),
            (country, Some(city), Some(hostname)) => {
                GeographicLocationConstraint::Hostname(country, city, hostname)
            }

            _ => unreachable!("invalid location arguments"),
        })
    }
}

impl From<LocationArgs> for Constraint<LocationConstraint> {
    fn from(value: LocationArgs) -> Self {
        if value.country.eq_ignore_ascii_case("any") {
            return Constraint::Any;
        }

        let location = match (value.country, value.city, value.hostname) {
            (country, None, None) => GeographicLocationConstraint::Country(country),
            (country, Some(city), None) => GeographicLocationConstraint::City(country, city),
            (country, Some(city), Some(hostname)) => {
                GeographicLocationConstraint::Hostname(country, city, hostname)
            }
            _ => unreachable!("invalid location arguments"),
        };
        Constraint::Only(LocationConstraint::Location(location))
    }
}
