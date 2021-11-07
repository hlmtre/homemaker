//! Custom Error implementation to allow for our own kind, along with
//! run-of-the-mill `std::io::Error`s.
//! ```
//! pub enum ErrorKind {
//!   DependencyUndefinedError,
//!   CyclicalDependencyError,
//!   SolutionError,
//!   ConfigError,
//!   Other,
//! }
//! ```
//! * DependencyUndefinedError: A stated dependency doesn't have an object telling us how to satisfy it.
//! * CyclicalDependencyError: a -> b and b -> a and neither is satisfied. The offending object is the tippy-top of the chain.
//! * SolutionError: Something went wrong in our script.
//! * ConfigError: Something is wrong with how you wrote the `config.toml`.
//! * Other: Other.
use console::style;
use std::fmt;
use std::io;

use crate::app;
use crate::config;

#[derive(Debug)]
pub enum HMError {
  Io(io::Error),
  Regular(ErrorKind),
  Other(String),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
  DependencyUndefinedError {
    dependency: String,
  },
  IncorrectPlatformError {
    dependency: String,
    platform: config::OS,
    target_platform: config::OS,
  },
  CyclicalDependencyError {
    dependency_graph: String,
  },
  SolutionError {
    solution: String,
  },
  ConfigError {
    line_number: usize,
  },
  Other,
}

impl From<io::Error> for HMError {
  fn from(err: io::Error) -> HMError {
    HMError::Io(err)
  }
}

/*
impl From<Deserializer::Error> for HMError {
  fn from(err: Deserializer::Error) -> HMError {
    HMError::Other(err)
  }
}
*/

impl ErrorKind {
  fn as_str(&self) -> &str {
    match *self {
      ErrorKind::ConfigError { line_number: _ } => "configuration error",
      ErrorKind::SolutionError { solution: _ } => "solution error",
      ErrorKind::DependencyUndefinedError { dependency: _ } => "dependency undefined",
      ErrorKind::IncorrectPlatformError {
        dependency: _,
        platform: _,
        target_platform: _,
      } => "incorrect platform for dependency",
      ErrorKind::CyclicalDependencyError {
        dependency_graph: _,
      } => "cyclical dependency",
      ErrorKind::Other => "other error",
    }
  }
}

impl fmt::Display for HMError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      HMError::Regular(ref err) => write!(f, "{:?}", err),
      HMError::Other(ref err) => write!(f, "{:?}", err),
      HMError::Io(ref err) => err.fmt(f),
    }
  }
}

pub fn happy_print(jubilation: &str) {
  let mut _a = app::APP.write().unwrap();
  _a.append_output(format!("{}!", style(jubilation).green().bold()));
  drop(_a);
}

/// Easy formatting for errors as they come in.
///
/// example:
///
/// ```
/// use hm::hmerror;
/// let _a = "src/config.toml";
/// let _e = "my_dummy_error";
/// hmerror::error(
///  format!("Couldn't open specified config file `{}`", _a).as_str(),
///  _e,
/// );
/// ```
pub fn error(complaint: &str, er: &str) {
  eprintln!("{}:\n ↳ Error: {}", style(complaint).red().bold(), er)
}

pub type Result<T> = std::result::Result<T, HMError>;
