extern crate dirs;
extern crate crossterm;

use std::{
  env, fs,
  path::{Path, PathBuf},
  process::exit,
  string::String,
  io::{stdout, Write},
};

use crossterm::{
  execute,
  style::{Color, Colorize, Colored, SetBackgroundColor, SetForegroundColor, ResetColor},
};

mod config;
mod hmerror;
mod mgmt;

fn main() {
  let args: Vec<String> = env::args().collect();
  /*
    accept either a config passed as arg 1 or try to open the default config location
  */
  let a: config::Config = match args.get(1) {
    Some(second) => match config::deserialize_file(&second) {
      Ok(c) => c,
      Err(e) => {
        execute!(stdout(), SetForegroundColor(Color::Red));
        eprintln!(
          "Couldn't open specified config file {}. Error: {}",
          &second,
          e
        );
        execute!(stdout(), ResetColor);
        exit(1)
      }
    },
    None => {
      let _p: PathBuf = ensure_config_dir()
        .map_err(|e| {
          execute!(stdout(), SetForegroundColor(Color::Red));
          panic!("Couldn't ensure config dir: {}", e);
        })
        .unwrap();
      match config::deserialize_file(_p.to_str().unwrap()) {
        Ok(c) => c,
        Err(e) => {
          execute!(stdout(), SetForegroundColor(Color::Red));
          eprintln!(
            "Couldn't open assumed config file {}. Error: {}",
            _p.to_string_lossy(),
            e
          );
          execute!(stdout(), ResetColor);
          exit(1)
        }
      }
    }
  };
  // call worker for objects in Config a here
  //if cfg!(debug_assertions) {
  //  println!("{}", a);
  //}
  #[allow(unused_must_use)]
  for mo in config::as_managed_objects(a) {
    mgmt::perform_operation_on(mo.clone()).map_err(|e| {
      execute!(stdout(), SetForegroundColor(Color::Red));
      eprintln!(
        "Failed to perform operation on {:#?}. \nError: {}\n",
        mo.clone(),
        e
      )
    });
    execute!(stdout(), ResetColor);
  }
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
