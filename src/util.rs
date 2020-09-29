extern crate crossterm;
extern crate indicatif;

use std::collections::{HashMap, HashSet};
use std::{sync::mpsc, time};

use indicatif::{MultiProgress, ProgressBar};

use std::io::{stdout, Write};

use super::config;
use super::mgmt;

use crossterm::{
  execute,
  style::{Color, ResetColor, SetForegroundColor},
};

pub fn do_tasks(a: HashMap<String, config::ManagedObject>) {
  let mut complex_operations = a.clone();
  let mut simple_operations = a.clone();
  complex_operations.retain(|_, v| v.is_task()); // all the things that aren't just symlink/copy
  simple_operations.retain(|_, v| !v.is_task()); // all the things that are quick (don't need to thread off)
  for (_name, _mo) in simple_operations.into_iter() {
    let _ = mgmt::perform_operation_on(_mo).map_err(|e| {
      let _ = execute!(stdout(), SetForegroundColor(Color::Red));
      eprintln!(
        "Failed to perform operation on {:#?}. \nError: {}\n",
        _name, e
      )
    });
    let _ = execute!(stdout(), ResetColor);
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

fn all_workers_done(workers: HashMap<String, config::Worker>) -> bool {
  for (_n, w) in workers {
    if !w.completed {
      return false;
    }
  }
  true
}
