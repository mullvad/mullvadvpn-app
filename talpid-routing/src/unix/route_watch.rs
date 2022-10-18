use std::{ffi::OsString, io, net::IpAddr};
use tokio::{
    io::{AsyncBufReadExt, BufReader, Lines},
    process::{ChildStdout, Command},
};
use tokio_stream::Stream;

#[derive(err_derive::Error, Debug)]
pub enum Error {
    /// Failed to spawn route.
    #[error(display = "Failed to spawn route command")]
    Spawn(#[error(source)] io::Error),

    /// Route subprocess has no stdout pipe.
    #[error(display = "`route monitor -n` subprocess has no stdout pipe")]
    NoStdout,

    /// Unexpected output from `route`.
    #[error(display = "Encountered unexpected output from route: _0")]
    UnexpectedOutput(OsString),

    /// `route monitor` subcommand exited unexpectedly
    #[error(display = "route subcommand exited unexpectedly")]
    UnexpectedShutdown,
}

#[derive(PartialEq, Debug)]
pub enum RouteChange {
    Add(Route),
    Remove(Route),
}

#[derive(PartialEq, Debug)]
pub struct Route {
    netmask: IpAddr,
    destination: IpAddr,
    iflocal: bool,
    ifaddr: Option<IpAddr>,
    interface: Option<String>,
}

pub struct RouteWatcher {
    route_output: Lines<BufReader<ChildStdout>>,
}

impl RouteWatcher {
    pub async fn new() -> Result<Self, Error> {
        let child = Command::new("/sbin/route").spawn().map_err(Error::Spawn)?;
        let route_output = BufReader::new(child.stdout.ok_or(Error::NoStdout)?).lines();

        Ok(Self { route_output })
    }

    pub async fn next(&mut self) -> Result<RouteChange, Error> {
        let mut buffer = vec![];

        while let Ok(Some(line)) = self.route_output.next_line().await {
            let line_empty = line.is_empty();
            if line_empty {
                if buffer.is_empty() {
                    return Self::parse_route(&buffer);
                }
            } else {
                buffer.push(line);
            }
        }

        Err(Error::UnexpectedShutdown)
    }

    fn parse_route(buffer: &[String]) -> Result<RouteChange, Error> {
        unimplemented!()
    }
}

// Gateway strings can be "255", "(255)" "fff fff fff"
fn parse_netmask_v4(input: &str) -> Result<IpAddr, Error> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_delete_message() {}
}
