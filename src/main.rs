extern crate dirs;
extern crate indicatif;

use std::{env, path::PathBuf, process::exit, string::String};

use config::deserialize_file;
use config::ensure_config_dir;
use config::Config;

mod config;
mod hmerror;
mod mgmt;
mod util;

fn main() {
  let args: Vec<String> = env::args().collect();
  /*
    accept either a config passed as arg 1 or try to open the default config location
  */
  let a: Config = match args.get(1) {
    Some(second) => match deserialize_file(&second) {
      Ok(c) => c,
      Err(e) => {
        hmerror::error(
          format!("Couldn't open specified config file `{}`", &second),
          e.as_str(),
        );
        exit(1)
      }
    },
    None => {
      let _p: PathBuf = ensure_config_dir()
        .map_err(|e| {
          hmerror::error(String::from("Couldn't ensure config dir: {}"), e);
          exit(1);
        })
        .unwrap();
      match deserialize_file(_p.to_str().unwrap()) {
        Ok(c) => c,
        Err(e) => {
          hmerror::error(
            format!(
              "Couldn't open assumed (unspecified) config file {}",
              _p.to_string_lossy()
            ),
            e.as_str(),
          );
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
