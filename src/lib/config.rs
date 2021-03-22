//! Define our `Config` and `Worker`.
//! Implement how to take the `config.toml` and turn it into a `Config { files: Vec<ManagedObject> }`.
//! Describes the `Worker` object, which is how we communicate back to our `main()` thread
//! about how our `task` is going.
extern crate serde;
extern crate strum_macros;
extern crate toml;

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::{
  fmt, fs,
  io::{self, prelude::*, BufReader},
  path::{Path, PathBuf},
  string::String,
};
//use strum;
use strum_macros::EnumString;
use toml::value;

use super::hmerror::{HMError, Result as HMResult};

///
/// Allow us to communicate meaningfully back to `main()` thread.
///
#[derive(Debug, Clone, Hash)]
pub struct Worker {
  pub name: String,
  pub status: Option<i32>,
  pub completed: bool,
}

impl<'a> Worker {
  // i'll get to you
  #[allow(dead_code)]
  pub fn new() -> Worker {
    Worker {
      name: String::from(""),
      status: Some(1),
      completed: false,
    }
  }
}
impl PartialEq for Worker {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
  }
}
impl Eq for Worker {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Ord, Hash, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum OS {
  Windows,
  Unknown,
  Linux(LinuxDistro),
}

impl Default for OS {
  fn default() -> Self {
    OS::Unknown
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Ord, Hash, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LinuxDistro {
  Fedora,
  Debian,
  Ubuntu,
  Arch,
  Generic,
}

impl Default for LinuxDistro {
  fn default() -> Self {
    LinuxDistro::Generic
  }
}

impl Eq for OS {}

impl Eq for LinuxDistro {}

///
/// Windows or Linux? If Linux, let's determine our distro, because package managers and stuff.
///
pub fn determine_os() -> OS {
  match sys_info::os_type() {
    Ok(s) => match s.to_ascii_lowercase().as_str() {
      "linux" => match sys_info::linux_os_release() {
        Ok(l) => {
          let a: String = l.name.unwrap().to_ascii_lowercase();
          if a.contains("fedora") {
            OS::Linux(LinuxDistro::Fedora)
          } else if a.contains("debian") {
            OS::Linux(LinuxDistro::Debian)
          } else if a.contains("ubuntu") {
            OS::Linux(LinuxDistro::Ubuntu)
          } else {
            OS::Unknown
          }
        }
        Err(_e) => OS::Unknown,
      },
      "windows" => OS::Windows,
      _ => OS::Unknown,
    },
    Err(_e) => OS::Unknown,
  }
}

/// We're a super-set of all the kinds of `ManagedObject`s we can be.
/// Just don't use the fields you don't wanna use.
/// A simple `ManagedObject` is a name, source, destination, and method (currently only symlink).
/// The simple `ManagedObject` would just be symlinked to its destination.
/// Complex `ManagedObject`s include solutions, which are executed scripts.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManagedObject {
  pub name: String,
  pub source: String,
  pub file: String,
  pub destination: String,
  pub method: String,
  pub task: String,
  pub solution: String,
  pub dependencies: Vec<String>,
  pub satisfied: bool,
  pub os: Option<OS>,
  pub force: bool,
  pub post: String,
}

impl ManagedObject {
  /// quite simply, if we're a task, we'll have a `solution`.
  pub fn is_task(&self) -> bool {
    return !self.solution.is_empty();
  }
  pub fn set_satisfied(&mut self) -> () {
    self.satisfied = true;
  }
}
impl PartialEq for ManagedObject {
  fn eq(&self, other: &Self) -> bool {
    if self.name == other.name
      && self.source == other.source
      && self.destination == other.destination
      && self.task == other.task
      && self.solution == other.solution
    {
      return true;
    }
    return false;
  }
}

impl fmt::Display for ManagedObject {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} {} {} {} {} {} {} {} {:?} {:?} {}",
      self.name,
      self.file,
      self.source,
      self.method,
      self.destination,
      self.task,
      self.solution,
      self.satisfied,
      self.os,
      self.force,
      self.post
    )
  }
}

