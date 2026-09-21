//! Assemble `THIRD-PARTY.md` from the licence data `cargo deny` produces.
//!
//! Booker is MIT, but what it ships is not: the layout engine is
//! Apache-2.0, the bundled fonts have their own licences, and an installed
//! application carries a few hundred Rust crates with it. Those notices
//! have to be accurate, and a list kept by hand goes stale the first time
//! somebody adds a dependency.
//!
//! So this reads `cargo deny list --format json`, which already knows every
//! crate in the tree and what it is licensed under — the same data the
//! `deps` CI job checks against `deny.toml`'s allow list — and, for the
//! JavaScript bundled into the application window, `pnpm licenses list
//! --prod`, and writes the file. Run it with `cargo xtask third-party`;
//! `--check` fails instead of writing, which is what CI uses to notice a
//! stale file.
//!
//! The two halves need different tools — `cargo deny` for one, an installed
//! `app/node_modules` for the other — and CI has them in different jobs, so
//! `--only rust` and `--only javascript` check one part of the file each.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Where the notices live, relative to the repository root.
const OUTPUT: &str = "THIRD-PARTY.md";

/// The font notices, which are not in the Cargo tree: the faces are bytes
/// compiled into `booker-typst`, so nothing but this file can mention them.
const FONT_NOTICE: &str = "crates/booker-typst/fonts/NOTICE.txt";

/// Where the JavaScript section starts; everything before it is the fonts
/// and the Rust crates.
const JAVASCRIPT: &str = "\n## JavaScript in the application window\n";

/// Which part of the file to check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Part {
    /// The fonts and the Rust crates: needs `cargo deny`.
    Rust,
    /// The JavaScript bundled into the window: needs `app/node_modules`.
    Javascript,
}

/// Write the file, or check that it is current — all of it, or one part.
pub fn run(check: bool, only: Option<Part>) -> Result<()> {
    let root = repository_root()?;
    let path = root.join(OUTPUT);

    if check {
        let existing = std::fs::read_to_string(&path)
            .with_context(|| format!("{OUTPUT} is not there; run `cargo xtask third-party`"))?;
        let (rust, javascript) = existing
            .split_once(JAVASCRIPT)
            .map(|(rust, js)| (rust.to_string(), format!("{JAVASCRIPT}{js}")))
            .unwrap_or((existing.clone(), String::new()));
        let stale = match only {
            Some(Part::Rust) => rust != rust_part(&root)?,
            Some(Part::Javascript) => javascript != javascript_part(&root)?,
            None => existing != format!("{}{}", rust_part(&root)?, javascript_part(&root)?),
        };
        if stale {
            bail!(
                "{OUTPUT} is out of date — a dependency changed. \
                 Run `cargo xtask third-party` and commit the result."
            );
        }
        println!("{OUTPUT} is up to date");
        return Ok(());
    }

    let generated = format!("{}{}", rust_part(&root)?, javascript_part(&root)?);
    std::fs::write(&path, generated).with_context(|| format!("writing {}", path.display()))?;
    println!("wrote {}", path.display());
    Ok(())
}

/// The workspace root, from Cargo rather than from a guess about where this
/// was run.
fn repository_root() -> Result<PathBuf> {
    // `xtask` lives one level below the root, and `CARGO_MANIFEST_DIR` is
    // where its own manifest is.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(Path::to_path_buf)
        .context("xtask should live in a subfolder of the repository")
}

/// One licence and the crates under it.
type ByLicence = BTreeMap<String, Vec<String>>;

fn rust_part(root: &Path) -> Result<String> {
    let by_licence = licences(root)?;

    let mut out = String::new();
    out.push_str(header());

    out.push_str("## Fonts\n\n");
    out.push_str(&fonts(root)?);

    out.push_str("\n## Rust crates\n\n");
    out.push_str(
        "Every crate compiled into a Booker build, grouped by licence. \
         Generated from the dependency tree, so it describes this build \
         rather than a build somebody documented once.\n\n",
    );

    let total: usize = by_licence.values().map(Vec::len).sum();
    out.push_str(&format!(
        "{} crates under {} licences.\n",
        total,
        by_licence.len()
    ));

    for (licence, crates) in &by_licence {
        out.push_str(&format!("\n### {licence}\n\n"));
        for name in crates {
            out.push_str(&format!("- {name}\n"));
        }
    }

    Ok(out)
}

