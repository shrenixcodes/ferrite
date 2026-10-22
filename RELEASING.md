# Releasing Ferrite

Ferrite has not yet published a GitHub release. This document describes
the process for when a maintainer decides to cut one — it does not mean
one is imminent, and following it requires push access to the upstream
repository (`git push` and `git push --tags`), which not every
contributor has.

## What "cutting a release" does

Pushing a tag matching `v*.*.*` (for example `v0.2.0`) triggers
`.github/workflows/release.yml`, which:

1. Builds Ferrite in release mode for four targets
   (`x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`,
   `x86_64-apple-darwin`, `aarch64-apple-darwin`), re-running the full
   `fmt`/`clippy`/`test` suite for each target first.
2. Only if **every** target's checks and build succeed, publishes a
   GitHub release named after the tag with each binary attached, using
   `gh release create --generate-notes` with the automatically provided
   `GITHUB_TOKEN` (no manually configured secret is required).

If any target's checks fail, the release job never runs — nothing is
published from unverified code.

**Required repository setting:** under Settings → Actions → General →
Workflow permissions, "Read and write permissions" must be enabled (or
equivalent fine-grained permissions granting `contents: write`) for the
publish step to be allowed to create a release. If permissions are
locked to read-only at the organization level, the release job's
"Create GitHub release and upload assets" step will fail with a
permissions error — that's a configuration gap to fix in repository
settings, not something the workflow can route around.

## Release checklist

Before tagging:

- [ ] `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
      `cargo test --all-targets`, and `cargo build --release` all pass locally.
- [ ] CI is green on `main` for the commit you intend to release.
- [ ] `CHANGELOG.md`'s `[Unreleased]` section accurately reflects what's
      shipping; rename it to `[X.Y.Z] - YYYY-MM-DD` and start a fresh
      empty `[Unreleased]` section above it.
- [ ] `Cargo.toml`'s `version` matches the tag you're about to push
      (without the `v` prefix — e.g. `0.2.0` in `Cargo.toml` for tag
      `v0.2.0`).
- [ ] `Cargo.lock` is up to date (`cargo check` after any `Cargo.toml`
      edit) and committed.
- [ ] README's feature list and platform-support claims still match
      what's actually implemented and actually tested.
- [ ] No secrets, local paths, or generated artifacts are staged.

Tagging and pushing:

```bash
git tag v0.2.0
git push origin v0.2.0
```

Pushing the tag (not a commit to `main`) is what triggers the release
workflow. After it completes, verify the GitHub release was created with
all four binaries attached before announcing it anywhere.

## Versioning policy

Ferrite is pre-1.0: version numbers are `0.MINOR.PATCH`. Per [Semantic
Versioning](https://semver.org/#spec-item-4), anything may change at any
`0.x` release, including breaking changes to the configuration file
format or library API — there is no stability guarantee yet. A 1.0
release will only be considered once the project has real-world usage
feedback, verified (not just compiled) support on all three target
platforms, and a stable configuration/API surface.
