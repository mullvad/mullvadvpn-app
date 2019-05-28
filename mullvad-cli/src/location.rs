use mullvad_types::relay_constraints::{Constraint, LocationConstraint};

pub fn get_subcommand() -> clap::App<'static, 'static> {
    clap::SubCommand::with_name("location")
        .arg(
            clap::Arg::with_name("country")
                .help("The two letter country code, or 'any' for no preference.")
                .required(true)
                .index(1)
                .validator(country_code_validator),
        )
        .arg(
            clap::Arg::with_name("city")
                .help("The three letter city code")
                .index(2)
                .validator(city_code_validator),
        )
        .arg(
            clap::Arg::with_name("hostname")
                .help("The hostname")
                .index(3),
        )
}

pub fn get_constraint(matches: &clap::ArgMatches<'_>) -> Constraint<LocationConstraint> {
    let country = matches.value_of("country").unwrap();
    let city = matches.value_of("city");
    let hostname = matches.value_of("hostname");

    match (country, city, hostname) {
        ("any", None, None) => Constraint::Any,
        ("any", ..) => clap::Error::with_description(
            "City can't be given when selecting 'any' country",
            clap::ErrorKind::InvalidValue,
        )
        .exit(),
        (country, None, None) => Constraint::Only(LocationConstraint::Country(country.to_owned())),
        (country, Some(city), None) => Constraint::Only(LocationConstraint::City(
            country.to_owned(),
            city.to_owned(),
        )),
        (country, Some(city), Some(hostname)) => Constraint::Only(LocationConstraint::Hostname(
            country.to_owned(),
            city.to_owned(),
            hostname.to_owned(),
        )),
        (..) => clap::Error::with_description(
            "Invalid country, city and hostname combination given",
            clap::ErrorKind::InvalidValue,
        )
        .exit(),
    }
}

fn country_code_validator(code: String) -> ::std::result::Result<(), String> {
    if code.len() == 2 || code == "any" {
        Ok(())
    } else {
        Err(String::from("Country codes must be two letters, or 'any'."))
    }
}

fn city_code_validator(code: String) -> ::std::result::Result<(), String> {
    if code.len() == 3 {
        Ok(())
    } else {
        Err(String::from("City codes must be three letters"))
    }
}
