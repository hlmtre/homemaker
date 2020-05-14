extern crate shellexpand;

use crate::config::ManagedObject;
use std::os::unix::fs;
use std::path::Path;

// fs::symlink(Path::new(shellexpand::tilde("~/a.txt").to_mut()), Path::new(shellexpand::tilde("~/b.txt").to_mut()))?;
fn symlink_file(source: String, target: String) -> std::io::Result<()> {
  fs::symlink(Path::new(shellexpand::tilde(&source).to_mut()), Path::new(shellexpand::tilde(&target).to_mut()))?;
  Ok(())
}

pub fn perform_operation_on(mo: ManagedObject) -> std::io::Result<()> {
  let _s: String = String::from("symlink");
  match &mo.method {
    _s =>  {
      let source: String = mo.source;
      let destination: String = mo.destination;
      return symlink_file(source, destination);
    },
  };
}
