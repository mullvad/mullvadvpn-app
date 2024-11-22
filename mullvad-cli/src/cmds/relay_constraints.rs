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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse location constraint from input: TODO")]
    Parse,
}

impl TryFrom<LocationArgs> for GeographicLocationConstraint {
    type Error = Error;

    fn try_from(value: LocationArgs) -> Result<Self, Self::Error> {
        match (value.country, value.city, value.hostname) {
            (country, None, None) => Ok(GeographicLocationConstraint::Country(country)),
            (country, Some(city), None) => Ok(GeographicLocationConstraint::City(country, city)),
            (country, Some(city), Some(hostname)) => Ok(GeographicLocationConstraint::Hostname(
                country, city, hostname,
            )),
            _ => Err(Error::Parse),
            //_ => unreachable!("invalid location arguments"),
        }
    }
}

impl TryFrom<LocationArgs> for LocationConstraint {
    type Error = Error;

    fn try_from(value: LocationArgs) -> Result<Self, Self::Error> {
        GeographicLocationConstraint::try_from(value).map(LocationConstraint::from)
    }
}

impl TryFrom<LocationArgs> for Constraint<GeographicLocationConstraint> {
    type Error = Error;

    fn try_from(value: LocationArgs) -> Result<Self, Self::Error> {
        if value.country.eq_ignore_ascii_case("any") {
            return Ok(Constraint::Any);
        }
        GeographicLocationConstraint::try_from(value).map(Constraint::Only)
    }
}

impl TryFrom<LocationArgs> for Constraint<LocationConstraint> {
    type Error = Error;

    fn try_from(value: LocationArgs) -> Result<Self, Self::Error> {
        LocationConstraint::try_from(value).map(Constraint::Only)
    }
}
