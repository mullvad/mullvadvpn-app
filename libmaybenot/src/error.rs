use std::str::Utf8Error;

/// An FFI friendly result type.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MaybenotResult {
    /// Operation completed successfully
    Ok = 0,

    MachineStringNotUtf8 = 1,
    InvalidMachineString = 2,
    StartFramework = 3,
    UnknownMachine = 4,
    NullPointer = 5,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("The machine string wasn't valid UTF-8")]
    MachineStringNotUtf8(#[source] Utf8Error),

    #[error("Failed to parse machine string")]
    InvalidMachineString,

    #[error("Failed to start framework")]
    StartFramework,

    #[error("A machine ID didn't map to a known machine")]
    UnknownMachine,

    #[error("A null pointer was encountered")]
    NullPointer,
}

impl From<Error> for MaybenotResult {
    fn from(error: Error) -> Self {
        match error {
            Error::MachineStringNotUtf8(_) => MaybenotResult::MachineStringNotUtf8,
            Error::InvalidMachineString => MaybenotResult::InvalidMachineString,
            Error::StartFramework => MaybenotResult::StartFramework,
            Error::UnknownMachine => MaybenotResult::UnknownMachine,
            Error::NullPointer => MaybenotResult::NullPointer,
        }
    }
}

impl<T> From<Result<T, Error>> for MaybenotResult {
    fn from(result: Result<T, Error>) -> Self {
        result
            .map(|_| MaybenotResult::Ok)
            .unwrap_or_else(MaybenotResult::from)
    }
}
