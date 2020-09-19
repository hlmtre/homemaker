extern crate indicatif;
extern crate shellexpand;
extern crate solvent;
extern crate symlink;

use crate::{
  config::{ManagedObject, Worker},
  hmerror::{ErrorKind as hmek, HMError},
};

use indicatif::ProgressBar;
use std::collections::HashMap;
use std::fs::metadata;
use std::io::{stdout, BufRead, BufReader, Error, ErrorKind, Write};
use std::sync::Arc;
use std::{
  path::Path,
  process::{Command, Stdio},
  sync::mpsc::Sender,
  {thread, time},
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

pub fn send_tasks_off_to_college(
  mo: &ManagedObject,
  tx: &Sender<Worker>,
  p: ProgressBar,
) -> Result<(), Error> {
  let s: String = mo.solution.clone().to_string();
  let n: String = mo.name.clone().to_string();
  let tx1: Sender<Worker> = Sender::clone(tx);
  let _child: thread::JoinHandle<Result<(), HMError>> = thread::spawn(move || {
    let mut c = Command::new("bash")
      .arg("-c")
      .arg(s)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .unwrap();
    loop {
      let mut w: Worker = Worker {
        name: n.clone(),
        status: None,
        completed: false,
      };
      //eprintln!("{:#?}", w);
      match c.try_wait() {
        Ok(Some(status)) => {
          p.finish_with_message("done");
          w.status = status.code();
          w.completed = match status.code().unwrap() {
            0 => true,
            _ => false,
          };
          tx1.send(w).unwrap();
          return Ok(());
        }
        Ok(None) => {
          tx1.send(w).unwrap();
          thread::sleep(time::Duration::from_millis(200));
        }
        Err(_e) => return Err(HMError::Regular(hmek::SolutionError)),
      }
    }
  });
  Ok(())
}

/*
  create non-cyclical dependency graph, then execute them in some (non-deterministic)
  order that solves things without dependencies, then works its way up (or complains about
  cyclical dependencies, which are unsolveable)
*/
pub fn get_task_batches(
  nodes: HashMap<String, ManagedObject>,
) -> Result<Vec<Vec<ManagedObject>>, HMError> {
  let mut depgraph: DepGraph<String> = DepGraph::new();
  for (name, node) in nodes.clone() {
    depgraph.register_dependencies(name.to_owned(), node.dependencies.clone());
  }
  let mut tasks: Vec<Vec<ManagedObject>> = Vec::new();
  for (name, _node) in nodes.clone() {
    let mut q: Vec<ManagedObject> = Vec::new();
    for n in depgraph.dependencies_of(&name).unwrap() {
      match n {
        Ok(r) => {
          let mut a = nodes.get(r).unwrap().to_owned();
          a.set_satisfied();
          q.push(a);
        }
        Err(_e) => {
          return Err(HMError::Regular(hmek::CyclicalDependencyError));
        }
      }
    }
    tasks.push(q);
  }
  Ok(tasks)
}

/*
pub fn get_task_thread(mo: &ManagedObject) -> Result<task::JoinHandle<()>, HMError> {
  let s = mo.solution.clone().to_string();
  let t = mo.name.clone().to_string();
  let child: task::JoinHandle<()> = task::spawn(async {
    let c = Command::new("bash")
      .arg("-c")
      .arg(s)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn();
  });
  Ok(child)
}
*/

#[allow(dead_code)]
fn execute_solution(solution: String) -> Result<(), HMError> {
  // marginally adapted but mostly stolen from
  // https://rust-lang-nursery.github.io/rust-cookbook/os/external.html

  let child: thread::JoinHandle<Result<(), HMError>> = thread::spawn(|| {
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

pub fn perform_operation_on(mo: ManagedObject) -> Result<(), HMError> {
  let _s = mo.method.as_str();
  match _s {
    "symlink" => {
      let source: String = mo.source;
      let destination: String = mo.destination;
      symlink_file(source, destination)
    }
    _ => {
      let _ = execute!(stdout(), SetForegroundColor(Color::Green));
      println!("{}", _s);
      let _ = execute!(stdout(), ResetColor);
      Ok(())
    }
  }
}
