//! The tutorials in `docs/tutorials/` are executed, not just read.
//!
//! A tutorial rots quietly: nothing fails when a printed line changes, a
//! reader just finds out. So every ```console block in every tutorial is
//! replayed here against a temporary book folder, and what Booker actually
//! printed is compared with what the tutorial claims it prints. A mismatch
//! names the tutorial and the line that is now a lie.
//!
//! The convention the blocks follow is documented for authors in
//! `docs/tutorials/index.md`; this file is its only implementation.
//!
//! Unlike `tests/commands.rs`, which calls `booker_cli::run` in process, this
//! spawns the real binary. A tutorial quotes the paths a reader's shell prints
//! and the exit codes their shell reports, and neither survives being faked.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Cargo builds the binary for this crate's integration tests and hands us
/// the path to it, so the tutorial is checked against the same `booker` the
/// reader would install.
const BOOKER: &str = env!("CARGO_BIN_EXE_booker");

/// A single U+2026 in an expected line means "anything here". It is for
/// durations and machine-specific paths, and nothing else.
const WILDCARD: char = '…';

#[test]
fn every_tutorial_prints_what_it_says_it_prints() {
    let directory = tutorials_directory();
    let mut tutorials: Vec<PathBuf> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{} cannot be read: {error}", directory.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect();
    tutorials.sort();

    assert!(
        !tutorials.is_empty(),
        "no tutorials in {} — `docs/requirements.md` asks for one per capability",
        directory.display()
    );

    let mut commands_run = 0;
    for tutorial in &tutorials {
        commands_run += check(tutorial);
    }
    assert!(
        commands_run > 0,
        "the tutorials contain no runnable ```console block, so nothing was checked"
    );
}

fn tutorials_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/tutorials")
}

/// Replay one tutorial. Returns how many commands were actually run.
fn check(tutorial: &Path) -> usize {
    let name = label(tutorial);
    let text = std::fs::read_to_string(tutorial)
        .unwrap_or_else(|error| panic!("{name} cannot be read: {error}"));

    let sandbox = tempfile::tempdir().expect("a temporary folder for the tutorial's book");
    let mut shell = Shell {
        tutorial: name.clone(),
        cwd: sandbox.path().to_path_buf(),
        last_exit: None,
    };

    let mut commands_run = 0;
    let mut files_written = 0;
    for fence in fences(&text) {
        let (language, attributes) = fence.info();
        if let Some(path) = attributes
            .iter()
            .find_map(|attribute| attribute.strip_prefix("file="))
        {
            shell.write_file(path, &fence);
            files_written += 1;
            continue;
        }
        if language != "console" || attributes.contains(&"ignore") {
            continue;
        }
        commands_run += shell.run_block(&fence);
    }

    // A numbered tutorial that checks nothing is the failure this whole file
    // exists to prevent, and it would otherwise pass silently.
    //
    // There are two ways to be checked, because there are two kinds of
    // tutorial. One walks the reader through commands, and every command is
    // run and its output compared. The other walks them through the window,
    // where there is nothing to type — but it still tells them to create
    // files, and those files must make a book that actually opens. So a
    // tutorial with no commands is held to that instead: whatever it told
    // the reader to write is loaded and built.
    let numbered = tutorial
        .file_name()
        .and_then(|file| file.to_str())
        .is_some_and(|file| file.starts_with(|c: char| c.is_ascii_digit()));
    if numbered && commands_run == 0 {
        assert!(
            files_written > 0,
            "{name} neither runs a command nor writes a file, so nothing in it is checked. \
             A tutorial nobody verifies is one that rots quietly."
        );
        shell.build_what_the_reader_typed();
    }

    commands_run
}

/// The path as the repository spells it, which is what a failure must name.
fn label(tutorial: &Path) -> String {
    let file = tutorial
        .file_name()
        .and_then(|file| file.to_str())
        .unwrap_or("<unnamed>");
    format!("docs/tutorials/{file}")
}

// --- the tutorial's own tiny shell -----------------------------------------

struct Shell {
    tutorial: String,
    cwd: PathBuf,
    last_exit: Option<i32>,
}

