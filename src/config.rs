extern crate serde;
extern crate toml;

use crate::hmerror::HMError;

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::{
  fmt, fs, io,
  io::{prelude::*, BufReader},
  string::String,
};
use toml::value;

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
}

impl fmt::Display for ManagedObject {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} {} {} {} {} {} {} {}",
      self.name,
      self.file,
      self.source,
      self.method,
      self.destination,
      self.task,
      self.solution,
      self.satisfied
    )
  }
}

impl Default for ManagedObject {
  fn default() -> Self {
    ManagedObject {
      name: String::from(""),
      source: String::from(""),
      destination: String::from(""),
      method: String::from(""),
      task: String::from(""),
      file: String::from(""),
      solution: String::from(""),
      dependencies: Vec::new(),
      satisfied: false,
    }
  }
}

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

/*
  this is all such terrible rust please don't look at it
*/
impl fmt::Display for Config {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut mos: Vec<ManagedObject> = Vec::new();
    for _f in self.files.iter() {
      let mut mo = ManagedObject::default();
      mo.name = _f.0.to_owned();
      match _f.1.get("file") {
        None => (),
        Some(_x) => {
          mo.file = String::from(_x.as_str().unwrap());
        }
      }
      match _f.1.get("source") {
        None => (),
        Some(_x) => {
          mo.source = String::from(_x.as_str().unwrap());
        }
      }
      match _f.1.get("method") {
        None => (),
        Some(_x) => {
          mo.method = String::from(_x.as_str().unwrap());
        }
      }
      match _f.1.get("destination") {
        None => (),
        Some(_x) => {
          mo.destination = String::from(_x.as_str().unwrap());
        }
      }
      match _f.1.get("solution") {
        None => (),
        Some(_x) => {
          mo.solution = String::from(_x.as_str().unwrap());
        }
      }
      match _f.1.get("task") {
        None => (),
        Some(_x) => {
          mo.task = String::from(_x.as_str().unwrap());
        }
      }
      match _f.1.get("dependencies") {
        None => (),
        Some(_x) => {
          let _f = _x.as_str().unwrap();
          // thanks https://stackoverflow.com/a/37547426
          mo.dependencies = _f.split(", ").map(|s| s.to_string()).collect();
        }
      }
      mos.push(mo);
    }
    write!(f, "{:#?}", mos)
  }
}

pub fn get_mo(n: String) -> Result<ManagedObject, HMError> {
  unimplemented!("not done")
}

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

pub fn as_managed_objects(config: Config) -> HashMap<String, ManagedObject> {
  let mut mos: HashMap<String, ManagedObject> = HashMap::new();
  for _f in config.files.iter() {
    let mut mo = ManagedObject::default();
    mo.name = _f.0.to_owned();
    match _f.1.get("solution") {
      None => (),
      Some(_x) => {
        mo.solution = String::from(_x.as_str().unwrap());
      }
    }
    match _f.1.get("task") {
      None => (),
      Some(_x) => {
        mo.task = String::from(_x.as_str().unwrap());
      }
    }
    match _f.1.get("source") {
      None => (),
      Some(_x) => {
        mo.source = String::from(_x.as_str().unwrap());
      }
    }
    match _f.1.get("method") {
      None => (),
      Some(_x) => {
        mo.method = String::from(_x.as_str().unwrap());
      }
    }
    match _f.1.get("destination") {
      None => (),
      Some(_x) => {
        mo.destination = String::from(_x.as_str().unwrap());
      }
    }
    match _f.1.get("dependencies") {
      None => (),
      Some(_x) => {
        let _f = _x.as_str().unwrap();
        // thanks https://stackoverflow.com/a/37547426
        mo.dependencies = _f.split(", ").map(|s| s.to_string()).collect();
      }
    }
    mos.insert(mo.name.clone(), mo);
  }
  return mos;
}

fn open_config(file: &str) -> io::Result<fs::File> {
  fs::File::open(file)
}

pub fn deserialize_file(file: &str) -> Result<Config, String> {
  let mut contents = String::new();
  let g = match open_config(file) {
    Ok(_a) => _a,
    Err(e) => return Err(e.to_string()),
  };
  let mut file_contents = BufReader::new(g);
  match file_contents.read_to_string(&mut contents) {
    Ok(v) => v,
    Err(_e) => 0,
  };
  if cfg!(debug_assertions) {
    println!("file: {}", &file);
  }
  toml::from_str(&contents).or_else(|e| Err(e.to_string()))
}
