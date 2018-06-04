// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::error::Error;
use std::process;

struct GitStatus {
    branch: Option<String>,
    ahead: i64,
    behind: i64,

    staged: i64,
    modified: i64,
    deleted: i64,
    unmerged: i64,
    untracked: i64,
}

fn color(c: i32) {
    if c >= 0 {
        print!("\x1b[38;5;{}m", c);
    } else {
        print!("\x1b[39m");
    }
}

fn bold(b: bool) {
    if b {
        print!("\x1b[1m");
    } else {
        print!("\x1b[22m");
    }
}

fn parse_porcelain2(data: String) -> Option<GitStatus> {
    let mut status = GitStatus {
        branch: None,
        ahead: 0,
        behind: 0,

        staged: 0,
        modified: 0,
        deleted: 0,
        unmerged: 0,
        untracked: 0,
    };
    // Simple parser for the porcelain v2 format
    for entry in data.split('\0') {
        let mut entry = entry.split(' ');
        match entry.next() {
            // Header lines
            Some("#") => {
                match entry.next()? {
                    "branch.head" => {
                        let head = entry.next()?;
                        if head != "(detached)" {
                            status.branch = Some(String::from(head));
                        }
                    },
                    "branch.ab" => {
                        let a = entry.next()?;
                        let b = entry.next()?;
                        status.ahead = a.parse::<i64>().ok()?.abs();
                        status.behind = b.parse::<i64>().ok()?.abs();
                    },
                    _ => {},
                }
            },
            // File entries
            Some("1") | Some("2") => {
                let mut xy = entry.next()?.chars();
                let x = xy.next()?;
                let y = xy.next()?;
                if x != '.' {
                    status.staged += 1;
                }
                match y {
                    'M' => status.modified += 1,
                    'D' => status.deleted += 1,
                    _ => {},
                }
            }
            Some("u") => status.unmerged += 1,
            Some("?") => status.untracked += 1,
            _ => {},
        }
    }
    Some(status)
}

fn main() -> Result<(), Box<Error>> {
    let output = process::Command::new("git")
        .args(&["status", "--porcelain=v2", "-z", "--branch", "--untracked-files=all"])
        .stdin(process::Stdio::null())
        .stderr(process::Stdio::null())
        .output()?;
    if !output.status.success() {
        // We're most likely not in a Git repo
        return Ok(())
    }
    let status = String::from_utf8(output.stdout)
        .ok().ok_or("Invalid UTF-8 while decoding Git output")?;

    let status = parse_porcelain2(status)
        .ok_or("Error while parsing Git output")?;

    print!("(");

    color(15);
    bold(true);
    if let Some(branch) = status.branch {
        print!("{}", branch);
    } else {
        // Detached head
        print!(":HEAD");
    }
    bold(false);
    color(-1);

    // Divergence with remote branch
    if status.ahead != 0 {
        print!("↑{}", status.ahead);
    }
    if status.behind != 0 {
        print!("↓{}", status.behind);
    }

    if status.untracked + status.modified + status.deleted + status.unmerged + status.staged > 0 {
        print!("|");
    }
    if status.untracked != 0 {
        color(2);
        print!("+{}", status.untracked);
    }
    if status.modified != 0 {
        color(5);
        print!("~{}", status.modified);
    }
    if status.deleted != 0 {
        color(1);
        print!("-{}", status.deleted);
    }
    if status.unmerged != 0 {
        color(3);
        print!("x{}", status.unmerged);
    }
    if status.staged != 0 {
        color(4);
        print!("•{}", status.staged);
    }

    color(-1);
    print!(")");

    Ok(())
}