impl Default for ManagedObject {
  fn default() -> Self {
    ManagedObject {
      name: String::from(""),
      source: String::from(""),
      file: String::from(""),
      destination: String::from(""),
      method: String::from(""),
      task: String::from(""),
      solution: String::from(""),
      dependencies: Vec::new(),
      satisfied: false,
      os: None,
      force: false,
      post: "".to_string(),
    }
  }
}

/// Represents just the file `config.toml` and contains a vector of things
/// that shall become `ManagedObject`s.
#[derive(Deserialize, Clone)]
pub struct Config {
  #[serde(rename = "obj", deserialize_with = "deserialize_files")]
  pub files: Vec<(String, value::Value)>,
}

impl Default for Config {
  fn default() -> Self {
    Config { files: Vec::new() }
  }
}

impl fmt::Display for Config {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mos = Config::as_managed_objects(self.clone());
    write!(f, "{:#?}", mos)
  }
}

/// Represents our Config file, which is just a big list of `[[obj]]`s
impl Config {
  /// Allows us to get a specified Managed Object by name
  #[allow(dead_code)]
  pub fn get_mo(&mut self, _n: &str) -> Option<ManagedObject> {
    match Config::as_managed_objects(self.clone()).get(_n) {
      Some(a) => Some(a.to_owned()),
      None => None,
    }
  }

  /// Convenience function that allows getting a HashMap from a `Config` of
  /// the `ManagedObject`s within.
  pub fn as_managed_objects(config: Config) -> HashMap<String, ManagedObject> {
    config
      .files
      .iter()
      .map(|(name, val)| {
        let mut mo = ManagedObject::default();
        mo.name = name.to_owned();
        if let Some(_x) = val.get("solution") {
          mo.solution = String::from(_x.as_str().unwrap());
        }
        if let Some(_x) = val.get("task") {
          mo.task = String::from(_x.as_str().unwrap());
        }
        if let Some(_x) = val.get("source") {
          mo.source = String::from(_x.as_str().unwrap());
        }
        if let Some(_x) = val.get("method") {
          mo.method = String::from(_x.as_str().unwrap());
        }
        if let Some(_x) = val.get("destination") {
          mo.destination = String::from(_x.as_str().unwrap());
        }
        if let Some(_x) = val.get("dependencies") {
          let _f = _x.as_array().unwrap();
          mo.dependencies = _f.iter().map(|v| v.as_str().unwrap().to_owned()).collect();
        }
        if let Some(_x) = val.get("force") {
          // haha boolean assignment go brr
          mo.force = _x.as_bool().unwrap();
        }
        if let Some(_x) = val.get("post") {
          mo.post = String::from(_x.as_str().unwrap());
        }
        //
        // the `os =` entry in the config will be formatted either
        // `windows` or `linux::<distro>`
        if let Some(_x) = val.get("os") {
          let _b = String::from(_x.as_str().unwrap());
          let _a: Vec<&str> = _b.split("::").collect::<Vec<&str>>();
          // TODO change this to match on strum's VariantNotFound
          mo.os = if _a.len() > 1 {
            Some(OS::Linux(
              LinuxDistro::from_str(_a[1].to_lowercase().as_str()).unwrap(),
            ))
          } else {
            Some(OS::from_str(_a[0].to_lowercase().as_str()).unwrap())
          };
        } else {
          mo.os = None;
        }
        (mo.name.clone(), mo)
      })
      .collect()
  }
}

/// This takes our file/task array and turns them into `ManagedObjects`,
/// to be stuffed into the `Config`.
pub fn deserialize_files<'de, D>(deserializer: D) -> Result<Vec<(String, value::Value)>, D::Error>
where
  D: Deserializer<'de>,
{
  let mut files: Vec<(String, value::Value)> = Vec::new();
  let raw_files: Vec<value::Table> = Deserialize::deserialize(deserializer)?;
  for mut entry in raw_files {
    if let Some(name) = entry.remove("file") {
      if let Some(name) = name.as_str() {
        files.push((name.to_owned(), value::Value::Table(entry)));
      }
    } else if let Some(name) = entry.remove("task") {
      if let Some(name) = name.as_str() {
        files.push((name.to_owned(), value::Value::Table(entry)));
      }
    }
  }
  Ok(files)
}

