use std::{
    io,
    process::{Child, Command, Stdio},
    str::FromStr,
};

type Error = ();

pub trait IntoExec: std::fmt::Debug {
    fn into_exec(self) -> Result<Exec, Error>;
}

#[derive(Debug)]
pub enum Exec {
    Command(Command),
    Func(fn() -> io::Result<()>),
}

#[derive(Debug)]
pub enum ExecHandle {
    Command(Child),
    Func,
}

impl Exec {
    pub fn spawn(&mut self) -> io::Result<ExecHandle> {
        match self {
            Self::Command(command) => command.spawn().map(ExecHandle::Command),
            Self::Func(f) => f().map(|_| ExecHandle::Func),
        }
    }
}

impl ExecHandle {
    pub fn id(&self) -> Option<u32> {
        match self {
            Self::Command(child) => Some(child.id()),
            _ => None,
        }
    }
    pub fn try_wait(&mut self) -> io::Result<()> {
        match self {
            Self::Command(child) => child.try_wait().map(drop),
            _ => Ok(()),
        }
    }
}

impl IntoExec for fn() -> io::Result<()> {
    fn into_exec(self) -> Result<Exec, Error> {
        Ok(Exec::Func(self))
    }
}

impl IntoExec for &str {
    fn into_exec(self) -> Result<Exec, Error> {
        FromStr::from_str(self)
    }
}

impl FromStr for Exec {
    type Err = Error;
    fn from_str(cmd: &str) -> Result<Self, Self::Err> {
        let mut args = cmd.split(' ');
        let mut bld: Command = Command::new(args.next().ok_or(())?);
        bld.args(args);
        bld.stdin(Stdio::null());
        bld.stderr(Stdio::null());
        bld.stdout(Stdio::null());
        Ok(Self::Command(bld))
    }
}
