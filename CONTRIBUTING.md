# Contributing to Ferrite

Thanks for your interest in contributing. Ferrite's core metrics engine,
dashboard, history, and alerting are implemented and tested; contributions
should build on that foundation rather than restructuring it without
discussion.

## Setting up your environment

1. Install a stable Rust toolchain via [rustup](https://rustup.rs/)
   (edition 2024 requires Rust 1.85 or later).
2. Clone the repository and build it:

   ```bash
   cargo build
   ```
3. Run it directly from the repo with `cargo run` (needs a real terminal —
   see the README's Troubleshooting section if it fails to start).

## Checks to run before submitting changes

Run all of the following locally and make sure they pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

These are the same checks enforced by continuous integration, which runs
on Linux, Windows, and macOS (see `.github/workflows/ci.yml`). A pull
request that fails any of them will not be merged.

## Keeping changes focused

- Keep each pull request scoped to a single logical change.
- Avoid bundling unrelated refactors, formatting-only changes, or new
  features into the same pull request as a bug fix.
- Avoid introducing new dependencies unless the change clearly requires one;
  explain the justification in the pull request description.
- Do not add speculative abstractions for features that are not yet in
  scope.

## Writing and running tests

- Add unit tests alongside the code they exercise (see `src/config/mod.rs`
  or `src/collector/network.rs` for examples of pure, dependency-free
  logic with thorough unit tests) and integration tests under `tests/`
  for public API behavior.
- Tests should assert concrete return values and error conditions, not just
  that a function runs without panicking.
- Prefer deterministic tests. Avoid relying on system load, timing, network
  access, or terminal availability. Where a real clock is unavoidable
  (e.g. history timestamps), use `Instant` arithmetic (`t0 + Duration`)
  rather than real `sleep` calls so tests stay fast and deterministic.
- Run `cargo test --all-targets` before submitting.

## Benchmarking performance-sensitive changes

If your change touches sorting, rate calculation, or history (see
`benches/core.rs`), run the relevant benchmark before and after your
change:

```bash
cargo bench
```

See [BENCHMARKING.md](BENCHMARKING.md) for what's covered and how to read
the output. Benchmark results are not enforced in CI (they're too
machine-dependent to gate on), but a large regression is worth
investigating before you open a pull request.

## Platform-specific contributions

Ferrite targets Linux, Windows, and macOS. If you're fixing or extending
platform-specific code (currently limited to configuration file path
resolution in `src/config/file.rs`), add a `#[cfg]`-gated unit test for
that platform rather than only testing on your own machine — see the
existing `windows_config_dir_uses_appdata`,
`linux_config_dir_prefers_xdg_config_home`, and
`macos_config_dir_uses_home_library` tests for the pattern. CI will run
your test on the matching OS even if you can't locally.

## Preparing a pull request

- Write a clear, specific commit message and pull request description
  explaining what changed and why.
- Reference any related issue.
- Confirm the checklist above passes before requesting review.
