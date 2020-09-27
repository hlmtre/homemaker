extern crate crossterm;
extern crate dirs;
extern crate indicatif;

use std::{
  env, fs,
  io::{stdout, Write},
  path::{Path, PathBuf},
  process::exit,
  result::Result,
  string::String,
};

use crossterm::{
  execute,
  style::{Color, ResetColor, SetForegroundColor},
};

mod config;
mod hmerror;
mod mgmt;
mod util;

fn main() {
  let args: Vec<String> = env::args().collect();
  /*
    accept either a config passed as arg 1 or try to open the default config location
  */
  let a: config::Config = match args.get(1) {
    Some(second) => match config::deserialize_file(&second) {
      Ok(c) => c,
      Err(e) => {
        let _ = execute!(stdout(), SetForegroundColor(Color::Red));
        eprintln!(
          "Couldn't open specified config file {}. Error: {}",
          &second, e
        );
        let _ = execute!(stdout(), ResetColor);
        exit(1)
      }
    },
    None => {
      let _p: PathBuf = ensure_config_dir()
        .map_err(|e| {
          let _ = execute!(stdout(), SetForegroundColor(Color::Red));
          panic!("Couldn't ensure config dir: {}", e);
        })
        .unwrap();
      match config::deserialize_file(_p.to_str().unwrap()) {
        Ok(c) => c,
        Err(e) => {
          let _ = execute!(stdout(), SetForegroundColor(Color::Red));
          eprintln!(
            "Couldn't open assumed config file {}. Error: {}",
            _p.to_string_lossy(),
            e
          );
          let _ = execute!(stdout(), ResetColor);
          exit(1)
        }
      }
    }
  };
  // do it here
  util::do_tasks(config::as_managed_objects(a));
  println!("doneskies.");
  exit(0);
}

fn ensure_config_dir() -> Result<PathBuf, &'static str> {
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
