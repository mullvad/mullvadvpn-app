use clap::Args;
use mullvad_types::{
    location::{CityCode, CountryCode, Hostname},
    relay_constraints::{Constraint, GeographicLocationConstraint, LocationConstraint},
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

        match (value.country, value.city, value.hostname) {
            (country, None, None) => {
                Constraint::Only(GeographicLocationConstraint::Country(country))
            }
            (country, Some(city), None) => {
                Constraint::Only(GeographicLocationConstraint::City(country, city))
            }
            (country, Some(city), Some(hostname)) => Constraint::Only(
                GeographicLocationConstraint::Hostname(country, city, hostname),
            ),
            _ => unreachable!("invalid location arguments"),
        }
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
