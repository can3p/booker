//! How a path is written wherever Booker shows it to anyone.

use std::path::Path;

/// The way a project-relative path appears in anything Booker prints: with
/// forward slashes, on every platform.
///
/// This is not cosmetic. `book.toml` spells chapter paths with `/` whoever
/// wrote it, so a message that says `content\01.md` disagrees with the file
/// the user would edit; and `AGENTS.md` §6 asks for output an agent can
/// compare across platforms — the CLI and the MCP server must print one
/// string for one file, not one per operating system. Anything that is not a
/// relative path (an absolute one, with a root or a drive prefix) is shown
/// the way the platform writes it, because inventing a form for it would be
/// worse than showing the real thing.
pub fn display_path(path: &Path) -> String {
    use std::path::Component;

    let mut shown = String::new();
    for component in path.components() {
        let part = match component {
            Component::Normal(part) => part.to_string_lossy(),
            Component::CurDir => continue,
            Component::ParentDir => std::borrow::Cow::Borrowed(".."),
            Component::RootDir | Component::Prefix(_) => {
                return path.to_string_lossy().into_owned()
            }
        };
        if !shown.is_empty() {
            shown.push('/');
        }
        shown.push_str(&part);
    }
    shown
}
#[cfg(test)]
mod tests {
    use super::display_path;
    use std::path::{Path, PathBuf};

    #[test]
    fn a_project_relative_path_reads_the_same_on_every_platform() {
        // Built by joining, which is where the separator comes from the
        // platform: this is the shape that made two Windows CI runs red.
        let built: PathBuf = ["assets", "fonts", "broken.ttf"].iter().collect();
        assert_eq!(display_path(&built), "assets/fonts/broken.ttf");

        // Written by a person in book.toml, which always uses `/`.
        assert_eq!(display_path(Path::new("content/01.md")), "content/01.md");

        // A leading `./` is noise in a message.
        assert_eq!(display_path(Path::new("./content/01.md")), "content/01.md");
    }

    #[test]
    fn an_absolute_path_is_shown_the_way_the_platform_writes_it() {
        let absolute = std::env::current_dir().expect("a working directory");
        assert_eq!(display_path(&absolute), absolute.to_string_lossy());
    }
}