fn header() -> &'static str {
    "# Third-party notices\n\n\
         **This file is generated. Do not edit it by hand** — run \
         `cargo xtask third-party`, which reads the dependency tree and \
         rewrites it. CI fails when it is out of date.\n\n\
         Booker itself is MIT licensed (see `LICENSE`). What a Booker \
         build *contains* is listed below: the fonts it embeds, every \
         Rust crate it is compiled from, and the JavaScript packages \
         bundled into the application window, with the licence each one \
         carries. \
         A crate listed under several licences is offered under any of \
         them, at the user's choice, which is the usual Rust \
         dual-licensing.\n\n\
         The layout engine is [Typst](https://typst.app), Apache-2.0, and \
         appears below as the `typst-*` crates.\n\n\
         Source for any crate here is at \
         <https://crates.io>, and for the MPL-2.0 crates specifically — \
         whose licence asks that the source of those files be available — \
         at the repository each crate names on its crates.io page. Booker \
         uses them unmodified.\n\n\
         ---\n\n"
}

/// The bundled font notices, copied in rather than summarised: a licence
/// that has been paraphrased is not a licence.
fn fonts(root: &Path) -> Result<String> {
    let path = root.join(FONT_NOTICE);
    let notice = std::fs::read_to_string(&path)
        .with_context(|| format!("reading the font notices at {}", path.display()))?;
    Ok(format!(
        "Booker embeds its fonts in the binary, so they travel with every \
         build. Their notices, copied from `{FONT_NOTICE}`:\n\n```\n{}\n```\n",
        notice.trim_end()
    ))
}

