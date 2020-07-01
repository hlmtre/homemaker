use std::fmt;

#[derive(Debug)]
pub enum HMError {
  Regular(ErrorKind),
  Custom(String)
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
  DependencyUndefined,
  SolutionError,
  ConfigError
}

impl ErrorKind {
  fn as_str(&self) -> &str {
    match *self {
      ErrorKind::ConfigError => "configuration error",
      ErrorKind::SolutionError => "solution error",
      ErrorKind::DependencyUndefined => "dependency undefined"
    }
  }
}

//impl Error for HMError {
//  fn description(&self) -> &str {
//    match *self {
//      HMError::Regular(ref err) => err.as_str(),
//      HMError::Custom(ref err) => err
//    }
//  }
//}

impl fmt::Display for HMError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      HMError::Regular(ref err) => write!(f, "A regular error occurred {:?}", err),
      HMError::Custom(ref err) => write!(f, "A custom error occurred {:?}", err),
    }
  }
}

pub type Result<T> = std::result::Result<T, HMError>;
