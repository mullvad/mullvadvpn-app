use crate::proto::RelayLocation;

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

pub fn get_constraint(matches: &clap::ArgMatches<'_>) -> RelayLocation {
    let country_original = matches.value_of("country").unwrap();
    let country = country_original.to_lowercase();
    let city = matches.value_of("city").map(str::to_lowercase);
    let hostname = matches.value_of("hostname").map(str::to_lowercase);

    match (country_original, city, hostname) {
        ("any", None, None) => RelayLocation::default(),
        ("any", ..) => clap::Error::with_description(
            "City can't be given when selecting 'any' country",
            clap::ErrorKind::InvalidValue,
        )
        .exit(),
        (_, None, None) => RelayLocation {
            country,
            ..Default::default()
        },
        (_, Some(city), None) => RelayLocation {
            country,
            city,
            ..Default::default()
        },
        (_, Some(city), Some(hostname)) => RelayLocation {
            country,
            city,
            hostname,
        },
        (..) => clap::Error::with_description(
            "Invalid country, city and hostname combination given",
            clap::ErrorKind::InvalidValue,
        )
        .exit(),
    }
}

pub fn format_location(location: &RelayLocation) -> String {
    if !location.hostname.is_empty() {
        format!(
            "city {}, {}, hostname {}",
            location.city, location.country, location.hostname
        )
    } else if !location.city.is_empty() {
        format!("city {}, {}", location.city, location.country)
    } else if !location.country.is_empty() {
        format!("country {}", location.country)
    } else {
        "any".to_string()
    }
}

fn country_code_validator(code: String) -> std::result::Result<(), String> {
    if code.len() == 2 || code == "any" {
        Ok(())
    } else {
        Err(String::from("Country codes must be two letters, or 'any'."))
    }
}

fn city_code_validator(code: String) -> std::result::Result<(), String> {
    if code.len() == 3 {
        Ok(())
    } else {
        Err(String::from("City codes must be three letters"))
    }
}
