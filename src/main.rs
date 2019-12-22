extern crate serde;
extern crate serde_json;
extern crate dirs;

//use std::io::File;
use std::path::Path;
use std::fs;

fn main() {
  ensure_config_dir();
}

fn ensure_config_dir() {
  let conf_dir = dirs::config_dir();
  let mut _a = match conf_dir {
    Some(p) => { // if something
      match fs::create_dir_all(p.join(Path::new("homemaker"))) {
        Ok(r) => r, // if no problem, return the thing in the thing
        Err(err) => panic!("Couldn't create {}. Error: {}", p.display(), err.to_string()),
      };
    },
    // if dirs:: call doesn't return anything
    None => panic!("No $HOME/.config found!"),
  };
}
