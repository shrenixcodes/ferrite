# Contributing to Ferrite

Thanks for your interest in contributing. This project is in an early,
foundational stage, so changes should stay focused and well-tested.

## Setting up your environment

1. Install a stable Rust toolchain via [rustup](https://rustup.rs/)
   (edition 2024 requires Rust 1.85 or later).
2. Clone the repository and build it:

   ```bash
   cargo build
   ```

## Checks to run before submitting changes

Run all of the following locally and make sure they pass:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
```

These are the same checks enforced by continuous integration. A pull request
that fails any of them will not be merged.

## Keeping changes focused

- Keep each pull request scoped to a single logical change.
- Avoid bundling unrelated refactors, formatting-only changes, or new
  features into the same pull request as a bug fix.
- Avoid introducing new dependencies unless the change clearly requires one;
  explain the justification in the pull request description.
- Do not add speculative abstractions for features that are not yet in
  scope.

## Writing and running tests

- Add unit tests alongside the code they exercise (see `src/config.rs` for
  an example) and integration tests under `tests/` for public API behavior.
- Tests should assert concrete return values and error conditions, not just
  that a function runs without panicking.
- Prefer deterministic tests. Avoid relying on system load, timing, network
  access, or terminal availability.
- Run `cargo test --all-targets` before submitting.

## Preparing a pull request

- Write a clear, specific commit message and pull request description
  explaining what changed and why.
- Reference any related issue.
- Confirm the checklist above passes before requesting review.
