extern crate dirs;
extern crate serde;
extern crate toml;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::string::String;

#[derive(Serialize, Deserialize)]
struct Config<'a> {
  objects: BTreeMap<&'a str, ManagedObject<'a>>,
}

#[derive(Serialize, Deserialize)]
struct ManagedObject<'a> {
  source: String,
  destination: String,
  method: String,
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let a = match args.get(1) {
    Some(second) => parse_config(get_config(PathBuf::from(&second)).ok().unwrap()),
    None => parse_config(get_config(ensure_config_dir().ok().unwrap()).ok().unwrap()),
  };
  //  let c = a.ok(); // get the Config out
  //  println!("config: {}", a);
  //  loop {
  //    match c.objects.iter().next() {
  //      Some(x) => {
  //        println!("{}", x.source);
  //      },
  //      None => { break }
  //    }
  //  }
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

fn get_config(config_file_path: PathBuf) -> Result<std::fs::File, io::Error> {
  let file_handle = fs::File::open(&config_file_path)?;
  println!("file: {}", &config_file_path.to_str().unwrap());
  Ok(file_handle)
}

fn parse_config(mut file_handle: fs::File) -> Result<Config, String> {
  let mut contents = String::new();
  match file_handle.read_to_string(&mut contents) {
    Ok(_a) => &contents,
    Err(_e) => return Err(String::from("Couldn't read file contents!")),
  };
  //  let t = r#"title = 'pls'
  //
  //            [tmux.conf]
  //            source = '~/dotfiles/.tmux.conf'
  //            destination = '~'
  //            method = 'symlink'
  //
  //            [fish.fish]
  //            source = '~/dotfiles/fish.fish'
  //            destination = '~/.config/fish/fish.fish'
  //            method = 'symlink'"#;
  let c: Config = match toml::from_str(contents.as_str()) {
    Ok(c) => c,
    Err(e) => {
      println!("error: {}", e.to_string());
      return Err(e.to_string());
    }
  };
  Ok(c)
}
