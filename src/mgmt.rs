extern crate shellexpand;

use crate::{config::ManagedObject, hmerror::HomemakerError};

use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::os::unix::fs;
use std::path::Path;
use std::{
  process::{Command, Stdio},
  thread,
};

use termion::color;

type Result<T> = std::result::Result<T, HomemakerError>;

fn symlink_file(source: String, target: String) -> Result<()> {
  fs::symlink(
    Path::new(shellexpand::tilde(&source).to_mut()),
    Path::new(shellexpand::tilde(&target).to_mut()),
  )?;
  Ok(())
}

fn execute_solution(solution: String) -> Result<()> {
  // marginally adapted but mostly stolen from
  // https://rust-lang-nursery.github.io/rust-cookbook/os/external.html

  let child: thread::JoinHandle<Result<()>> = thread::spawn(move || {
    let output = Command::new("bash")
      .arg("-c")
      .arg(solution)
      .stdout(Stdio::piped())
      .spawn()?
      .stdout
      .ok_or_else(|| Error::new(ErrorKind::Other, "Couldn't capture stdout"))?;
    let reader = BufReader::new(output);
    // reset to white from whatever was before (green or red)
    println!("{}", color::Fg(color::Reset));
    reader
      .lines()
      .filter_map(|line| line.ok())
      .for_each(|line| println!("{}", line));
    Ok(())
  });
  child.join().unwrap()
}

pub fn perform_operation_on(mo: ManagedObject) -> Result<()> {
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
      let cmd: String = mo.solution;
      println!(
        "{}Executing `{}` for task `{}`",
        color::Fg(color::Green),
        cmd,
        mo.name.to_owned()
      );
      return execute_solution(cmd);
    }
    _ => {
      println!("{}", _s);
      return Ok(());
    }
  }
}
