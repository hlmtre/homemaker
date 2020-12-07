//! `hm` (short for `homemaker`) provides a library to do basic filesystem operations
//! that you'd want a dotfile manager to do (like symlink).
//! It also provides functionality to thread off heavier operations
//! into task threads, which regularly report back their status with
//! Worker objects over a `std::sync::mpsc`.
//!
//! `hm` can also perform dependency resolution, thanks to the solvent crate. You can
//! provide a big ol' list of tasks to complete, each with their own dependencies, and
//! as long as you have a solveable dependency graph, you can get workable batches from
//! get_task_batches(). They will be in *some* order that will resolve your dependencies,
//! but that order is non-deterministic - if there's multiple ways to solve a graph,
//! you'll probably get all those different ways back if you run it multiple times.
//!
//! The crate provides this library, which is in turn used by the bin `hm` (in `src/bin/main.rs`).
//! `hm` is a commandline program to help with dotfile (and more) management.
//!
//! It can handle putting configuration files where they should go, but its real
//! strength lies in the solutions it'll execute - shell scripts, usually - to
//! pull a git repository, compile something from source, etc.
//!
//! `hm` exists because I bring along a few utilities to all Linux boxes I regularly
//! use, and those are often built from source. So rather than manually install
//! all dependency libraries, then build each dependent piece, then finally the
//! top-level dependent program, I built hm.
//!
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
#![allow(unused_macros)]
extern crate console;
extern crate indicatif;
extern crate shellexpand;
extern crate solvent;
extern crate symlink;
extern crate sys_info;

pub mod config;
mod hm_macro;
pub mod hmerror;

use config::{ManagedObject, Worker};
use hmerror::{ErrorKind as hmek, HMError};

use console::{pad_str, style, Alignment};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use solvent::DepGraph;
use std::{
  collections::{HashMap, HashSet},
  fmt,
  fs::metadata,
  io::{BufRead, BufReader, Error, ErrorKind},
  path::Path,
  process::{exit, Command, Stdio},
  sync::mpsc::{self, Sender},
  {thread, time},
};
use symlink::{symlink_dir as sd, symlink_file as sf};

/// I just wanna borrow one to look at it for a minute.
/// use me with std::mem::transmute()
/// absolutely not pub struct. don't look at me.
#[derive(Debug, Clone)]
struct SneakyDepGraphImposter<String> {
  nodes: Vec<String>,
  dependencies: HashMap<usize, HashSet<usize>>,
  satisfied: HashSet<usize>,
}

impl fmt::Display for SneakyDepGraphImposter<String> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut i: usize = 0;
    for n in self.nodes.clone() {
      if self.dependencies.get(&i).is_some() {
        let _ = write!(f, "[ {} -> ", n);
        for d in self.dependencies.get(&i) {
          if d.len() == 0 {
            write!(f, "<no deps> ")?;
          }
          let mut j: usize = 1;
          for m in d {
            if j == d.len() {
              write!(f, "{} ", self.nodes[*m])?;
            } else {
              write!(f, "{}, ", self.nodes[*m])?;
            }
            j += 1;
          }
        }
      }
      i += 1;
      if i != self.nodes.len() {
        write!(f, "], ")?;
      } else {
        write!(f, "]")?;
      }
    }
    Ok(())
  }
}

