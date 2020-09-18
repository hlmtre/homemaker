extern crate crossterm;
extern crate dirs;
extern crate indicatif;
extern crate tokio;

use std::{
  env, fs,
  io::{stdout, Write},
  path::{Path, PathBuf},
  process::exit,
  result::Result,
  string::String,
  thread::{sleep, JoinHandle},
  time,
};

use indicatif::{ProgressBar, ProgressStyle};

use crossterm::{
  execute,
  style::{Color, ResetColor, SetForegroundColor},
};

mod config;
mod hmerror;
mod mgmt;

#[tokio::main]
async fn main() {
  let args: Vec<String> = env::args().collect();
  /*
    accept either a config passed as arg 1 or try to open the default config location
  */
  let a: config::Config = match args.get(1) {
    Some(second) => match config::deserialize_file(&second) {
      Ok(c) => c,
      Err(e) => {
        // if the platform doesn't support color, just don't set it
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
  let a = config::as_managed_objects(a);
  let mut complex_operations = a.clone();
  let mut simple_operations = a.clone();
  complex_operations.retain(|_, v| v.is_task()); // all the things that aren't just symlink/copy
  simple_operations.retain(|_, v| !v.is_task()); // all the things that are quick (don't need to thread off)
  for (_name, _mo) in simple_operations.into_iter() {
    let _ = mgmt::perform_operation_on(_mo).map_err(|e| {
      let _ = execute!(stdout(), SetForegroundColor(Color::Red));
      eprintln!(
        "Failed to perform operation on {:#?}. \nError: {}\n",
        _name, e
      )
    });
    let _ = execute!(stdout(), ResetColor);
  }
  match mgmt::perform_task_batches(complex_operations) {
    Ok(_re) => match _re {
      Some(arr) => {
        for jh in arr {
          let result = jh.await;
          eprintln!("{:#?}", result);
          /*
          eprintln!("hi");
          let p = ProgressBar::new_spinner();
          p.set_style(ProgressStyle::default_spinner());
          p.enable_steady_tick(200);
          p.tick();
          sleep(time::Duration::from_millis(10));
          */
        }
      }
      None => {}
    },
    Err(_) => (),
  }
  // let mut mcl: Vec<JoinHandle<Result<std::process::Child, std::io::Error>>> = Vec::new();
  // for (_name, _mo) in complex_operations.into_iter() {
  //   let _my_child = mgmt::get_task_thread(&_mo).map(|ct| mcl.push(ct));
  // }
  // eprintln!("{:#?}", mcl);
  //let d = mgmt::perform_task_batches(complex_operations).expect("Cyclical dependencies!");
  //eprintln!("{:#?}", d);
  //for (k, v) in c {
  //  for mo in v {
  //  }
  //}
  //println!("{:#?}", b);
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
