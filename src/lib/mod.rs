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
#![allow(clippy::many_single_char_names)]
#![allow(dead_code)]
#![allow(unused_macros)]
extern crate console;
extern crate indicatif;
extern crate log;
extern crate shellexpand;
extern crate simplelog;
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
use log::{info, warn};
use solvent::DepGraph;
use std::{
  collections::{HashMap, HashSet},
  fmt,
  fs::{copy, create_dir_all, metadata, remove_dir_all, remove_file},
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
        #[allow(clippy::for_loops_over_fallibles)]
        for d in self.dependencies.get(&i) {
          if d.is_empty() {
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
/// Copy our {file|directory} to the destination. Generally
/// we'll be doing this in a tilde'd home subdirectory, so
/// we need to be careful to get our Path right.
///
pub fn copy_item(source: String, target: String, force: bool) -> Result<(), HMError> {
  let _lsource: String = shellexpand::tilde(&source).to_string();
  let _ltarget: String = shellexpand::tilde(&target).to_string();
  let md = match metadata(_lsource.clone()) {
    Ok(a) => a,
    Err(e) => return Err(HMError::Io(e)),
  };
  if force && Path::new(_ltarget.as_str()).exists() {
    if md.is_dir() {
      remove_dir_all(_ltarget.clone())?;
    } else {
      remove_file(_ltarget.clone())?;
    }
  }
  copy(Path::new(_lsource.as_str()), Path::new(_ltarget.as_str()))?;
  Ok(())
}

///
/// Either create a symlink to a file or directory. Generally
/// we'll be doing this in a tilde'd home subdirectory, so
/// we need to be careful to get our Path right.
///
pub fn symlink_item(source: String, target: String, force: bool) -> Result<(), HMError> {
  let _lsource: String = shellexpand::tilde(&source).to_string();
  let _ltarget: String = shellexpand::tilde(&target).to_string();
  let md = match metadata(_lsource.clone()) {
    Ok(a) => a,
    Err(e) => return Err(HMError::Io(e)),
  };
  let lmd = metadata(_ltarget.clone());
  if force && Path::new(_ltarget.as_str()).exists() {
    // this is reasonably safe because if the target exists our lmd is a Result not an Err
    if lmd.unwrap().is_dir() {
      remove_dir_all(_ltarget.clone())?;
    } else {
      remove_file(_ltarget.clone())?;
    }
  }
  // create all the parent directories required for the target file/directory
  create_dir_all(Path::new(_ltarget.as_str()).parent().unwrap())?;
  if md.is_dir() {
    sd(Path::new(_lsource.as_str()), Path::new(_ltarget.as_str()))?;
  } else if md.is_file() {
    sf(Path::new(_lsource.as_str()), Path::new(_ltarget.as_str()))?;
  }
  Ok(())
}

///
/// Take a ManagedObject task, an mpsc tx, and a Progressbar. Execute task and regularly inform the rx
/// (all the way over back in `main()`)about our status using config::Worker.
///
/// -TODO-: allow the `verbose` bool to show the output of the tasks as they go.
/// Hey, it's done! Writes out to the logs/ directory.
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
    let output: std::process::ChildStdout = c.stdout.take().unwrap();
    let reader: BufReader<std::process::ChildStdout> = BufReader::new(output);
    // run this in another thread or the little block of tasks draws line-by-line
    // instead of all at once then updating as the task info gets Worker'd back
    thread::spawn(|| {
      reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| info!("{}", line));
    });
    p.set_style(
      ProgressStyle::default_spinner()
        .template("[{elapsed:4}] {prefix:.bold.dim} {spinner} {wide_msg}"),
    );
    p.enable_steady_tick(200);
    let x = pad_str(format!("task {}", n).as_str(), 30, Alignment::Left, None).into_owned();
    p.set_prefix(x);
    loop {
      let mut w: Worker = Worker {
        name: n.clone(),
        status: None,
        completed: false,
      };
      match c.try_wait() {
        // we check each child status...
        Ok(Some(status)) => {
          if status.success() {
            // if we're done, send back :thumbsup:
            p.finish_with_message(console::style("✓").green().to_string());
            w.status = status.code();
            w.completed = true;
            tx1.send(w).unwrap();
            info!("Successfully completed {}.", n);
            return Ok(());
          } else {
            // or :sadface:
            drop(tx1);
            warn!("Error within `{}`", s1);
            p.abandon_with_message(console::style("✗").red().to_string());
            return Err(HMError::Regular(hmek::SolutionError {
              solution: String::from(s1),
            }));
          }
        } // it's sent back nothing, not error, but not done
        Ok(None) => {
          tx1.send(w).unwrap();
          thread::sleep(time::Duration::from_millis(200));
        }
        Err(_e) => {
          // ahh send back err!
          drop(tx1);
          p.abandon_with_message(console::style("✗").red().to_string());
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
/// let v: Vec<Vec<ManagedObject>> = get_task_batches(nodes, None).unwrap();
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
  target_task: Option<String>,
) -> Result<Vec<Vec<ManagedObject>>, HMError> {
  let our_os = config::determine_os();
  let mut our_nodes = nodes.clone();
  let mut depgraph: DepGraph<String> = DepGraph::new();
  let mut nodes_to_remove: Vec<String> = Vec::new();
  let mut wrong_platforms: HashMap<String, config::OS> = HashMap::new();
  for (name, node) in &our_nodes {
    if node.os.is_none() || node.os.clone().unwrap() == our_os {
      depgraph.register_dependencies(name.to_owned(), node.dependencies.clone());
    } else {
      nodes_to_remove.push(name.to_string());
      wrong_platforms.insert(name.clone(), node.os.clone().unwrap());
    }
  }
  for n in nodes_to_remove {
    our_nodes.remove(&n);
  }
  let mut tasks: Vec<Vec<ManagedObject>> = Vec::new();
  let mut _dedup: HashSet<String> = HashSet::new();
  /*
    ok. we've got a target task. we can get the depgraph for ONLY that task,
    and don't need to go through our entire config to solve for each.
    just the subtree involved in our target.
  */
  if target_task.is_some() {
    let tt_name = target_task.unwrap();
    // TODO: break out getting our tasklist into its own function
    // de-dup me!
    let mut qtdg: Vec<ManagedObject> = Vec::new();
    let tdg: solvent::DepGraphIterator<String> = match depgraph.dependencies_of(&tt_name) {
      Ok(i) => i,
      Err(_) => {
        return Err(HMError::Regular(hmek::DependencyUndefinedError {
          dependency: String::from(tt_name),
        }));
      }
    };
    for n in tdg {
      match n {
        Ok(r) => {
          let mut a = match our_nodes.get(r) {
            Some(a) => a,
            None => {
              /*
              if we have a dependency, but it can't be solved because it's for the incorrect platform,
              let's complain about it.
              doing it this way is necessary because we DO still want our dependency graph to get run.
              */
              if wrong_platforms.contains_key(r) {
                return Err(HMError::Regular(hmek::IncorrectPlatformError {
                  dependency: String::from(r),
                  platform: our_os,
                  target_platform: wrong_platforms.get(r).cloned().unwrap(),
                }));
              } else {
                return Err(HMError::Regular(hmek::DependencyUndefinedError {
                  dependency: String::from(r),
                }));
              }
            }
          }
          .to_owned();
          a.set_satisfied();
          qtdg.push(a);
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
    tasks.push(qtdg);
  } else {
    for (name, _node) in &our_nodes {
      let mut q: Vec<ManagedObject> = Vec::new();
      let dg: solvent::DepGraphIterator<String> = depgraph.dependencies_of(&name).unwrap();
      for n in dg {
        match n {
          Ok(r) => {
            let c = String::from(r.as_str());
            // returns true if the set DID NOT have c in it already
            if _dedup.insert(c) {
              let mut a = match our_nodes.get(r) {
                Some(a) => a,
                None => {
                  /*
                  if we have a dependency, but it can't be solved because it's for the incorrect platform,
                  let's complain about it.
                  doing it this way is necessary because we DO still want our dependency graph to get run.
                  */
                  if wrong_platforms.contains_key(r) {
                    return Err(HMError::Regular(hmek::IncorrectPlatformError {
                      dependency: String::from(r),
                      platform: our_os,
                      target_platform: wrong_platforms.get(r).cloned().unwrap(),
                    }));
                  } else {
                    return Err(HMError::Regular(hmek::DependencyUndefinedError {
                      dependency: String::from(r),
                    }));
                  }
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
/// Pretty simple.
/// Hand off to the actual function that does the work.
///
pub fn perform_operation_on(mo: ManagedObject) -> Result<(), HMError> {
  let _s = mo.method.as_str();
  match _s {
    "symlink" => {
      let source: String = mo.source;
      let destination: String = mo.destination;
      symlink_item(source, destination, mo.force)
    }
    "copy" => {
      let source: String = mo.source;
      let destination: String = mo.destination;
      copy_item(source, destination, mo.force)
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
pub fn do_tasks(
  a: HashMap<String, config::ManagedObject>,
  target_task: Option<String>,
) -> Result<(), HMError> {
  let mut complex_operations = a.clone();
  let mut simple_operations = a.clone();
  complex_operations.retain(|_, v| v.is_task()); // all the things that aren't just symlink/copy
  simple_operations.retain(|_, v| !v.is_task()); // all the things that are quick (don't need to thread off)
  if target_task.is_some() {
    let tt_name = target_task.clone().unwrap();
    // only keep the target task, if it's in here...
    // we can't do this with complex tasks, because the target may have deps we want
    // we'll handle that later in get_task_batches
    simple_operations.retain(|_, v| v.name == tt_name);
  }
  for (_name, _mo) in simple_operations.into_iter() {
    // lol postmaclone
    let p = _mo.post.clone();
    let a = perform_operation_on(_mo).map_err(|e| {
      hmerror::error(
        format!("Failed to perform operation on {:#?}", _name).as_str(),
        e.to_string().as_str(),
      )
    });
    if a.is_ok() {
      hmerror::happy_print(format!("Successfully performed operation on {:#?}", _name).as_str());
      if p.len() > 0 {
        println!("↳ Executing post {} for {}... ", p, _name);
        let _ = execute_solution(p);
      }
    }
  }
  let (tx, rx) = mpsc::channel();
  let mp: MultiProgress = MultiProgress::new();
  let mut t: HashSet<String> = HashSet::new();
  let _v = get_task_batches(complex_operations, target_task.clone()).unwrap_or_else(|er| {
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
