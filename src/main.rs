use std::env;
use std::error::Error;
use std::process;
use std::sync::LazyLock;

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

enum Shell {
	Unknown,
	Bash,
	Zsh,
}

static SHELL: LazyLock<Shell> = LazyLock::new(|| match env::args().nth(1) {
	Some(arg) => match arg.as_str() {
		"bash" => Shell::Bash,
		"zsh" => Shell::Zsh,
		_ => Shell::Unknown,
	},
	None => Shell::Unknown,
});

fn escape_start() {
	match *SHELL {
		Shell::Bash => print!("\\["),
		Shell::Zsh => print!("%{{"),
		_ => {}
	}
}

fn escape_end() {
	match *SHELL {
		Shell::Bash => print!("\\]"),
		Shell::Zsh => print!("%}}"),
		_ => {}
	}
}

fn color(c: i32) {
	escape_start();
	if c >= 0 {
		print!("\x1b[38;5;{}m", c);
	} else {
		print!("\x1b[39m");
	}
	escape_end();
}

fn bold(b: bool) {
	escape_start();
	if b {
		print!("\x1b[1m");
	} else {
		print!("\x1b[22m");
	}
	escape_end();
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
			Some("#") => match entry.next()? {
				"branch.head" => {
					let head = entry.next()?;
					if head != "(detached)" {
						status.branch = Some(String::from(head));
					}
				}
				"branch.ab" => {
					let a = entry.next()?;
					let b = entry.next()?;
					status.ahead = a.parse::<i64>().ok()?.abs();
					status.behind = b.parse::<i64>().ok()?.abs();
				}
				_ => {}
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
					'A' => status.untracked += 1,
					'M' => status.modified += 1,
					'D' => status.deleted += 1,
					_ => {}
				}
			}
			Some("u") => status.unmerged += 1,
			Some("?") => status.untracked += 1,
			_ => {}
		}
	}
	Some(status)
}

fn main() -> Result<(), Box<dyn Error>> {
	let output = process::Command::new("git")
		.args(["config", "--get", "-z", "gitprompt-rs.showUntrackedFiles"])
		.stdin(process::Stdio::null())
		.stderr(process::Stdio::null())
		.output()?;
	// Any errors here are non-fatal and result in using the default value
	let untracked = output
		.status
		.success()
		.then_some(())
		.and_then(|_| {
			let out = String::from_utf8(output.stdout).ok()?;
			out.split('\0').next().and_then(|v| match v {
				"all" => Some("all"),
				"normal" | "yes" | "true" | "1" => Some("normal"),
				"no" | "false" | "0" => Some("no"),
				_ => None,
			})
		})
		.unwrap_or("all");

	let output = process::Command::new("git")
		.args([
			"status",
			"--porcelain=v2",
			"-z",
			"--branch",
			format!("--untracked-files={untracked}").as_str(),
		])
		.stdin(process::Stdio::null())
		.stderr(process::Stdio::null())
		.output()?;
	if !output.status.success() {
		// We're most likely not in a Git repo
		return Ok(());
	}
	let status = String::from_utf8(output.stdout)
		.ok()
		.ok_or("Invalid UTF-8 while decoding Git output")?;

	let status = parse_porcelain2(status).ok_or("Error while parsing Git output")?;

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

	if status.untracked + status.modified + status.deleted + status.unmerged + status.staged > 0
	{
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
