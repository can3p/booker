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
//! `deps` CI job checks against `deny.toml`'s allow list — and writes the
//! file. Run it with `cargo xtask third-party`; `--check` fails instead of
//! writing, which is what CI uses to notice a stale file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Where the notices live, relative to the repository root.
const OUTPUT: &str = "THIRD-PARTY.md";

/// The font notices, which are not in the Cargo tree: the faces are bytes
/// compiled into `booker-typst`, so nothing but this file can mention them.
const FONT_NOTICE: &str = "crates/booker-typst/fonts/NOTICE.txt";

/// Write the file, or check that it is current.
pub fn run(check: bool) -> Result<()> {
    let root = repository_root()?;
    let generated = assemble(&root)?;
    let path = root.join(OUTPUT);

    if check {
        let existing = std::fs::read_to_string(&path)
            .with_context(|| format!("{OUTPUT} is not there; run `cargo xtask third-party`"))?;
        if existing != generated {
            bail!(
                "{OUTPUT} is out of date — a dependency changed. \
                 Run `cargo xtask third-party` and commit the result."
            );
        }
        println!("{OUTPUT} is up to date");
        return Ok(());
    }

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

fn assemble(root: &Path) -> Result<String> {
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
         build *contains* is listed below: the fonts it embeds and every \
         Rust crate it is compiled from, with the licence each one carries. \
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
    fn the_registry_url_is_dropped() {
        assert_eq!(
            short_name("serde 1.0.1 registry+https://github.com/rust-lang/crates.io-index"),
            "serde 1.0.1"
        );
    }
}
