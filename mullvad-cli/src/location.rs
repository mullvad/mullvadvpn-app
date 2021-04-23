use mullvad_management_interface::types::RelayLocation;

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

pub fn get_constraint_from_args(matches: &clap::ArgMatches<'_>) -> RelayLocation {
    let country = matches.value_of("country").unwrap();
    let city = matches.value_of("city");
    let hostname = matches.value_of("hostname");
    get_constraint(country, city, hostname)
}

pub fn get_constraint<T: AsRef<str>>(
    country: T,
    city: Option<T>,
    hostname: Option<T>,
) -> RelayLocation {
    let country_original = country.as_ref();
    let country = country_original.to_lowercase();
    let city = city.map(|s| s.as_ref().to_lowercase());
    let hostname = hostname.map(|s| s.as_ref().to_lowercase());

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

pub fn format_location(location: Option<&RelayLocation>) -> String {
    if let Some(location) = location {
        if !location.hostname.is_empty() {
            return format!(
                "city {}, {}, hostname {}",
                location.city, location.country, location.hostname
            );
        } else if !location.city.is_empty() {
            return format!("city {}, {}", location.city, location.country);
        } else if !location.country.is_empty() {
            return format!("country {}", location.country);
        }
    }
    "any location".to_string()
}

pub fn format_providers(providers: &Vec<String>) -> String {
    if !providers.is_empty() {
        format!("provider(s) {}", providers.join(", "))
    } else {
        "any provider".to_string()
    }
}

pub fn country_code_validator<T: AsRef<str>>(code: T) -> std::result::Result<(), String> {
    if code.as_ref().len() == 2 || code.as_ref() == "any" {
        Ok(())
    } else {
        Err(String::from("Country codes must be two letters, or 'any'."))
    }
}

pub fn city_code_validator<T: AsRef<str>>(code: T) -> std::result::Result<(), String> {
    if code.as_ref().len() == 3 {
        Ok(())
    } else {
        Err(String::from("City codes must be three letters"))
    }
}
