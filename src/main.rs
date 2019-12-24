//extern crate serde;
//extern crate serde_yaml;
extern crate dirs;
extern crate toml;

//use std::io::File;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::io;
use std::io::Read;
use std::string::String;

struct ManagedObject {
  source: PathBuf,
  destination: PathBuf,
  method: String,
}

fn main() {
  let args: Vec<String> = env::args().collect();
  println!("{:?}", args);
  match args.get(2) {
    Some(second) => parse_config(get_config(PathBuf::from(&second)).ok().unwrap()),
    None => parse_config(get_config(ensure_config_dir().ok().unwrap()).ok().unwrap())
  };
}

fn ensure_config_dir() -> Result<PathBuf, &'static str> {
  let conf_dir = dirs::config_dir();
  let mut _a = match conf_dir {
    Some(p) => { // if something
      let whole_path = p.join(Path::new("homemaker"));
      let _r = fs::create_dir_all(&whole_path);
      match _r {
        Ok(()) => return Ok(PathBuf::from(&whole_path)),
        Err(e) => return Err("Couldn't create config dir!")
      }
    },
    // if dirs::config_path() call doesn't return anything
    None => return Err("Couldn't get config directory from $XDG"),
  };
}

fn get_config(config_file_path: PathBuf) -> Result<std::fs::File, io::Error> {
  let file_handle = fs::File::open(config_file_path)?;
  Ok(file_handle)
}

fn parse_config(mut file_handle: fs::File) -> Result<Vec<ManagedObject>, String> {
  let mut contents = String::new();
  let _a = match file_handle.read_to_string(&mut contents) {
    Ok(_r) => { 
      let v: Vec<ManagedObject> = Vec::new();
      return Ok(v);
    },
    Err(e) => return Err(e.to_string()),
  };
}
