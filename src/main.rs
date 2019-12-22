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
  let our_folder = Path::new("homemaker");
  let conf_dir = dirs::config_dir();
  let mut _a = match conf_dir {
    Some(p) => {
      match fs::create_dir_all(p.join(our_folder)) {
        Ok(r) => r,
        Err(err) => panic!("Couldn't create {}. Error: {}", p.display(), err.to_string()),
      };
    },
    None    => panic!("No $HOME/.config found!"),
  };
}
