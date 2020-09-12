extern crate shellexpand;
extern crate solvent;
extern crate symlink;

use crate::{
  config::ManagedObject,
  hmerror::{ErrorKind as hmek, HMError, HomemakerError},
};

use std::collections::HashMap;
use std::fs::metadata;
use std::io::{stdout, BufRead, BufReader, Error, ErrorKind, Write};
use std::path::Path;
use std::{
  process::{Command, Stdio},
  thread,
};

use crossterm::{
  execute,
  style::{Color, ResetColor, SetForegroundColor},
};

use symlink::{symlink_dir as sd, symlink_file as sf};

use solvent::DepGraph;

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

/*
  return a list of lists of ManagedObjects,
  whose order guarantees we'll satisfy dependencies first
*/
pub fn perform_task_batches(nodes: HashMap<String, ManagedObject>) -> Result<(), HMError> {
  let mut depgraph: DepGraph<String> = DepGraph::new();
  for (name, node) in nodes.clone() {
    depgraph.register_dependencies(name.to_owned(), node.dependencies.clone());
  }
  for (name, _node) in nodes.clone() {
    for n in depgraph.dependencies_of(&name).unwrap() {
      match n {
        Ok(r) => execute_solution(nodes.get(r).unwrap().solution.clone())?,
        Err(_e) => {
          return Err(HMError::Regular(hmek::CyclicalDependencyError));
        }
      }
    }
  }

  Ok(())
}

fn execute_solution(solution: String) -> Result<(), HMError> {
  // marginally adapted but mostly stolen from
  // https://rust-lang-nursery.github.io/rust-cookbook/os/external.html

  let child: thread::JoinHandle<Result<(), HMError>> = thread::spawn(move || {
    let output = Command::new("bash")
      .arg("-c")
      .arg(solution)
      .stdout(Stdio::piped())
      .spawn()?
      .stdout
      .ok_or_else(|| Error::new(ErrorKind::Other, "Couldn't capture stdout"))?;
    if cfg!(debug_assertions) {
      let reader = BufReader::new(output);
      reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));
    }
    Ok(())
  });
  child.join().unwrap()
}

pub fn perform_operation_on(mut mo: ManagedObject) -> Result<(), HMError> {
  let _s = mo.method.as_str();
  match _s {
    "symlink" => {
      let source: String = mo.source;
      let destination: String = mo.destination;
      return symlink_file(source, destination);
    }
    "execute" => {
      // in here, we must construct the list of dependencies as managed objects,
      // then either complete them or make sure their 'satisfied' field is true
      //Err(HMError::Regular(hmek::SolutionError))
      let cmd: String = mo.solution;
      let _ = execute!(stdout(), SetForegroundColor(Color::Green));
      println!("Executing `{}` for task `{}`", cmd, mo.name.to_owned());
      let _ = execute!(stdout(), ResetColor);
      let a = execute_solution(cmd);
      match &a {
        Ok(()) => {
          mo.satisfied = true;
          return Ok(());
        }
        Err(e) => return a,
      }
    }
    _ => {
      let _ = execute!(stdout(), SetForegroundColor(Color::Green));
      println!("{}", _s);
      let _ = execute!(stdout(), ResetColor);
      return Ok(());
    }
  }
}