impl Shell {
    /// Build the book the reader was told to type, and insist it works.
    ///
    /// This is how a tutorial about the *window* is checked: it has no
    /// commands in it, but the files it tells somebody to create must make
    /// a book that opens and lays out. A `book.toml` with a typo in it, or
    /// a chapter in a folder Booker does not look in, fails here rather
    /// than in front of a reader.
    fn build_what_the_reader_typed(&mut self) {
        let output = Command::new(BOOKER)
            .arg("build")
            .arg(".")
            .current_dir(&self.cwd)
            .output()
            .unwrap_or_else(|error| panic!("{}: cannot run booker: {error}", self.tutorial));

        let printed = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.status.success(),
            "{}: the files this tutorial tells the reader to create do not make a book \
             that builds.\n{printed}",
            self.tutorial
        );
        assert!(
            printed.contains("Problems: none"),
            "{}: the book this tutorial tells the reader to create has problems in it.\n\
             A tutorial may walk somebody into a mistake on purpose — but then it must \
             show them the mistake, and this one does not reach that far.\n{printed}",
            self.tutorial
        );
    }

    fn write_file(&self, path: &str, fence: &Fence) {
        let target = self.cwd.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap_or_else(|error| {
                panic!(
                    "{}:{}: cannot create the folder for `{path}`: {error}",
                    self.tutorial, fence.start
                )
            });
        }
        let mut body: String = fence
            .lines
            .iter()
            .map(|(_, line)| line.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        body.push('\n');
        std::fs::write(&target, body).unwrap_or_else(|error| {
            panic!(
                "{}:{}: cannot write `{path}`: {error}",
                self.tutorial, fence.start
            )
        });
    }

    /// Run one ```console block. Returns how many commands it contained.
    fn run_block(&mut self, fence: &Fence) -> usize {
        let steps = steps(&self.tutorial, fence);
        let mut unchecked_failure: Option<(usize, i32)> = None;
        let mut count = 0;

        for step in &steps {
            if step.command == "echo $?" {
                self.check_exit_code(step);
                unchecked_failure = None;
                continue;
            }
            self.report_unchecked(unchecked_failure.take());

            let output = self.run(step);
            self.compare(step, &output);
            count += 1;

            if let Some(code) = self.last_exit.filter(|code| *code != 0) {
                unchecked_failure = Some((step.line, code));
            }
        }
        self.report_unchecked(unchecked_failure);
        count
    }

    /// A command that failed without the tutorial saying so is a tutorial that
    /// will teach someone to ignore a failure.
    fn report_unchecked(&self, failure: Option<(usize, i32)>) {
        if let Some((line, code)) = failure {
            panic!(
                "{}:{line}: this command exited with code {code}, and the tutorial does not say \
                 so.\n  If that is the lesson, follow it with:\n      $ echo $?\n      {code}",
                self.tutorial
            );
        }
    }

    fn check_exit_code(&self, step: &Step) {
        let code = self.last_exit.unwrap_or_else(|| {
            panic!(
                "{}:{}: `echo $?` before any command has run",
                self.tutorial, step.line
            )
        });
        let claimed = step
            .expected
            .first()
            .map(|(_, line)| line.trim().to_string())
            .unwrap_or_default();
        assert_eq!(
            claimed,
            code.to_string(),
            "{}:{}: the tutorial says the previous command exited with `{claimed}`, but it \
             exited with `{code}`",
            self.tutorial,
            step.expected.first().map_or(step.line, |(line, _)| *line)
        );
    }

    fn run(&mut self, step: &Step) -> String {
        let arguments = split(&step.command);
        let program = arguments.first().map(String::as_str).unwrap_or_default();
        match program {
            "booker" => {
                let output = Command::new(BOOKER)
                    .args(&arguments[1..])
                    .current_dir(&self.cwd)
                    .output()
                    .unwrap_or_else(|error| {
                        panic!(
                            "{}:{}: `{}` could not be started: {error}",
                            self.tutorial, step.line, step.command
                        )
                    });
                self.last_exit = output.status.code();
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&output.stderr));
                text
            }
            "cd" => {
                let target = self.cwd.join(argument(self, step, &arguments));
                assert!(
                    target.is_dir(),
                    "{}:{}: `{}` — there is no such folder at this point in the tutorial",
                    self.tutorial,
                    step.line,
                    step.command
                );
                self.cwd = target;
                self.last_exit = Some(0);
                String::new()
            }
            "rm" => {
                let target = self.cwd.join(argument(self, step, &arguments));
                std::fs::remove_file(&target).unwrap_or_else(|error| {
                    panic!(
                        "{}:{}: `{}` — {error}",
                        self.tutorial, step.line, step.command
                    )
                });
                self.last_exit = Some(0);
                String::new()
            }
            // Deliberately not a shell. A command nobody checks is how a
            // tutorial starts lying.
            _ => panic!(
                "{}:{}: `{}` is not a command this check knows how to run.\n  It understands \
                 `booker …`, `cd …`, `rm …` and `echo $?`.\n  If the reader should see it but it \
                 cannot be run here, mark the block ```console ignore.",
                self.tutorial, step.line, step.command
            ),
        }
    }

    fn compare(&self, step: &Step, actual: &str) {
        let actual: Vec<&str> = actual
            .lines()
            .map(|line| line.trim_end_matches('\r'))
            .collect();

        for (index, printed) in actual.iter().enumerate() {
            let Some((line, expected)) = step.expected.get(index) else {
                panic!(
                    "{}:{}: `{}` printed a line the tutorial does not have:\n      {printed}\n  \
                     Copy what it really prints, using `…` where the value varies.",
                    self.tutorial,
                    step.expected.last().map_or(step.line, |(line, _)| *line),
                    step.command
                );
            };
            assert!(
                matches(expected, printed),
                "{}:{line}: the tutorial says this line is printed:\n      {expected}\n  but \
                 `{}` printed:\n      {printed}",
                self.tutorial,
                step.command
            );
        }

        if let Some((line, expected)) = step.expected.get(actual.len()) {
            panic!(
                "{}:{line}: the tutorial says this line is printed:\n      {expected}\n  but \
                 `{}` printed nothing more.",
                self.tutorial, step.command
            );
        }
    }
}

