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
  hmerror,
};
use chrono::prelude::*;
use hm::app::APP;
use hm::task::do_tasks;
use indicatif::HumanDuration;
use log::{info, warn};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::io;
use std::{env, fs::File, path::PathBuf, process::exit, string::String, time::Instant};
use tui::{backend::CrosstermBackend, layout::Alignment, style::Style, widgets::Paragraph};
use tui::{style::Modifier, Terminal};
use tui::{
  text::Span,
  widgets::{Block, Borders, Wrap},
};

use tui::layout::{Constraint, Direction, Layout};

/// Pull apart our arguments, if they're called, get our Config, and error-check.
/// Then work our way through the Config, executing the easy stuff, and threading off the hard.
fn main() {
  let l = Local::now();
  let mut slc = ConfigBuilder::new();
  slc.set_time_to_local(true);
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
  /*
   let mut a = APP.write().unwrap();
   a.append_summary("beginning hm execution...".to_string());
  */
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
  let config: Config = match arg_config {
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

  // tui creation
  // you'll probably have to change all your prints
  // to print INSIDE the box
  // you'll be showing progress of the tasks on the left,
  // and task stdout output on the right in (presumably larger) box
  let stdout = io::stdout();
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend).unwrap();

  loop {
    let _ = terminal.draw(|f| {
      let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
          [
            Constraint::Percentage(40),
            Constraint::Percentage(50),
            Constraint::Percentage(10),
          ]
          .as_ref(),
        )
        .split(f.size());
      let create_block = |title| {
        Block::default()
          .borders(Borders::ALL)
          .style(Style::default())
          .title(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
          ))
      };
      let block = Block::default().title("hm").borders(Borders::ALL);
      let mut a = APP.write().unwrap();
      a.append_summary("HELLO THERE GENERAL KENOBI".to_string());
      drop(a);
      let b = APP.read().unwrap();
      let paragraph = Paragraph::new(b.hm_task_summary().get(0).unwrap().clone())
        //let paragraph = Paragraph::new("WAZZZZAPPPP".to_string())
        .style(Style::default())
        .block(create_block("tasks"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
      drop(b);
      f.render_widget(block, chunks[0]);
      let block = Block::default().title("task output").borders(Borders::ALL);
      f.render_widget(paragraph, chunks[0]);
      f.render_widget(block, chunks[1]);
    });
    // end tui
    let started = Instant::now();
    println!("app: {:?}", APP.read().unwrap());
    match do_tasks(Config::as_managed_objects(config), target_task) {
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
