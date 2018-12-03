use duct::Expression;
use std::ffi::{OsStr, OsString};

pub trait RunExpr: Sized {
    fn run_expr(self) -> ::std::io::Result<()>;
    fn stdout(self) -> ::std::io::Result<String>;
}


impl RunExpr for Expression {
    fn run_expr(self) -> ::std::io::Result<()> {
        log::trace!("Executing command - {:?}", self);
        self.run().map(|_| ())
    }

    fn stdout(self) -> ::std::io::Result<String> {
        log::trace!("Executing command - {:?}", self);
        self.stdout_capture()
            .run()
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
    }
}


pub struct Exec {
    cmd: OsString,
    args: Vec<OsString>,
}

impl Exec {
    pub fn cmd<S: AsRef<OsStr>>(cmd: S) -> Exec {
        Exec {
            cmd: cmd.as_ref().to_owned(),
            args: vec![],
        }
    }

    pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Exec {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn into_expr(self) -> Expression {
        duct::cmd(self.cmd, self.args)
    }
}
