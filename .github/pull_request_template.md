<!-- What this changes, and why. The wave and track if it is not obvious. -->

## Checklist

- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass locally
- [ ] Tests cover what changed — a bug fix starts with a test that failed before it
- [ ] The documents in `AGENTS.md` §2 describe what the software does now, the README included
- [ ] User-visible errors name the file, and the line where that applies
- [ ] Nothing generated is committed (`build/`, `.booker/`, `app/src/lib/bindings/`)
