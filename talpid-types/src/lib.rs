//! # License
//!
//! Copyright (C) 2017  Amagicom AB
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the
//! GNU General Public License as published by the Free Software Foundation, either version 3 of
//! the License, or (at your option) any later version.

#![deny(rust_2018_idioms)]

use std::{error::Error, fmt};

#[cfg(target_os = "android")]
pub mod android;
pub mod net;
pub mod tunnel;


pub trait ErrorExt {
    /// Creates a string representation of the entire error chain.
    fn display_chain(&self) -> String;

    /// Like [`display_chain`] but with an extra message at the start of the chain
    fn display_chain_with_msg(&self, msg: &str) -> String;
}

impl<E: Error> ErrorExt for E {
    fn display_chain(&self) -> String {
        let mut s = format!("Error: {}", self);
        let mut source = self.source();
        while let Some(error) = source {
            s.push_str(&format!("\nCaused by: {}", error));
            source = error.source();
        }
        s
    }

    fn display_chain_with_msg(&self, msg: &str) -> String {
        let mut s = format!("Error: {}\nCaused by: {}", msg, self);
        let mut source = self.source();
        while let Some(error) = source {
            s.push_str(&format!("\nCaused by: {}", error));
            source = error.source();
        }
        s
    }
}

#[derive(Debug)]
pub struct BoxedError(Box<dyn Error>);

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BoxedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl BoxedError {
    pub fn new(error: impl Error + 'static) -> Self {
        BoxedError(Box::new(error))
    }
}
