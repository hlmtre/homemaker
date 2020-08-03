extern crate shellexpand;
extern crate symlink;

use crate::{
  config::ManagedObject,
  hmerror::{ErrorKind as hmek, HMError, HomemakerError},
};

use std::fs::metadata;
use std::io::{stdout, BufRead, BufReader, Error, ErrorKind, Write};
use std::path::Path;
use std::{
  process::{Command, ExitStatus, Stdio},
  thread,
};

use crossterm::{
  execute,
  style::{Color, ResetColor, SetForegroundColor},
};

use symlink::{symlink_dir as sd, symlink_file as sf};

fn symlink_file(source: String, target: String) -> Result<(), HMError> {
  let md = match metadata(source.to_owned()) {
    Ok(a) => a,
    Err(e) => return Err(HMError::Io(e)),
  };
  if md.is_dir() {
    sd(
      Path::new(shellexpand::tilde(&source).to_mut()),
      Path::new(shellexpand::tilde(&target).to_mut()),
    )?;
  } else if md.is_file() {
    sf(
      Path::new(shellexpand::tilde(&source).to_mut()),
      Path::new(shellexpand::tilde(&target).to_mut()),
    )?;
  }
  Ok(())
}

fn execute_solution(solution: String) -> Result<(ExitStatus), HMError> {
  // marginally adapted but mostly stolen from
  // https://rust-lang-nursery.github.io/rust-cookbook/os/external.html

  let child: thread::JoinHandle<Result<(), HMError>> = thread::spawn(move || {
    let c = Command::new("bash")
      .arg("-c")
      .arg(solution)
      .stdout(Stdio::piped())
      .spawn()?;
    let output = c.stdout.unwrap();
    let reader = BufReader::new(output);
    reader
      .lines()
      .filter_map(|line| line.ok())
      .for_each(|line| println!("{}", line));
    match c.try_wait() {
      Ok(Some(status)) => return Ok(status),
      Ok(None) => return Ok(-1),
      Err(e) => return e,
    }
  });
  child.join();
}

pub fn perform_operation_on(mo: ManagedObject) -> Result<(), HMError> {
  let _s = mo.method.as_str();
  match _s {
    "symlink" => {
      let source: String = mo.source;
      let destination: String = mo.destination;
      return symlink_file(source, destination);
    }
    "execute" => {
      //      if !mo.dependencies.is_empty() {
      //        for d in mo.dependencies
      //        {
      //
      //        }
      //      }
      //Err(HMError::Regular(hmek::SolutionError))
      let cmd: String = mo.solution;
      let _ = execute!(stdout(), SetForegroundColor(Color::Green));
      println!("Executing `{}` for task `{}`", cmd, mo.name.to_owned());
      let _ = execute!(stdout(), ResetColor);
      return execute_solution(cmd);
    }
    _ => {
      let _ = execute!(stdout(), SetForegroundColor(Color::Green));
      println!("{}", _s);
      let _ = execute!(stdout(), ResetColor);
      return Ok(());
    }
  }
}
