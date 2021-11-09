#![allow(clippy::many_single_char_names)]
use crate::hmerror::{self, ErrorKind as hmek, HMError};
use crate::{app, config};
use crate::{
  app::APP,
  config::{ManagedObject, Worker},
};

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
#[derive(Debug, Clone, PartialEq)]
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
  let s: String = mo.solution.clone();
  let s1: String = mo.solution.clone();
  let n: String = mo.name.clone();
  let tx1: Sender<Worker> = Sender::clone(tx);
  let _: thread::JoinHandle<Result<(), HMError>> = thread::spawn(move || {
    let mut c = Command::new("bash")
      .arg("-c")
      .arg(s)
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .unwrap();
    let mut _a = APP.write().unwrap();
    _a.append_summary(&"i do tasks and i'm ok".to_string());
    drop(_a);
    let output: std::process::ChildStdout = c.stdout.take().unwrap();
    let reader: BufReader<std::process::ChildStdout> = BufReader::new(output);
    // run this in another thread or the little block of tasks draws line-by-line
    // instead of all at once then updating as the task info gets Worker'd back
    thread::spawn(|| {
      reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
          let mut _a = APP.write().unwrap();
          _a.append_output(&line);
          drop(_a);
          info!("{}", line)
        });
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
            let mut _a = APP.write().unwrap();
            _a.append_summary(&format!("Successfully completed {}.", n));
            drop(_a);
            info!("Successfully completed {}.", n);
            return Ok(());
          } else {
            // or :sadface:
            drop(tx1);
            warn!("Error within `{}`", s1);
            p.abandon_with_message(console::style("✗").red().to_string());
            return Err(HMError::Regular(hmek::SolutionError { solution: s1 }));
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
          return Err(HMError::Regular(hmek::SolutionError { solution: s1 }));
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
/// use hm::task::{get_task_batches, send_tasks_off_to_college};
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
  mut nodes: HashMap<String, ManagedObject>,
  target_task: Option<String>,
) -> Result<Vec<Vec<ManagedObject>>, HMError> {
  let our_os = config::determine_os();
  let mut depgraph: DepGraph<String> = DepGraph::new();
  let mut nodes_to_remove: Vec<String> = Vec::new();
  let mut wrong_platforms: HashMap<String, config::OS> = HashMap::new();
  for (name, node) in &nodes {
    if node.os.is_none() || node.os.clone().unwrap() == our_os {
      depgraph.register_dependencies(name.to_owned(), node.dependencies.clone());
    } else {
      nodes_to_remove.push(name.to_string());
      wrong_platforms.insert(name.clone(), node.os.clone().unwrap());
    }
  }
  for n in nodes_to_remove {
    nodes.remove(&n);
  }
  let mut tasks: Vec<Vec<ManagedObject>> = Vec::new();
  let mut _dedup: HashSet<String> = HashSet::new();
  /*
    ok. we've got a target task. we can get the depgraph for ONLY that task,
    and don't need to go through our entire config to solve for each.
    just the subtree involved in our target.
  */
  if let Some(tt_name) = target_task {
    // TODO: break out getting our tasklist into its own function
    // de-dup me!
    let mut qtdg: Vec<ManagedObject> = Vec::new();
    let tdg: solvent::DepGraphIterator<String> = match depgraph.dependencies_of(&tt_name) {
      Ok(i) => i,
      Err(_) => {
        return Err(HMError::Regular(hmek::DependencyUndefinedError {
          dependency: tt_name,
        }));
      }
    };
    for n in tdg {
      match n {
        Ok(r) => {
          let mut a = match nodes.get(r) {
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
    // not doing target task
    // we're doing the entire config
    for name in nodes.keys() {
      let mut q: Vec<ManagedObject> = Vec::new();
      let dg: solvent::DepGraphIterator<String> = depgraph.dependencies_of(name).unwrap();
      for n in dg {
        match n {
          Ok(r) => {
            let c = String::from(r.as_str());
            // returns true if the set DID NOT have c in it already
            if _dedup.insert(c) {
              let mut a = match nodes.get(r) {
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
    let reader = BufReader::new(output);
    reader
      .lines()
      .filter_map(|line| line.ok())
      .for_each(|line| {
        let mut a = APP.write().unwrap();
        a.append_summary(&line);
        drop(a);
      });
    Ok(())
  });
  let mut a = APP.write().unwrap();
  a.append_summary(&"HELLO THERE GENERAL KENOBI".to_string());
  drop(a);
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
      println!("{}", style(_s.to_string()).red());
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
  a: HashMap<String, ManagedObject>,
  target_task: Option<String>,
) -> Result<(), HMError> {
  let mut complex_operations = a.clone();
  let mut simple_operations = a;
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
      let mut a = APP.write().unwrap();
      a.append_summary(&"Success!".to_string());
      //app::tui_element_append_output("Success!".to_string());
      hmerror::happy_print(format!("Successfully performed operation on {:#?}", _name).as_str());
      if !p.is_empty() {
        let mut _a = APP.write().unwrap();
        _a.append_output(&format!("↳ Executing post {} for {}... ", p, _name));
        drop(_a);
        let _ = execute_solution(p);
      }
      drop(a);
    }
  }
  let (tx, rx) = mpsc::channel();
  let mp: MultiProgress = MultiProgress::new();
  let mut t: HashSet<String> = HashSet::new();
  let _v = get_task_batches(complex_operations, target_task).unwrap_or_else(|er| {
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

  let mut v: HashMap<String, Worker> = HashMap::new();

  // create a map of each Worker, and just poll them until they're done
  // the hashmap ensures we have only one of each worker
  loop {
    if let Ok(_t) = rx.try_recv() {
      v.insert(_t.name.clone(), _t.clone());
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
fn all_workers_done(workers: &HashMap<String, Worker>) -> bool {
  for w in workers.values() {
    if !w.completed {
      return false;
    }
  }
  true
}

#[cfg(test)]
mod task_test {
  use super::*;
  use solvent::DepGraph;

  #[test]
  // make sure solvent hasn't changed their struct too much
  fn test_depgraph_transmute() {
    let mut depgraph: DepGraph<String> = DepGraph::new();
    let a = "dependent_a";
    let deps = ["dependency_a".to_string(), "dependency_b".to_string()].to_vec();
    depgraph.register_dependencies(a.to_string(), deps);
    let mut my_sneaky_depgraph: SneakyDepGraphImposter<String> =
      unsafe { std::mem::transmute(depgraph.clone()) };
    let tdg: solvent::DepGraphIterator<String> = depgraph.dependencies_of(&a.to_string()).unwrap();
    let mut v: Vec<String> = Vec::new();
    for e in tdg {
      v.push(e.unwrap().to_string());
    }
    // they have the same contents but comparison tests order of elements too
    assert_eq!(v.sort(), my_sneaky_depgraph.nodes.sort());
  }
}
