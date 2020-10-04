extern crate console;
extern crate indicatif;

use std::collections::{HashMap, HashSet};
use std::{sync::mpsc, time};

use indicatif::{MultiProgress, ProgressBar};

use super::config;
use super::hmerror;
use super::mgmt;

///
/// Take our list of ManagedObjects to do stuff to, and determine
/// if they're simple or complex (simple is symlink or copy, complex
/// maybe compilation or pulling a git repo). We just do the simple ones, as they
/// won't be computationally expensive.
///
/// For complex ones we get a list of list of MOs that we can do in some order that
/// satisfies their dependencies, then we hand them off to mgmt::send_tasks_off_to_college().
///
pub fn do_tasks(a: HashMap<String, config::ManagedObject>) {
  let mut complex_operations = a.clone();
  let mut simple_operations = a.clone();
  complex_operations.retain(|_, v| v.is_task()); // all the things that aren't just symlink/copy
  simple_operations.retain(|_, v| !v.is_task()); // all the things that are quick (don't need to thread off)
  for (_name, _mo) in simple_operations.into_iter() {
    let _ = mgmt::perform_operation_on(_mo).map_err(|e| {
      hmerror::error(
        format!("Failed to perform operation on {:#?}", _name),
        e.to_string().as_str(),
      )
    });
  }
  let (tx, rx) = mpsc::channel();
  let mp: MultiProgress = MultiProgress::new();
  let mut t: HashSet<String> = HashSet::new();
  for _a in mgmt::get_task_batches(complex_operations).unwrap() {
    for _b in _a {
      t.insert(_b.name.to_string());
      let _p: ProgressBar = mp.add(ProgressBar::new_spinner());
      mgmt::send_tasks_off_to_college(&_b, &tx, _p).expect("ohtehnoes");
    }
  }

  let mut v: HashMap<String, config::Worker> = HashMap::new();

  loop {
    match rx.try_recv() {
      Ok(_t) => {
        v.insert(_t.name.clone(), _t.clone());
      }
      Err(_) => {}
    }
    std::thread::sleep(time::Duration::from_millis(10));
    if !all_workers_done(v.clone()) {
      continue;
    }
    break;
  }
  mp.join().unwrap();
}

///
/// Iterate through all the workers passed in. If any isn't marked as complete, `return false;`.
///
fn all_workers_done(workers: HashMap<String, config::Worker>) -> bool {
  for (_n, w) in workers {
    if !w.completed {
      return false;
    }
  }
  true
}
