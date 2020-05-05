extern crate serde;

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::string::String;
use toml::value;

#[derive(Serialize, Deserialize, Debug)]
pub struct ManagedObject {
    source: String,
    destination: String,
    method: String,
}

impl fmt::Display for ManagedObject {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {} {}", self.source, self.method, self.destination)
  }
}

impl Default for ManagedObject {
  fn default() -> Self {
    ManagedObject { source: String::from(""), destination: String::from(""), method: String::from("") }
  }
}

#[derive(Deserialize, Clone)]
pub struct Config {
  #[serde(rename = "file", deserialize_with = "deserialize_files")]
  files: Vec<(String, value::Value)>,
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
      let mut s = String::new();
      for _f in self.files.iter() {
        let mut mo = ManagedObject::default();
        s.push_str("\n");
        s.push_str(&_f.0);
        s.push_str(&": ");
        match _f.1.get("source") {
          None => (),
          Some(_x) =>  {
            s.push_str(_x.as_str().unwrap());
            mo.source = String::from(_x.as_str().unwrap());
          }
        }
        match _f.1.get("method") {
          None => (),
          Some(_x) => {
            if _x.as_str().unwrap() == "symlink" {
              s.push_str(&" => ");
              mo.method = String::from("symlink");
            }
          }
        }
        match _f.1.get("destination") {
          None => (),
          Some(_x) => {
            s.push_str(_x.as_str().unwrap());
            mo.destination = String::from(_x.as_str().unwrap());
          }
        }
        mos.push(mo);
      }
      write!(f, "{:#?}", mos)
    }
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
    }
  }
  Ok(files)
}

/*
let config: Config = deserialize_file(matches.value_of("config").unwrap())?;
*/

pub fn deserialize_file(file: &str) -> Result<Config, String> {
  let mut contents = String::new();
  println!("file: {}", &file);
  let mut file = BufReader::new(fs::File::open(file).ok().unwrap());
  match file.read_to_string(&mut contents) {
    Ok(v) => v,
    Err(_e) => 0
  };
  toml::from_str(&contents).or_else(|e| Err(e.to_string()))
}