/// Take a big, fat guess.
/// Open the specified file. We've already made sure our Path and stuff
/// look good.
fn open_config(file: &str) -> io::Result<fs::File> {
  fs::File::open(file)
}

/// Open our config file and read the entire contents into hopefully
/// valid toml. Either we gucci and return back a `Config` made of toml,
/// or we explain what went wrong with the `toml` Err.
pub fn deserialize_file(file: &str) -> HMResult<Config> {
  let mut contents = String::new();
  let g = match open_config(file) {
    Ok(_a) => _a,
    Err(e) => return Err(HMError::Other(e.to_string())),
  };

  let mut file_contents = BufReader::new(g);
  match file_contents.read_to_string(&mut contents) {
    Ok(v) => v,
    Err(_e) => 0,
  };
  if cfg!(debug_assertions) {
    println!("file: {}", &file);
  }
  deserialize_str(&contents)
}

fn deserialize_str(contents: &str) -> HMResult<Config> {
  toml::from_str(contents).or_else(|e| Err(HMError::Other(e.to_string())))
}

/// Make sure $XDG_CONFIG_DIR exists.
/// On Linux and similar this is /home/\<username\>/.config;
/// macOS /Users/\<username\>/.config,
/// Windows C:\\Users\\\<username\>\\.config
///
/// Assuming it does exist or we can create it, stuff config.toml on the end of it
/// and return `Ok(my_path_buf/config.toml)`.
///
/// This is safe because `ensure_config_dir()` is called only after we already know
/// the user didn't specify a `config.toml` path themselves. We must check our
/// default expected location for it.
pub fn ensure_config_dir() -> Result<PathBuf, &'static str> {
  // get /home/<username>/.config, if exists...
  match dirs::config_dir() {
    Some(p) => {
      // if something
      // creates a PathBuf from $XDG_CONFIG_DIR
      let whole_path = p.join(Path::new("homemaker"));
      match fs::create_dir_all(&whole_path) {
        /*
        then when we return it, do the entire config dir path (/home/hlmtre/.config)
        and add our config file to the end of the PathBuf
        ```
        Ok(("/home/hlmtre/.config").join("config.toml"))
        ```
        sort of as a pseudocodey example
         */
        Ok(()) => return Ok(PathBuf::from(&whole_path.join("config.toml"))),
        Err(_e) => return Err("Couldn't create config dir!"),
      }
    }
    // if dirs::config_path() call doesn't return anything
    None => return Err("Couldn't get config directory from $XDG"),
  };
}

#[cfg(test)]
mod config_test {
  use super::*;

  /*
    source looks like this:
    [[obj]]
    file = 'tmux.conf'
    source = '~/dotfiles/.tmux.conf'
    destination = '~/.tmux.conf'
    method = 'symlink'
  */
  #[test]
  fn test_mo_deserialization() {
    let mut a: Config = deserialize_file("./benches/config.toml").unwrap();
    let mut mo = ManagedObject::default();
    mo.name = String::from("tmux.conf");
    mo.source = String::from("~/dotfiles/.tmux.conf");
    mo.destination = String::from("~/.tmux.conf");
    mo.method = String::from("symlink");
    assert_eq!(mo, a.get_mo("tmux.conf").unwrap());
  }

  #[test]
  fn dependencies_is_array() {
    let mut a: Config = deserialize_str(
      r#"
[[obj]]
task = 'zt'
solution = 'cd ~/dotfiles/zt && git pull'
dependencies = ['grim', 'slurp']
    "#,
    )
    .unwrap();
    assert_eq!(vec!["grim", "slurp"], a.get_mo("zt").unwrap().dependencies);
  }
}
