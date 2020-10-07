extern crate console;
extern crate indicatif;
extern crate shellexpand;
extern crate solvent;
extern crate symlink;

use crate::{
  config::{ManagedObject, Worker},
  hmerror::{ErrorKind as hmek, HMError},
};

use console::{pad_str, style, Alignment};
use indicatif::{ProgressBar, ProgressStyle};
use solvent::DepGraph;
use std::{
  collections::HashMap,
  collections::HashSet,
  fs::metadata,
  io::{BufRead, BufReader, Error, ErrorKind},
  path::Path,
  process::{Command, Stdio},
  sync::mpsc::Sender,
  {thread, time},
};
use symlink::{symlink_dir as sd, symlink_file as sf};

/*
use crossterm::{
  execute,
  style::{Color, ResetColor, SetForegroundColor},
};
*/

///
/// Either create a symlink to a file or directory. Generally
/// we'll be doing this in a tilde'd home subdirectory, so
/// we need to be careful to get our Path right.
///
fn symlink_file(source: String, target: String) -> Result<(), HMError> {
  let md = match metadata(shellexpand::tilde(&source).to_string()) {
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

///
/// Take a ManagedObject task, an mpsc tx, and a Progressbar. Execute task and regularly inform the rx
/// about our status using config::Worker.
///
/// Return () or io::Error (something went wrong in our task).
///
pub fn send_tasks_off_to_college(
  mo: &ManagedObject,
  tx: &Sender<Worker>,
  p: ProgressBar,
) -> Result<(), Error> {
  let s: String = mo.solution.clone().to_string();
  //let s1 = s.clone();
  let n: String = mo.name.clone().to_string();
  let tx1: Sender<Worker> = Sender::clone(tx);
  let _child: thread::JoinHandle<Result<(), HMError>> = thread::spawn(move || {
    let mut c = Command::new("bash")
      .arg("-c")
      .arg(s)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    //let output = c.stdout.take().unwrap();
    //let reader = BufReader::new(output);
    /*
      reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));
    }
    */
    //p.println(s1);
    p.set_style(
      ProgressStyle::default_spinner()
        .template("[{elapsed:4}] {prefix:.bold.dim} {spinner} {wide_msg}"),
    );
    p.enable_steady_tick(200);
    p.set_prefix(
      pad_str(format!("task {}", n).as_str(), 30, Alignment::Left, None)
        .into_owned()
        .as_str(),
    );
    loop {
      let mut w: Worker = Worker {
        name: n.clone(),
        status: None,
        completed: false,
      };
      match c.try_wait() {
        Ok(Some(status)) => {
          p.finish_with_message(console::style("✓").green().to_string().as_str());
          w.status = status.code();
          w.completed = true;
          tx1.send(w).unwrap();
          return Ok(());
        }
        Ok(None) => {
          tx1.send(w).unwrap();
          thread::sleep(time::Duration::from_millis(200));
        }
        Err(_e) => {
          drop(tx1);
          p.abandon_with_message("error!");
          return Err(HMError::Regular(hmek::SolutionError));
        }
      }
    }
  });
  Ok(())
}

/*
*/
///
/// Create a non-cyclical dependency graph and give it back as a Vec&lt;Vec&lt;ManagedObject&gt;&gt;.
/// Will return a CyclicalDependencyError if the graph is unsolveable.
/// Intended to be used with either mgmt::execute_solution or mgmt::send_tasks_off_to_college.
///
///
/// Example:
///
/// ```
/// let (rx, tx) = mpsc::channel();
/// let p = ProgressBar::new_spinner();
/// let v: Vec&lt;Vec&lt;ManagedObject&gt;&gt; = get_task_batches(nodes);
/// for a in v {
///   for b in a {
///     mgmt::send_tasks_off_to_college(&b, &tx, p);
///   }
/// }
/// ```
///
pub fn get_task_batches(
  nodes: HashMap<String, ManagedObject>,
) -> Result<Vec<Vec<ManagedObject>>, HMError> {
  let mut depgraph: DepGraph<String> = DepGraph::new();
  for (name, node) in nodes.clone() {
    depgraph.register_dependencies(name.to_owned(), node.dependencies.clone());
  }
  let mut tasks: Vec<Vec<ManagedObject>> = Vec::new();
  let mut _dedup: HashSet<String> = HashSet::new();
  for (name, _node) in nodes.clone() {
    let mut q: Vec<ManagedObject> = Vec::new();
    for n in depgraph.dependencies_of(&name).unwrap() {
      match n {
        Ok(r) => {
          let c = String::from(r.as_str());
          // returns true if the set DID NOT have c in it already
          if _dedup.insert(c) {
            let mut a = nodes.get(r).unwrap().to_owned();
            a.set_satisfied();
            q.push(a);
          }
        }
        Err(_e) => {
          return Err(HMError::Regular(hmek::CyclicalDependencyError));
        }
      }
    }
    tasks.push(q);
  }
  drop(_dedup);
  Ok(tasks)
}

///
/// Execute the shell commands specified in the MO's solution in a thread, so as not to block.
///
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

///
/// Pretty simple. We currently support only symlinking, but copying would be trivial.
/// Hand off to the actual function that does the work.
///
pub fn perform_operation_on(mo: ManagedObject) -> Result<(), HMError> {
  let _s = mo.method.as_str();
  match _s {
    "symlink" => {
      let source: String = mo.source;
      let destination: String = mo.destination;
      symlink_file(source, destination)
    }
    _ => {
      println!("{}", style(format!("{}", _s)).red());
      Ok(())
    }
  }
}
