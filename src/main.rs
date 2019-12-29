extern crate dirs;
extern crate serde;
extern crate toml;

use serde::{Deserialize, Deserializer, Serialize};
use serde::de::DeserializeOwned;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::string::String;
use toml::value;
use toml;

#[derive(Deserialize)]
struct Config {
  #[serde(rename = "file", deserialize_with = "deserialize_files")]
  files: Vec<(String, value::Value)>,
}

impl Default for Config {
  fn default() -> Self {
    Config { files: Vec::new() }
  }
}

fn deserialize_files<'de, D>(deserializer: D) -> Result<Vec<(String, value::Value)>, D::Error>
where
  D: Deserializer<'de>,
{
  let mut files: Vec<(String, value::Value)> = Vec::new();
  let raw_files: Vec<value::Table> = Deserialize::deserialize(deserializer)?;
  for mut entry in raw_files {
    if let Some(name) = entry.remove("file") {
      if let Some(name) = name.as_str() {
        files.push((name.to_owned(), value::Value::Table(entry)))
      }
    }
  }

  Ok(files)
}

/*
let config: Config = deserialize_file(matches.value_of("config").unwrap())?;
 */

pub fn deserialize_file<T>(file: &str) -> Result<Config, toml::de::Error>
where
    T: DeserializeOwned,
{
    let mut contents = String::new();
    let mut file = BufReader::new(
        fs::File::open(file),
    );
    file.read_to_string(&mut contents);
    Ok(toml::from_str(&contents))
}

#[derive(Serialize, Deserialize)]
struct ManagedObject<'a> {
  source: &'a str,
  destination: &'a str,
  method: &'a str,
}

impl fmt::Display for Config {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "hello")
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let mut co = String::new();
  let a = match args.get(1) {
    Some(second) => get_config(PathBuf::from(&second)).ok().unwrap(),
    None => get_config(ensure_config_dir().ok().unwrap()).ok().unwrap(),
  };
  //loop {
  //  match c.objects.iter().next() {
  //    Some(x) => {
  //      println!("{}", x.1);
  //    },
  //    None => { break }
  //  }
  //}
}

fn ensure_config_dir() -> Result<PathBuf, &'static str> {
  let conf_dir = dirs::config_dir();
  let mut _a = match conf_dir {
    Some(p) => {
      // if something
      let whole_path = p.join(Path::new("homemaker"));
      let _r = fs::create_dir_all(&whole_path);
      match _r {
        Ok(()) => return Ok(PathBuf::from(&whole_path)),
        Err(_e) => return Err("Couldn't create config dir!"),
      }
    }
    // if dirs::config_path() call doesn't return anything
    None => return Err("Couldn't get config directory from $XDG"),
  };
}

fn get_config(config_file_path: PathBuf) -> Result<Config, io::Error> {
  let file_handle = fs::File::open(&config_file_path)?;
  println!("file: {}", &config_file_path.to_str().unwrap());
  let mut contents = String::new();
  Ok(toml::from_str(&contents).unwrap())
}
