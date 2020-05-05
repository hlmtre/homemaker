extern crate dirs;

use std::env;
use std::fs;
use std::process::exit;
use std::path::{Path, PathBuf};
use std::string::String;

mod mgmt;
mod config;

fn main() {
    let args: Vec<String> = env::args().collect();
    /*
      accept either a config passed as arg 1 or try to open the default config location
    */
    let a: config::Config = match args.get(1) {
        Some(second) => config::deserialize_file(&second).unwrap(),
        None => {
          let _p: PathBuf = ensure_config_dir()
              .map_err(|e| panic!("Couldn't ensure config dir: {}", e)).unwrap();
          match config::deserialize_file(_p.to_str().unwrap()) {
            Ok(c) => c,
            Err(e) => {
              println!("Couldn't open assumed config file {}. Error: {}", _p.to_string_lossy(), e);
              exit(1)
            }
          }
        },
    };
    println!("{}", a);
}

fn ensure_config_dir() -> Result<PathBuf, &'static str> {
  // get /home/<username>/.config, if exists...
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

/*
fn get_config(config_file_path: PathBuf) -> Result<config::config::Config, io::Error> {
  let file_handle = fs::File::open(&config_file_path)?;
  println!("file: {}", &config_file_path.to_str().unwrap());
  let mut contents = String::new();
  Ok(toml::from_str(&contents).unwrap())
}
*/