/// The JavaScript that ships inside the application window: the app's
/// production dependencies and everything they pull in, as pnpm resolves
/// them. Development tools — Vite, TypeScript, the Tauri CLI — build the
/// window but are not in it, so they are not listed.
fn javascript_part(root: &Path) -> Result<String> {
    let app = root.join("app");
    let output = Command::new("pnpm")
        .args(["licenses", "list", "--prod", "--json"])
        .current_dir(&app)
        .output()
        .context(
            "running `pnpm licenses list`; the JavaScript half of the notices needs pnpm \
             and an installed `app/node_modules` (`cd app && pnpm install`)",
        )?;
    if !output.status.success() {
        bail!(
            "pnpm licenses list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let by_licence = parse_pnpm(
        &String::from_utf8(output.stdout).context("pnpm printed something that is not UTF-8")?,
    )?;

    let mut out = String::from(JAVASCRIPT);
    out.push_str(
        "\nThe window is a web page bundled into the application. These are \
         the packages that end up in that bundle, grouped by licence, \
         generated from `app/package.json`'s production dependencies. \
         Source for each is on <https://www.npmjs.com>.\n\n",
    );
    let total: usize = by_licence.values().map(Vec::len).sum();
    out.push_str(&format!(
        "{} packages under {} licences.\n",
        total,
        by_licence.len()
    ));
    for (licence, packages) in &by_licence {
        out.push_str(&format!("\n### {licence}\n\n"));
        for name in packages {
            out.push_str(&format!("- {name}\n"));
        }
    }
    Ok(out)
}

/// Read `pnpm licenses list --json`: `{"<licence>": [{"name", "versions"},
/// …], …}`.
fn parse_pnpm(json: &str) -> Result<ByLicence> {
    let value: serde_json::Value =
        serde_json::from_str(json).context("pnpm printed something that is not JSON")?;
    let object = value
        .as_object()
        .context("pnpm's licence list is not an object; has its format changed?")?;
    let mut by_licence = ByLicence::new();
    for (licence, packages) in object {
        let packages = packages
            .as_array()
            .context("a pnpm licence entry is not a list")?;
        let mut names = Vec::new();
        for package in packages {
            let name = package
                .get("name")
                .and_then(|name| name.as_str())
                .context("a pnpm package has no name")?;
            let versions = package
                .get("versions")
                .and_then(|versions| versions.as_array())
                .map(|versions| {
                    versions
                        .iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            names.push(format!("{name} {versions}").trim().to_string());
        }
        names.sort_unstable();
        by_licence.insert(licence.clone(), names);
    }
    Ok(by_licence)
}

/// Ask `cargo deny` what is in the tree.
fn licences(root: &Path) -> Result<ByLicence> {
    let output = Command::new("cargo")
        .args(["deny", "list", "--format", "json"])
        .current_dir(root)
        .output()
        .context(
            "running `cargo deny`. Install it with `cargo install cargo-deny --locked`, \
             or `brew install cargo-deny`",
        )?;
    if !output.status.success() {
        bail!(
            "cargo deny list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    parse(
        &String::from_utf8(output.stdout)
            .context("cargo deny printed something that is not UTF-8")?,
    )
}

/// Read `cargo deny list --format json`.
///
/// Its shape is `{"licenses": [[<licence>, [<crate id>, …]], …]}`, where a
/// crate id is `name version source`. Only the first two are wanted.
fn parse(json: &str) -> Result<ByLicence> {
    let value: serde_json::Value =
        serde_json::from_str(json).context("cargo deny printed something that is not JSON")?;
    let entries = value
        .get("licenses")
        .and_then(|licenses| licenses.as_array())
        .context("cargo deny's output has no `licenses` array; has its format changed?")?;

    let mut by_licence = ByLicence::new();
    for entry in entries {
        let pair = entry
            .as_array()
            .context("a licence entry is not an array")?;
        let [licence, crates] = pair.as_slice() else {
            bail!("a licence entry is not a pair");
        };
        let licence = licence.as_str().context("a licence name is not a string")?;
        let crates = crates.as_array().context("a crate list is not an array")?;

        let mut names: Vec<String> = crates
            .iter()
            .filter_map(|crate_id| crate_id.as_str())
            .map(short_name)
            .collect();
        names.sort_unstable();
        names.dedup();
        by_licence.insert(licence.to_string(), names);
    }
    Ok(by_licence)
}

/// `serde 1.0.1 registry+https://…` becomes `serde 1.0.1`. The registry is
/// the same for every crate here and only makes the file harder to read.
fn short_name(crate_id: &str) -> String {
    crate_id
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_cargo_deny_prints() {
        let json = r#"{"licenses":[
            ["MIT",["serde 1.0.1 registry+https://github.com/rust-lang/crates.io-index"]],
            ["Apache-2.0",["typst 0.15.1 registry+https://github.com/rust-lang/crates.io-index",
                           "serde 1.0.1 registry+https://github.com/rust-lang/crates.io-index"]]
        ]}"#;

        let by_licence = parse(json).expect("it parses");

        assert_eq!(by_licence["MIT"], vec!["serde 1.0.1"]);
        assert_eq!(
            by_licence["Apache-2.0"],
            vec!["serde 1.0.1", "typst 0.15.1"],
            "crates are sorted, so two runs produce the same file"
        );
    }

    #[test]
    fn a_crate_under_two_licences_appears_under_both() {
        // Rust's usual dual licensing: the user chooses, so both notices
        // have to be there.
        let json = r#"{"licenses":[
            ["MIT",["anyhow 1.0.0 registry+x"]],
            ["Apache-2.0",["anyhow 1.0.0 registry+x"]]
        ]}"#;
        let by_licence = parse(json).unwrap();
        assert_eq!(by_licence["MIT"], vec!["anyhow 1.0.0"]);
        assert_eq!(by_licence["Apache-2.0"], vec!["anyhow 1.0.0"]);
    }

    #[test]
    fn a_changed_output_format_is_an_error_not_an_empty_file() {
        // The worst outcome would be silently writing notices for nothing.
        assert!(parse(r#"{"something-else":[]}"#).is_err());
        assert!(parse("not json at all").is_err());
    }

    #[test]
    fn reads_what_pnpm_prints() {
        let json = r#"{"MIT":[{"name":"svelte","versions":["5.57.1"]},{"name":"crelt","versions":["1.0.7"]}],
                       "Apache-2.0 OR MIT":[{"name":"@tauri-apps/api","versions":["2.11.1"]}]}"#;
        let by_licence = parse_pnpm(json).unwrap();
        assert_eq!(by_licence["MIT"], vec!["crelt 1.0.7", "svelte 5.57.1"]);
        assert_eq!(
            by_licence["Apache-2.0 OR MIT"],
            vec!["@tauri-apps/api 2.11.1"]
        );
        assert!(parse_pnpm("[]").is_err());
    }

    #[test]
    fn the_registry_url_is_dropped() {
        assert_eq!(
            short_name("serde 1.0.1 registry+https://github.com/rust-lang/crates.io-index"),
            "serde 1.0.1"
        );
    }
}
