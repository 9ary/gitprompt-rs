use std::process;

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
        print!("\x1b[21m");
    }
}

fn main() {
    let status = match process::Command::new("git")
            .args(&["status", "--porcelain=v2", "-z", "--branch", "--untracked-files=all"])
            .stdin(process::Stdio::null())
            .stderr(process::Stdio::null())
            .output() {
        Err(e) => {
            eprintln!("git: {}", e);
            process::exit(1);
        },
        Ok(output) => {
            if !output.status.success() {
                // We're most likely not in a Git repo
                process::exit(0);
            }
            String::from_utf8(output.stdout).expect("Failed to decode output from Git")
        },
    };

    // Details on the current branch
    let mut branch = None;
    let mut ahead = 0;
    let mut behind = 0;

    // File counters
    let mut staged = 0;
    let mut modified = 0;
    let mut deleted = 0;
    let mut unmerged = 0;
    let mut untracked = 0;

    // Simple parser for the porcelain v2 format
    for entry in status.split('\0') {
        let mut entry = entry.split(' ');
        match entry.next() {
            // Header lines
            Some("#") => {
                match entry.next().unwrap() {
                    "branch.head" => {
                        let head = entry.next().unwrap();
                        if head != "(detached)" {
                            branch = Some(head);
                        }
                    },
                    "branch.ab" => {
                        let a = entry.next().unwrap();
                        let b = entry.next().unwrap();
                        ahead = a.parse::<i64>().unwrap().abs();
                        behind = b.parse::<i64>().unwrap().abs();
                    },
                    _ => {},
                }
            },
            // File entries
            Some("1") | Some("2") => {
                let mut xy = entry.next().unwrap().chars();
                let x = xy.next().unwrap();
                let y = xy.next().unwrap();
                if x != '.' {
                    staged += 1;
                }
                match y {
                    'M' => modified += 1,
                    'D' => deleted += 1,
                    _ => {},
                }
            }
            Some("u") => unmerged += 1,
            Some("?") => untracked += 1,
            _ => {},
        }
    }

    print!("(");

    color(15);
    bold(true);
    if let Some(branch) = branch {
        print!("{}", branch);
    } else {
        // Detached head
        print!(":HEAD");
    }
    bold(false);
    color(-1);

    // Divergence with remote branch
    if ahead != 0 {
        print!("↑{}", ahead);
    }
    if behind != 0 {
        print!("↓{}", behind);
    }

    if untracked + modified + deleted + unmerged + staged > 0 {
        print!("|");
    }
    if untracked != 0 {
        color(2);
        print!("+{}", untracked);
    }
    if modified != 0 {
        color(5);
        print!("~{}", modified);
    }
    if deleted != 0 {
        color(1);
        print!("-{}", deleted);
    }
    if unmerged != 0 {
        color(3);
        print!("x{}", unmerged);
    }
    if staged != 0 {
        color(4);
        print!("•{}", staged);
    }

    color(-1);
    print!(")");
}
