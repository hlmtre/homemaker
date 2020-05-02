extern crate shellexpand;

use std::os::unix::fs;
use std::path::Path;

// fs::symlink(Path::new(shellexpand::tilde("~/a.txt").to_mut()), Path::new(shellexpand::tilde("~/b.txt").to_mut()))?;
pub fn symlink_file(source: String, target: String) -> std::io::Result<()> {
  fs::symlink(Path::new(shellexpand::tilde(&source).to_mut()), Path::new(shellexpand::tilde(&target).to_mut()))?;
  Ok(())
}
