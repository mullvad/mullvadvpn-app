use duct::Expression;
use std::ffi::{OsStr, OsString};

pub trait RunExpr: Sized {
    fn run_expr(self) -> ::std::io::Result<()>;
}


impl RunExpr for Expression {
    fn run_expr(self) -> ::std::io::Result<()> {
        self.run().map(|_| ())
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

    pub fn to_expr(self) -> Expression {
        duct::cmd(self.cmd, self.args)
    }
}
