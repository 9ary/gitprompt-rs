# gitprompt-rs

A very simple Git prompt written in Rust

## Usage

Just add `$(gitprompt-rs)` to your shell prompt.

If you're using ZSH, you will want to use `$(gitprompt-rs zsh)` in order to
insert the appropriate escapes in the output, otherwise, it will miscalculate
the length of your prompt and go crazy.

The prompt looks like this: `(master↑4↓7|+2~3-5x6•8)`. The information on
display is as follows:
- Branch info:
  - `master`: name of the current branch, `:HEAD` in detached head mode
  - `↑`: number of commits ahead of remote
  - `↓`: number of commits behind remote
- Work area:
  - `+`: untracked (new) files
  - `~`: modified files
  - `-`: deleted files
  - `x`: merge conflicts
- `•`: staged changes

## Installation

- Manual: Make sure you have a recent Rust toolchain. Clone this repo, then run
  `cargo install`.
- Arch Linux: [AUR package](https://aur.archlinux.org/packages/gitprompt-rs/)
- Other distros: make a pull request to add your package or build script!