///
/// Either create a symlink to a file or directory. Generally
/// we'll be doing this in a tilde'd home subdirectory, so
/// we need to be careful to get our Path right.
///
pub fn symlink_file(source: String, target: String) -> Result<(), HMError> {
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
/// (all the way over back in `main()`)about our status using config::Worker.
///
/// TODO: allow the `verbose` bool to show the output of the tasks as they go.
///
/// Return () or io::Error (something went wrong in our task).
///
pub fn send_tasks_off_to_college(
  mo: &ManagedObject,
  tx: &Sender<Worker>,
  p: ProgressBar,
) -> Result<(), Error> {
  let s: String = mo.solution.clone().to_string();
  let s1: String = mo.solution.clone().to_string();
  let n: String = mo.name.clone().to_string();
  let tx1: Sender<Worker> = Sender::clone(tx);
  let _: thread::JoinHandle<Result<(), HMError>> = thread::spawn(move || {
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
          if status.success() {
            p.finish_with_message(console::style("✓").green().to_string().as_str());
            w.status = status.code();
            w.completed = true;
            tx1.send(w).unwrap();
            return Ok(());
          } else {
            drop(tx1);
            p.abandon_with_message(console::style("✗").red().to_string().as_str());
            return Err(HMError::Regular(hmek::SolutionError {
              solution: String::from(s1),
            }));
          }
        }
        Ok(None) => {
          tx1.send(w).unwrap();
          thread::sleep(time::Duration::from_millis(200));
        }
        Err(_e) => {
          drop(tx1);
          p.abandon_with_message(console::style("✗").red().to_string().as_str());
          return Err(HMError::Regular(hmek::SolutionError {
            solution: String::from(s1),
          }));
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
/// extern crate indicatif;
/// use std::sync::mpsc;
/// use std::collections::HashMap;
/// use indicatif::{MultiProgress, ProgressBar};
/// use hm::config::ManagedObject;
/// use hm::{get_task_batches, send_tasks_off_to_college};
/// let nodes: HashMap<String, ManagedObject> = HashMap::new();
/// let (tx, rx) = mpsc::channel();
/// let mp: MultiProgress = MultiProgress::new();
/// let v: Vec<Vec<ManagedObject>> = get_task_batches(nodes).unwrap();
/// for a in v {
///   for b in a {
///     let _p: ProgressBar = mp.add(ProgressBar::new_spinner());
///     send_tasks_off_to_college(&b, &tx, _p);
///   }
/// }
/// ```
///
pub fn get_task_batches(
  nodes: HashMap<String, ManagedObject>,
) -> Result<Vec<Vec<ManagedObject>>, HMError> {
  let our_os = config::determine_os();
  let mut depgraph: DepGraph<String> = DepGraph::new();
  for (name, node) in nodes.clone() {
    if node.os.is_none() || node.os.unwrap() == our_os {
      depgraph.register_dependencies(name.to_owned(), node.dependencies.clone());
    }
  }
  let mut tasks: Vec<Vec<ManagedObject>> = Vec::new();
  let mut _dedup: HashSet<String> = HashSet::new();
  for (name, _node) in nodes.clone() {
    let mut q: Vec<ManagedObject> = Vec::new();
    let _name = name.clone();
    let dg = depgraph.dependencies_of(&name).unwrap();
    for n in dg {
      match n {
        Ok(r) => {
          let c = String::from(r.as_str());
          // returns true if the set DID NOT have c in it already
          if _dedup.insert(c) {
            let mut a = match nodes.get(r) {
              Some(a) => a,
              None => {
                return Err(HMError::Regular(hmek::DependencyUndefinedError {
                  dependency: String::from(r),
                }));
              }
            }
            .to_owned();
            a.set_satisfied();
            q.push(a);
          }
        }
        Err(_e) => unsafe {
          // we'll just borrow this for a second
          // just to look
          // i'm not gonna touch it i promise
          let my_sneaky_depgraph: SneakyDepGraphImposter<String> = std::mem::transmute(depgraph);
          return Err(HMError::Regular(hmek::CyclicalDependencyError {
            // we can do this because we've implemented fmt::Display for SneakyDepGraphImposter
            dependency_graph: my_sneaky_depgraph.to_string(),
          }));
        },
      }
    }
    tasks.push(q);
  }
  Ok(tasks)
}

///
/// Execute the shell commands specified in the MO's solution in a thread, so as not to block.
///
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

///
/// Take our list of ManagedObjects to do stuff to, and determine
/// if they're simple or complex (simple is symlink or copy, complex
/// maybe compilation or pulling a git repo). We just do the simple ones, as they
/// won't be computationally expensive.
///
/// For complex ones we get a list of list of MOs that we can do in some order that
/// satisfies their dependencies, then we hand them off to send_tasks_off_to_college().
///
pub fn do_tasks(a: HashMap<String, config::ManagedObject>) -> Result<(), HMError> {
  let mut complex_operations = a.clone();
  let mut simple_operations = a.clone();
  complex_operations.retain(|_, v| v.is_task()); // all the things that aren't just symlink/copy
  simple_operations.retain(|_, v| !v.is_task()); // all the things that are quick (don't need to thread off)
  for (_name, _mo) in simple_operations.into_iter() {
    let _ = perform_operation_on(_mo).map_err(|e| {
      hmerror::error(
        format!("Failed to perform operation on {:#?}", _name).as_str(),
        e.to_string().as_str(),
      )
    });
  }
  let (tx, rx) = mpsc::channel();
  let mp: MultiProgress = MultiProgress::new();
  let mut t: HashSet<String> = HashSet::new();
  let _v = get_task_batches(complex_operations).unwrap_or_else(|er| {
    hmerror::error(
      "Error occurred attempting to get task batches",
      format!("{}{}", "\n", er.to_string().as_str()).as_str(),
    );
    exit(3);
  });
  for _a in _v {
    for _b in _a {
      t.insert(_b.name.to_string());
      let _p: ProgressBar = mp.add(ProgressBar::new_spinner());
      send_tasks_off_to_college(&_b, &tx, _p).expect("ohtehnoes");
    }
  }

  let mut v: HashMap<String, config::Worker> = HashMap::new();

  // create a map of each Worker, and just poll them until they're done
  // the hashmap ensures we have only one of each worker
  loop {
    match rx.try_recv() {
      Ok(_t) => {
        v.insert(_t.name.clone(), _t.clone());
      }
      Err(_) => {}
    }
    std::thread::sleep(time::Duration::from_millis(10));
    if !all_workers_done(&v) {
      continue;
    }
    break;
  }
  mp.join().unwrap();
  Ok(())
}

///
/// Iterate through all the workers passed in. If any isn't marked as complete, `return false;`.
/// Let's take a reference, because we're only reading, and don't need ownership.
///
fn all_workers_done(workers: &HashMap<String, config::Worker>) -> bool {
  for (_n, w) in workers {
    if !w.completed {
      return false;
    }
  }
  true
}