fn argument(shell: &Shell, step: &Step, arguments: &[String]) -> String {
    assert_eq!(
        arguments.len(),
        2,
        "{}:{}: `{}` takes exactly one path here",
        shell.tutorial,
        step.line,
        step.command
    );
    arguments[1].clone()
}

// --- parsing ---------------------------------------------------------------

/// One command in a ```console block, with the lines it claims to print.
struct Step {
    line: usize,
    command: String,
    expected: Vec<(usize, String)>,
}

fn steps(tutorial: &str, fence: &Fence) -> Vec<Step> {
    let mut steps: Vec<Step> = Vec::new();
    for (line, text) in &fence.lines {
        if let Some(command) = text.strip_prefix("$ ") {
            steps.push(Step {
                line: *line,
                command: command.trim().to_string(),
                expected: Vec::new(),
            });
        } else {
            let step = steps.last_mut().unwrap_or_else(|| {
                panic!(
                    "{tutorial}:{line}: a ```console block starts with output rather than a \
                     `$ ` command"
                )
            });
            step.expected.push((*line, text.clone()));
        }
    }
    steps
}

/// A fenced block, with the line numbers it came from so a failure can point
/// at the source rather than at a copy of it.
struct Fence {
    info: String,
    start: usize,
    lines: Vec<(usize, String)>,
}

impl Fence {
    /// `console ignore` → `("console", ["ignore"])`.
    fn info(&self) -> (&str, Vec<&str>) {
        let mut words = self.info.split_whitespace();
        (words.next().unwrap_or_default(), words.collect())
    }
}

/// Fenced blocks, CommonMark's rule for the closing fence: at least as many
/// backticks as opened it. `docs/tutorials/index.md` documents the convention
/// inside four-backtick blocks, and they must not be mistaken for it.
fn fences(text: &str) -> Vec<Fence> {
    let mut out = Vec::new();
    let mut open: Option<(usize, String, usize)> = None;
    let mut lines: Vec<(usize, String)> = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let ticks = line
            .chars()
            .take_while(|character| *character == '`')
            .count();
        match &open {
            None => {
                if ticks >= 3 {
                    open = Some((ticks, line[ticks..].trim().to_string(), number));
                    lines.clear();
                }
            }
            Some((opened, _, _)) => {
                if ticks >= *opened && line[ticks..].trim().is_empty() {
                    let (_, info, start) = open.take().expect("the fence is open");
                    out.push(Fence {
                        info,
                        start,
                        lines: std::mem::take(&mut lines),
                    });
                } else {
                    lines.push((number, line.to_string()));
                }
            }
        }
    }
    out
}

/// Split a command line the way a shell would, honouring quotes so that
/// `--title "The Moon in a Jar"` arrives as one argument.
fn split(command: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;

    for character in command.chars() {
        match quote {
            Some(open) if character == open => quote = None,
            Some(_) => current.push(character),
            None if character == '"' || character == '\'' => {
                quote = Some(character);
                started = true;
            }
            None if character.is_whitespace() => {
                if started {
                    arguments.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            None => {
                current.push(character);
                started = true;
            }
        }
    }
    if started {
        arguments.push(current);
    }
    arguments
}

/// Does what Booker printed match what the tutorial claims, allowing `…` to
/// stand for anything?
fn matches(expected: &str, actual: &str) -> bool {
    if !expected.contains(WILDCARD) {
        return expected == actual;
    }
    let parts: Vec<&str> = expected.split(WILDCARD).collect();
    let (first, last) = (parts[0], parts[parts.len() - 1]);

    let Some(mut rest) = actual.strip_prefix(first) else {
        return false;
    };
    for part in &parts[1..parts.len() - 1] {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    rest.ends_with(last)
}

#[cfg(test)]
mod wildcard {
    use super::matches;

    #[test]
    fn a_line_without_a_wildcard_must_be_exact() {
        assert!(matches("Problems: none", "Problems: none"));
        assert!(!matches("Problems: none", "Problems: 1 error, 0 warnings"));
    }

    #[test]
    fn a_wildcard_stands_for_anything_between_what_is_written() {
        assert!(matches("2 pages in … ms", "2 pages in 27 ms"));
        assert!(matches("2 pages in … ms", "2 pages in 1043 ms"));
        assert!(!matches("2 pages in … ms", "3 pages in 27 ms"));
        assert!(!matches("2 pages in … ms", "2 pages in 27 seconds"));
    }

    #[test]
    fn a_wildcard_does_not_let_the_rest_of_the_line_drift() {
        assert!(!matches(
            "Built build/… — 2 pages",
            "Built dist/x.pdf — 2 pages"
        ));
        assert!(matches(
            "Built build/… — 2 pages",
            "Built build/x.pdf — 2 pages"
        ));
    }
}
