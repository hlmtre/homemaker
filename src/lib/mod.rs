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

//!
//!
//! have a dotfiles directory with all your stuff in it? have homemaker put everything in its right place.
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
//!  os = 'linux::fedora'
//!
//!  [[obj]]
//!  task = 'zt'
//!  solution = 'cd ~/dotfiles/zt && git pull'
//!  dependencies = ['maim, slop']
//!
//!  [[obj]]
//!  task = 'slop'
//!  source = '~/dotfiles/zt/slop'
//!  solution = 'cd ~/dotfiles/zt/slop; make clean; cmake -DCMAKE_INSTALL_PREFIX="/usr" ./ && make && sudo make install'
//!  method = 'execute'
//!  os = 'linux::debian'
//!  ```
//!  3. `hm --config ~/path/to/your/config.toml`
//!
//!  [![built with spacemacs](https://cdn.rawgit.com/syl20bnr/spacemacs/442d025779da2f62fc86c2082703697714db6514/assets/spacemacs-badge.svg)](http://spacemacs.org) and neovim.

pub mod app;
pub mod config;
mod hm_macro;
pub mod hmerror;
pub mod task;
