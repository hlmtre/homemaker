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
#![allow(dead_code)]
extern crate dirs;
extern crate indicatif;

use std::{env, path::PathBuf, process::exit, string::String, time::Instant};

use indicatif::HumanDuration;

use config::deserialize_file;
use config::ensure_config_dir;
use config::Config;

mod config;
mod hmerror;
mod lib;
mod util;

/// Pull apart our arguments, if they're called, get our Config, and error-check.
/// Then work our way through the Config, executing the easy stuff, and threading off the hard.
fn main() {
  let mut args: Vec<String> = env::args().collect();
  // it's a little hackish, but we don't have to bring in an external crate to do our args
  let mut verbose: bool = false;
  let mut i = 0;
  for a in args.clone() {
    if a.trim() == "-v" {
      verbose = true;
      break;
    }
    if a.trim() == "-h" {
      help();
      break;
    }
    i += 1;
  }
  if verbose {
    args.remove(i);
  }
  /*
  accept either a config passed as arg 1 or try to open the default config location
   */
  let a: Config = match args.get(1) {
    Some(second) => match deserialize_file(&second) {
      Ok(c) => c,
      Err(e) => {
        hmerror::error(
          format!("Couldn't open specified config file `{}`", &second).as_str(),
          e.as_str(),
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
            e.as_str(),
          );
          exit(1)
        }
      }
    }
  };
  // do it here
  let started = Instant::now();
  util::do_tasks(config::as_managed_objects(a));
  println!("Done in {}.", HumanDuration(started.elapsed()));
  exit(0);
}

/// Print help for the user.
fn help() {
  println!(
    "usage:
    hm [-h] | [-v] [<config>]
    -v | verbose output
    -h | this help message
    <config> and -v are not required.
    if config is not specified, default location of ~/.config/homemaker/config.toml is assumed."
  );
  exit(1)
}
