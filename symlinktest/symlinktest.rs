use std::os::unix::fs;
use std::path::Path;
extern crate shellexpand;

fn main() -> std::io::Result<()> {
  fs::symlink(Path::new(shellexpand::tilde("~/a.txt").to_mut()), Path::new(shellexpand::tilde("~/b.txt").to_mut()))?;
  Ok(())
}
