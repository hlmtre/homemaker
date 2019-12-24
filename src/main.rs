extern crate serde;
extern crate dirs;
extern crate toml;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::io;
use std::io::Read;
use std::string::String;

#[derive(Serialize, Deserialize)]
struct Config {
  ManagedObject: ManagedObject
}

#[derive(Serialize, Deserialize)]
struct ManagedObject {
  source: String,
  destination: String,
  method: String,
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let _a = match args.get(1) {
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
        Err(_e) => return Err("Couldn't create config dir!")
      }
    },
    // if dirs::config_path() call doesn't return anything
    None => return Err("Couldn't get config directory from $XDG"),
  };
}

fn get_config(config_file_path: PathBuf) -> Result<std::fs::File, io::Error> {
  let file_handle = fs::File::open(&config_file_path)?;
  println!("file: {}", &config_file_path.to_str().unwrap());
  Ok(file_handle)
}

fn parse_config(mut file_handle: fs::File) -> Result<Config, String> {
  let mut contents = String::new();
  //let mut v: Vec<ManagedObject> = Vec::new();
  //let _f = r#"
  //[ManagedObject]
  //source = '~/.dotfiles/.tmux.conf'
  //destination = '~'
  //method = 'symlink'"#;
  let _a = match file_handle.read_to_string(&mut contents) {
    Ok(_r) => { 
      //println!("contents: \n{}", &contents);
      let c: Config = toml::from_str(contents.as_str()).ok().unwrap();
      return Ok(c);
    },
    Err(e) => return Err(e.to_string()),
  };
}
