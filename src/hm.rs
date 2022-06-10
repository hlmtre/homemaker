//! hm is a commandline program to help with dotfile (and more) management.
//!
//! It can handle putting configuration files where they should go, but its real
//! strength lies in the solutions it'll execute - shell scripts, usually - to
//! pull a git repository, compile something from source, etc.
//!
//! hm exists because I bring along a few utilities to all Linux boxes I regularly
//! use, and those are often built from source. So rather than manually install
//! all dependency libraries, then build each dependent piece, then finally the
//! top-level dependent program, I built hm.

//!
//!
//! have a dotfiles directory with all your stuff in it? have homemaker put everything in its right place.
//!
//! [gfycat of it in action](https://gfycat.com/skinnywarmheartedafricanpiedkingfisher)
//!
//!
//!  1. create a config.toml file either anywhere or in ~/.config/homemaker/.
//!  2. enter things to do things to in the file.
//!  example:
//!  ``` text
//!  ## config.toml
//!
//!  [[obj]]
//!  file = 'tmux.conf'
//!  source = '~/dotfiles/.tmux.conf'
//!  destination = '~/.tmux.conf'
//!  method = 'symlink'
//!
//!  [[obj]]
//!  task = 'zt'
//!  solution = 'cd ~/dotfiles/zt && git pull'
//!  dependencies = 'maim, slop'
//!
//!  [[obj]]
//!  task = 'slop'
//!  source = '~/dotfiles/zt/slop'
//!  solution = 'cd ~/dotfiles/zt/slop; make clean; cmake -DCMAKE_INSTALL_PREFIX="/usr" ./ && make && sudo make install'
//!  method = 'execute'
//!  ```
//!  3. `hm ~/path/to/your/config.toml`
//!
//!  [![built with spacemacs](https://cdn.rawgit.com/syl20bnr/spacemacs/442d025779da2f62fc86c2082703697714db6514/assets/spacemacs-badge.svg)](http://spacemacs.org)
//!
//!  thanks to actual good code:
//!  serde
//!  toml
//!  symlink
//!  solvent
//!  indicatif
//!  console

use ::hm::{
  config::{deserialize_file, ensure_config_dir, Config},
  do_tasks, hmerror,
};
use chrono::prelude::*;
use indicatif::HumanDuration;
use log::{info, warn};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::{env, fs::File, path::PathBuf, process::exit, string::String, time::Instant};

/// Pull apart our arguments, if they're called, get our Config, and error-check.
/// Then work our way through the Config, executing the easy stuff, and threading off the hard.
fn main() {
  let l = Local::now();
  let mut slc = ConfigBuilder::new();
  let _ = slc.set_time_offset_to_local();
  let mut p = "./logs/".to_string();
  let mut log_file_name = String::from("hm-task-");
  // we don't really care if we can make the directory. if we can, great.
  match std::fs::create_dir("./logs/") {
    Ok(_) => {}
    Err(e) => {
      warn!("Couldn't create log directory :( . Error: {}", e);
    }
  };
  log_file_name.push_str(l.to_string().as_str());
  log_file_name.push_str(".log");
  p.push_str(log_file_name.as_str());
  // nifty thing is we can make it here, and then we _never_
  // have to pass it around - singleton. just info!, trace!, warn!, etc
  let _ = WriteLogger::init(LevelFilter::Trace, slc.build(), File::create(p).unwrap());
  info!("beginning hm execution...");
  let args: Vec<String> = env::args().collect();
  // it's a little hackish, but we don't have to bring in an external crate to do our args
  let mut target_task: Option<String> = None;
  let mut arg_config: Option<String> = None;
  for i in 0..args.len() {
    match args[i].as_str() {
      "-t" | "--task" => {
        if args.len() > i && !args[i + 1].starts_with('-') {
          // ensure the next arg is not a flag
          // assume the next one is the named task to complete
          target_task = Some(args[i + 1].clone());
        } else {
          hmerror::error(
            "-t flag requires specified task immediately after",
            "No task was specified.",
          );
          help();
        }
      }
      "clean" => {
        match clean() {
          Ok(_) => {
            exit(0);
          }
          Err(e) => {
            eprintln!("{}", e);
            exit(0);
          }
        };
      }
      "-c" | "--config" if args.len() > i + 1 => {
        arg_config = Some(args[i + 1].clone());
      }
      "-h" | "--help" => {
        help();
      }
      _ => {}
    }
  }
  /*
  accept either a config passed in specifically (with -c or --config) or try to open the default config location
   */
  let a: Config = match arg_config {
    Some(second) => match deserialize_file(&second) {
      Ok(c) => c,
      Err(e) => {
        hmerror::error(
          format!("Couldn't open specified config file `{}`", &second).as_str(),
          e.to_string().as_str(),
        );
        exit(1)
      }
    },
    None => {
      let _p: PathBuf = ensure_config_dir()
        .map_err(|e| {
          hmerror::error("Couldn't ensure config dir: {}", e);
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
            )
            .as_str(),
            e.to_string().as_str(),
          );
          exit(1)
        }
      }
    }
  };
  // do it here
  let started = Instant::now();
  match do_tasks(Config::as_managed_objects(a), target_task) {
    Ok(_) => {
      println!("Done in {}.", HumanDuration(started.elapsed()));
      exit(0);
    }
    Err(e) => {
      hmerror::error(format!("{}", e).as_str(), "poooooop");
      exit(3);
    }
  }
}

/// Clean up our logs directory.
fn clean() -> std::io::Result<()> {
  std::fs::remove_dir_all("./logs/")?;
  Ok(())
}

/// Print help for the user.
fn help() {
  println!(
    "usage:
    hm [-h] | [-t|--task] [<task>] | --clean | [-c|--config] [<config>]
    -t | --task             > run specific named task
    -h | --help             > this help message
    --clean                 > removes the contents of the log directory
    -c | --config [config]  > Optional.
    if config is not specified, default location of ~/.config/homemaker/config.toml is assumed."
  );
  exit(0)
}
