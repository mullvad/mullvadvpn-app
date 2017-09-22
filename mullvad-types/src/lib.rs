//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

extern crate chrono;
#[macro_use]
extern crate serde_derive;

extern crate talpid_types;

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

pub mod account;
pub mod location;
pub mod states;
pub mod relay_endpoint;
