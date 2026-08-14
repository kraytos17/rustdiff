# Contributing

Thanks for contributing to `rustdiff`!

## Commit conventions

Release notes and the changelog are generated from commit messages with
[`git-cliff`](https://git-cliff.org), so please follow the
[Conventional Commits](https://www.conventionalcommits.org) spec:

```
feat: add --ignore-blank-lines
fix(html): escape filenames in <title>
perf(core): coalesce ops in place
refactor: ...
docs: ...
test: ...
ci: ...
chore: ...
```

- **Breaking changes**: add `!` after the type (`feat!: ...`) or a
  `BREAKING CHANGE:` footer line.
- **Release commits** (`release x.y.z`) are skipped from the notes
  automatically — keep using them when cutting a release.

Commit messages should describe *what changed and why*, in the imperative
mood ("add", not "added").

## Releasing

```sh
cargo set-version 0.8.0        # bump Cargo.toml + Cargo.lock
git cliff -o CHANGELOG.md     # regenerate the changelog
git commit -m "release 0.8.0"
git tag v0.8.0
git push --tags               # CI builds binaries + publishes release notes
```

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings -W clippy::pedantic -W clippy::nursery
cargo test --all-targets
cargo test --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

Fuzzing (nightly): `cargo +nightly fuzz run diff_round_trip` and
`cargo +nightly fuzz run render_round_trip`.
