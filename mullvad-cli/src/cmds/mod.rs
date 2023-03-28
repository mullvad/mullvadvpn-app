use clap::builder::{PossibleValuesParser, TypedValueParser, ValueParser};

pub mod account;
pub mod auto_connect;
pub mod beta_program;
pub mod bridge;
pub mod dns;
pub mod lan;
pub mod lockdown;
pub mod obfuscation;
pub mod relay;
pub mod relay_constraints;
pub mod reset;
pub mod split_tunnel;
pub mod status;
pub mod tunnel;
pub mod tunnel_state;
pub mod version;

/// A value parser that parses "on" or "off" into a boolean
fn on_off_parser() -> ValueParser {
    on_off_parser_custom("on", "off")
}

/// A value parser that parses `on_label` into true, and `off_label` into false
fn on_off_parser_custom(on_label: &'static str, off_label: &'static str) -> ValueParser {
    ValueParser::new(
        PossibleValuesParser::new([on_label, off_label]).map(move |val| val == on_label),
    )
}
