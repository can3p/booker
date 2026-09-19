//! The migration seam.
//!
//! There are no migrations yet — format 1 is the first format. The seam
//! exists now because of the rule it enforces (`AGENTS.md` §7): changing the
//! project format means bumping `FORMAT_VERSION` **and** adding the step
//! that carries every existing project across, in the same change. When that
//! day comes, the work is to write one `Migration` and add it to
//! [`Migrations::builtin`]; everything around it — ordering, the version
//! bump, refusing a project from the future — already exists and is tested.

use std::path::PathBuf;

use booker_core::{Error, Result, FORMAT_VERSION};
use toml_edit::DocumentMut;

use crate::config::BOOK_TOML;
use crate::edit::ConfigEditor;

/// One step from one format version to the next.
pub struct Migration {
    /// The version this step reads.
    pub from: u32,
    /// The version it produces. Always `from + 1`: small steps compose, and
    /// a project two versions behind is migrated twice rather than by a
    /// special case.
    pub to: u32,
    /// What it does, in a sentence, for the log and for the user.
    pub description: &'static str,
    /// The edit itself, over the same targeted-edit API the app uses, so a
    /// migration cannot reformat a file it was only meant to touch in one
    /// place.
    pub apply: fn(&mut ConfigEditor<'_>) -> Result<()>,
}

/// An ordered set of migration steps.
pub struct Migrations {
    steps: Vec<Migration>,
}

impl Migrations {
    /// The migrations this build ships. Empty: format 1 is the first.
    pub fn builtin() -> Migrations {
        Migrations { steps: Vec::new() }
    }

    pub fn new(steps: Vec<Migration>) -> Migrations {
        Migrations { steps }
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The steps that carry a project at version `from` up to this build's
    /// format version, in the order they must run.
    ///
    /// Errors when the project is newer than this build, or when a step in
    /// the chain is missing — which would be a bug in Booker, not in the
    /// project, and must be loud.
    pub fn plan(&self, from: u32) -> Result<Vec<&Migration>> {
        if from > FORMAT_VERSION {
            return Err(Error::FormatTooNew {
                found: from,
                supported: FORMAT_VERSION,
            });
        }
        let mut plan = Vec::new();
        let mut version = from;
        while version < FORMAT_VERSION {
            let Some(step) = self.steps.iter().find(|step| step.from == version) else {
                return Err(Error::Project {
                    path: PathBuf::from(BOOK_TOML),
                    message: format!(
                        "this project is format {version}, and this build of Booker has no \
                         migration from {version} to {}",
                        version + 1
                    ),
                });
            };
            version = step.to;
            plan.push(step);
        }
        Ok(plan)
    }

    /// Run the plan against a document, leaving `format` set to the version
    /// that was reached. Returns the descriptions of the steps that ran, so
    /// the caller can tell the user what changed.
    pub fn run(&self, document: &mut DocumentMut, from: u32) -> Result<Vec<&'static str>> {
        let plan = self.plan(from)?;
        if plan.is_empty() {
            return Ok(Vec::new());
        }
        let mut applied = Vec::new();
        let mut editor = ConfigEditor::new(document);
        for step in plan {
            (step.apply)(&mut editor)?;
            editor.set_format(step.to);
            applied.push(step.description);
        }
        Ok(applied)
    }
}

impl Default for Migrations {
    fn default() -> Self {
        Migrations::builtin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stand-in chain, so the seam is exercised before there is anything
    /// real to migrate. When the first real migration lands, this test keeps
    /// working and the real one is added beside it.
    fn pretend_chain() -> Migrations {
        Migrations::new(vec![
            Migration {
                from: 0,
                to: 1,
                description: "rename `name` to `title`",
                apply: |editor| {
                    editor.rename(&["name"], "title");
                    Ok(())
                },
            },
            Migration {
                from: 1,
                to: 2,
                description: "move `paper` into `[page] size`",
                apply: |editor| {
                    if let Some(paper) = editor.get_string(&["paper"]) {
                        editor.set_string(&["page", "size"], &paper);
                        editor.remove(&["paper"]);
                    }
                    Ok(())
                },
            },
        ])
    }

    #[test]
    fn this_build_ships_no_migrations_and_needs_none() {
        let migrations = Migrations::builtin();
        assert!(migrations.is_empty());
        assert!(
            migrations
                .plan(FORMAT_VERSION)
                .map(|plan| plan.is_empty())
                .unwrap_or(false),
            "a current project is not migrated"
        );
    }

    #[test]
    fn a_project_from_the_future_is_named_as_such_rather_than_guessed_at() {
        let error = Migrations::builtin()
            .plan(FORMAT_VERSION + 5)
            .err()
            .expect("a project from the future is refused");
        assert!(
            matches!(error, Error::FormatTooNew { found, supported }
                if found == FORMAT_VERSION + 5 && supported == FORMAT_VERSION),
            "{error}"
        );
    }

    #[test]
    fn a_missing_step_is_an_error_and_not_a_silent_skip() {
        let migrations = Migrations::new(Vec::new());
        // Pretend this build is ahead of a project it has no step for.
        let error = migrations
            .plan(FORMAT_VERSION - 1)
            .err()
            .expect("a gap in the chain is an error");
        assert!(format!("{error}").contains("no migration"), "{error}");
    }

    #[test]
    fn steps_run_in_order_and_leave_the_format_version_correct() {
        // The chain runs up to this build's FORMAT_VERSION, whatever it is.
        let migrations = pretend_chain();
        let mut document: DocumentMut = "# my book\nname = \"Mia\"\npaper = \"a5\"\n"
            .parse()
            .unwrap();
        let applied = migrations.run(&mut document, 0).expect("the chain runs");
        assert_eq!(applied.len(), FORMAT_VERSION as usize);
        let text = document.to_string();
        assert!(
            text.contains("# my book\ntitle = \"Mia\""),
            "the comment stays with the key it was written for: {text}"
        );
        assert!(
            text.contains(&format!("format = {FORMAT_VERSION}")),
            "the version is bumped by the migration itself: {text}"
        );
    }

    #[test]
    fn a_migration_touches_only_what_it_was_written_to_touch() {
        let migrations = pretend_chain();
        let mut document: DocumentMut =
            "name = \"Mia\"   # working title\nlanguage = \"fr\"\n\n[page]\nfacing = false\n"
                .parse()
                .unwrap();
        migrations.run(&mut document, 0).expect("the chain runs");
        let text = document.to_string();
        assert!(text.contains("# working title"), "{text}");
        assert!(text.contains("language = \"fr\""), "{text}");
        assert!(text.contains("facing = false"), "{text}");
    }
}
