//extern crate serde;
//extern crate serde_yaml;
extern crate dirs;
extern crate toml;

//use std::io::File;
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::io;

struct ManagedObject {
  source: PathBuf,
  destination: PathBuf,
  method: String,
}

fn main() {
  let args: Vec<String> = env::args().collect();
  println!("{:?}", args);
  match args.get(2) {
    Some(second) => get_config(PathBuf::from(&second)),
    None => get_config(ensure_config_dir())
  };
}

fn ensure_config_dir() -> Result<PathBuf, io::Error> {
  let conf_dir = dirs::config_dir();
  let mut _a = match conf_dir {
    Some(p) => { // if something
      match fs::create_dir_all(p.join(Path::new("homemaker"))) {
        Ok(PathBuf::from(p.join(Path::new("homemaker"))))
      };
    },
    // if dirs::config_path() call doesn't return anything
    None => return (),
  };
}

fn get_config(config_file_path: PathBuf) -> Result<std::fs::File, io::Error> {
  let file_handle = fs::File::open(config_file_path)?;
  Ok(file_handle)
}

fn parse_config(file_handle: fs::File) -> Result<Vec<ManagedObject>, io::Error> {
  Ok(Vec::new())
}
