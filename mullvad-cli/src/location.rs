use mullvad_types::relay_constraints::{Constraint, LocationConstraint};

pub fn get_subcommand() -> clap::App<'static> {
    clap::App::new("location")
        .arg(
            clap::Arg::new("country")
                .help("The two letter country code, or 'any' for no preference.")
                .required(true)
                .index(1)
                .validator(country_code_validator),
        )
        .arg(
            clap::Arg::new("city")
                .help("The three letter city code")
                .index(2)
                .validator(city_code_validator),
        )
        .arg(clap::Arg::new("hostname").help("The hostname").index(3))
}

pub fn get_constraint_from_args(matches: &clap::ArgMatches) -> Constraint<LocationConstraint> {
    let country = matches.value_of("country").unwrap();
    let city = matches.value_of("city");
    let hostname = matches.value_of("hostname");
    get_constraint(country, city, hostname)
}

pub fn get_constraint<T: AsRef<str>>(
    country: T,
    city: Option<T>,
    hostname: Option<T>,
) -> Constraint<LocationConstraint> {
    let country_original = country.as_ref();
    let country = country_original.to_lowercase();
    let city = city.map(|s| s.as_ref().to_lowercase());
    let hostname = hostname.map(|s| s.as_ref().to_lowercase());

    match (country_original, city, hostname) {
        ("any", None, None) => Constraint::Any,
        ("any", ..) => clap::Error::raw(
            clap::ErrorKind::InvalidValue,
            "City can't be given when selecting 'any' country",
        )
        .exit(),
        (_, None, None) => Constraint::Only(LocationConstraint::Country(country)),
        (_, Some(city), None) => Constraint::Only(LocationConstraint::City(country, city)),
        (_, Some(city), Some(hostname)) => {
            Constraint::Only(LocationConstraint::Hostname(country, city, hostname))
        }
        (..) => clap::Error::raw(
            clap::ErrorKind::InvalidValue,
            "Invalid country, city and hostname combination given",
        )
        .exit(),
    }
}

pub fn country_code_validator(code: &str) -> std::result::Result<(), String> {
    if code.len() == 2 || code == "any" {
        Ok(())
    } else {
        Err(String::from("Country codes must be two letters, or 'any'."))
    }
}

pub fn city_code_validator(code: &str) -> std::result::Result<(), String> {
    if code.len() == 3 {
        Ok(())
    } else {
        Err(String::from("City codes must be three letters"))
    }
}
