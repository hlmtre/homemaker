extern crate dirs;
extern crate serde;
extern crate toml;

use serde::{Deserialize, Serialize, Deserializer};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::string::String;
use std::fmt;
use toml::value;

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "file", deserialize_with = "deserialize_files")]
    files: Vec<(String, value::Value)>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            files: Vec::new(),
        }
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

#[derive(Serialize, Deserialize)]
struct ManagedObject<'a> {
  source: &'a str,
  destination: &'a str,
  method: &'a str,
}

impl fmt::Display for ManagedObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "source: {}, destination: {}, method: {}", self.source, self.destination, self.method)
    }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let mut co = String::new();
  let a = match args.get(1) {
    Some(second) => parse_config(get_config(PathBuf::from(&second)).ok().unwrap(), &mut co),
    None => parse_config(get_config(ensure_config_dir().ok().unwrap()).ok().unwrap(), &mut co),
  };
    let c = a.ok(); // get the Config out
    println!("config: {}", c.unwrap());
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
  Ok(file_handle)
}
