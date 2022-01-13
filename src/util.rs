use std::fmt;
use std::error::Error as StdError;
use std::process::ExitStatus;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatusError(Option<i32>);

/// TODO remove it
///
/// see https://github.com/rust-lang/rust/issues/84908
pub trait ExitStatusExt {
    fn exit_ok2(&self) -> Result<(), ExitStatusError>;
}

impl ExitStatusExt for ExitStatus {
    fn exit_ok2(&self) -> Result<(), ExitStatusError> {
        if self.success() {
            Ok(())
        } else {
            Err(ExitStatusError(self.code()))
        }
    }
}

impl fmt::Display for ExitStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl StdError for ExitStatusError {}
