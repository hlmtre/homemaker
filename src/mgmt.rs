extern crate shellexpand;

use crate::config::ManagedObject;
use std::os::unix::fs;
use std::path::Path;
use std::process::Command;

// fs::symlink(Path::new(shellexpand::tilde("~/a.txt").to_mut()), Path::new(shellexpand::tilde("~/b.txt").to_mut()))?;
fn symlink_file(source: String, target: String) -> std::io::Result<()> {
  fs::symlink(Path::new(shellexpand::tilde(&source).to_mut()), Path::new(shellexpand::tilde(&target).to_mut()))?;
  Ok(())
}

fn execute_solution(solution: String) -> std::io::Result<()> {
  let output = Command::new("bash").arg("-c").arg(solution).output().expect("LKJALKJ");
  println!("{:#?}", output);
  Ok(())
}

pub fn perform_operation_on(mo: ManagedObject) -> std::io::Result<()> {
  let _s = mo.method.as_str();
  match _s {
    "symlink" =>  {
      let source: String = mo.source;
      let destination: String = mo.destination;
      return symlink_file(source, destination);
    },
    "execute" => {
      let cmd: String = mo.solution;
      return execute_solution(cmd);
    },
    _ => {
      println!("{}", _s);
      return Ok(());
    }
  };
}
