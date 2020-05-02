extern crate dirs;
extern crate serde;
extern crate toml;

//use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::string::String;
use toml::value;

mod mgmt;

#[derive(Serialize, Deserialize)]
struct ManagedObject<'a> {
    source: &'a str,
    destination: &'a str,
    method: &'a str,
}

#[derive(Deserialize, Clone)]
struct Config {
  #[serde(rename = "file", deserialize_with = "deserialize_files")]
  files: Vec<(String, value::Value)>,
}

impl Default for Config {
  fn default() -> Self {
    Config { files: Vec::new() }
  }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hello")
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let a: Config = match args.get(1) {
        Some(second) => deserialize_file(&second).unwrap(),
        None => {
          let _p: PathBuf = ensure_config_dir()
              .map_err(|e| panic!("Couldn't ensure config dir: {}", e)).unwrap();
          deserialize_file(_p.to_str().unwrap()).unwrap()
        },
    };
    let mut counter = 0;
    for element in a.clone().files.iter() {
      println!("{}: {}", counter, element.1);
      counter+=1;
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
          files.push((name.to_owned(), value::Value::Table(entry)));
      }
    }
  }
  Ok(files)
}

/*
let config: Config = deserialize_file(matches.value_of("config").unwrap())?;
*/

fn deserialize_file(file: &str) -> Result<Config, String> {
  let mut contents = String::new();
  println!("file: {}", &file);
  let mut file = BufReader::new(fs::File::open(file).ok().unwrap());
  match file.read_to_string(&mut contents) {
      Ok(v) => v,
      Err(_e) => 0
  };
  toml::from_str(&contents).or_else(|e| Err(e.to_string()))
}

fn ensure_config_dir() -> Result<PathBuf, &'static str> {
  let conf_dir = dirs::config_dir();
  let mut _a = match conf_dir {
    Some(p) => {
      // if something
      // creates a PathBuf from $XDG_CONFIG_DIR
      let whole_path = p.join(Path::new("homemaker"));
      let _r = fs::create_dir_all(&whole_path);
      match _r {
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

fn get_config(config_file_path: PathBuf) -> Result<Config, io::Error> {
  let file_handle = fs::File::open(&config_file_path)?;
  println!("file: {}", &config_file_path.to_str().unwrap());
  let mut contents = String::new();
  Ok(toml::from_str(&contents).unwrap())
}
