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

pub fn get_constraint(matches: &clap::ArgMatches<'_>) -> Vec<String> {
    let country_original = matches.value_of("country").unwrap();
    let country = country_original.to_lowercase();
    let city = matches.value_of("city").map(str::to_lowercase);
    let hostname = matches.value_of("hostname").map(str::to_lowercase);

    match (country_original, city, hostname) {
        ("any", None, None) => Vec::with_capacity(0),
        ("any", ..) => clap::Error::with_description(
            "City can't be given when selecting 'any' country",
            clap::ErrorKind::InvalidValue,
        )
        .exit(),
        (_, None, None) => [country].to_vec(),
        (_, Some(city), None) => [country, city].to_vec(),
        (_, Some(city), Some(hostname)) => [country, city, hostname].to_vec(),
        (..) => clap::Error::with_description(
            "Invalid country, city and hostname combination given",
            clap::ErrorKind::InvalidValue,
        )
        .exit(),
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
