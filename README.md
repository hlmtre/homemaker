have a dotfiles directory with all your stuff in it? have homemaker put everything in its right place.

[gfycat of it in action](https://gfycat.com/skinnywarmheartedafricanpiedkingfisher)


1. create a config.toml file either anywhere or in ~/.config/homemaker/.
2. enter things to do things to in the file.
example:
```
## config.toml

[[obj]]
file = 'tmux.conf'
source = '~/dotfiles/.tmux.conf'
destination = '~/.tmux.conf'
method = 'symlink'

[[obj]]
task = 'zt'
solution = 'cd ~/dotfiles/zt && git pull'
dependencies = 'maim, slop'

[[obj]]
task = 'slop'
source = '~/dotfiles/zt/slop'
solution = 'cd ~/dotfiles/zt/slop; make clean; cmake -DCMAKE_INSTALL_PREFIX="/usr" ./ && make && sudo make install'
method = 'execute'
```
3. `hm ~/path/to/your/config.toml`

[![built with spacemacs](https://cdn.rawgit.com/syl20bnr/spacemacs/442d025779da2f62fc86c2082703697714db6514/assets/spacemacs-badge.svg)](http://spacemacs.org)

thanks to actual good code:
serde
toml
symlink
solvent
indicatif
console
