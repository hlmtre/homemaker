use std::fmt;
use std::io;

pub struct HomemakerError {
  line_number: usize,
  complaint: String,
  encapsulated_error: Option<io::Error>,
}

impl fmt::Display for HomemakerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}: {}", self.line_number, self.complaint)
  }
}

impl fmt::Debug for HomemakerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "HomemakerError {{ line_number: {}, complaint: {}, encapsulated_error: {} }}",
      self.line_number,
      self.complaint,
      self.encapsulated_error.as_ref().unwrap().to_string()
    )
  }
}

impl From<io::Error> for HomemakerError {
  fn from(error: io::Error) -> Self {
    HomemakerError {
      line_number: 0,
      complaint: error.to_string(),
      encapsulated_error: Some(error),
    }
  }
}

#[derive(Debug)]
pub enum HMError {
  Io(io::Error),
  Regular(ErrorKind),
  Other(String),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
  DependencyUndefinedError,
  SolutionError,
  ConfigError,
  Other,
}

#[derive(Debug)]
struct ConfigError {
  LineNumber: i32,
  Complaint: String,
}

#[derive(Debug)]
struct SolutionError {
  Complaint: String,
}

#[derive(Debug)]
struct DependencyUndefinedError {
  Dependency: String,
  Dependent: String,
}

impl From<io::Error> for HMError {
  fn from(err: io::Error) -> HMError {
    HMError::Io(err)
  }
}

impl ErrorKind {
  fn as_str(&self) -> &str {
    match *self {
      ErrorKind::ConfigError => "configuration error",
      ErrorKind::SolutionError => "solution error",
      ErrorKind::DependencyUndefinedError => "dependency undefined",
      ErrorKind::Other => "other error",
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
      HMError::Other(ref err) => write!(f, "An error occurred {:?}", err),
      HMError::Io(ref err) => err.fmt(f),
    }
  }
}
