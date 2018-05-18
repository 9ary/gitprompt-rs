# gitprompt-rs

A very simple Git prompt written in Rust

## Usage

Just add `$(gitprompt-rs)` to your shell prompt.

If you're using ZSH, add `setopt promptsubst` to your zshrc, set the `PROMPT`
variable with single quotes `'`, and use the following function to wrap the
output:
```shell
function pr_git() {
    # Wrap SGR sequences with %{ and %} to avoid confusing zsh's length calculation
    gitprompt-rs | perl -p -e 's/(\[.+?m)/%{\1%}/g'
}
```
This isn't built into the program for compatibility with other shells while
keeping things as simple as possible, and the overhead of calling Perl isn't
that big anyway.

## Installation

- Manual: Make sure you have a recent Rust toolchain. Clone this repo, then run
  `cargo install`.
- Arch Linux: [AUR package](https://aur.archlinux.org/packages/gitprompt-rs/)
- Other distros: make a pull request to add your package or build script!
