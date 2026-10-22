# Security Policy

Ferrite is an early-stage, community-maintained project. There is no
dedicated security team, paid support contract, or private disclosure
mailing list at this time — reports are handled by whoever is
maintaining the project when they come in, on a best-effort basis.

## Reporting a vulnerability

- **If GitHub's private vulnerability reporting is enabled for this
  repository**, use the "Report a vulnerability" button under the
  repository's **Security** tab. This sends the report privately to the
  maintainers without disclosing it publicly.
- **If that option isn't available**, or you're unsure, open a regular
  [GitHub issue](https://github.com/shrenixcodes/ferrite/issues) and
  describe the problem without including exploit details that would put
  other users at risk before a fix ships. A maintainer will follow up to
  coordinate a more private channel if the report turns out to be
  sensitive.

Please include:
- What you were running (Ferrite version, OS, terminal emulator).
- Steps to reproduce.
- What you expected to happen versus what actually happened.
- Why you believe it has security impact (not just a bug).

## Scope

Ferrite is a local, terminal-native system monitor. It:
- Reads system metrics (CPU, memory, disk, network counters, process
  list) via the [`sysinfo`](https://docs.rs/sysinfo) crate.
- Optionally reads a local TOML configuration file that you control (see
  the README's Configuration section); it never writes, creates, or
  transmits that file anywhere.
- Has no network server, no remote telemetry, and no privilege
  escalation of any kind — it only reads what the OS already exposes to
  an unprivileged process.

Given that scope, realistic security concerns are things like: a crash
or panic triggered by untrusted input (a malformed config file, unusual
process/interface names, extreme terminal dimensions), a path-handling
bug in configuration file resolution, or a dependency with a known
vulnerability that Ferrite actually exercises. Issues like "it reports
information about my own machine" are expected behavior, not a
vulnerability — that's the point of a system monitor running as your own
user.

## Supported versions

Ferrite does not yet have a stable 1.0 release or a long-term support
policy. Security fixes will target the latest released version; there is
no commitment yet to backport fixes to older tags.

## Dependency vulnerabilities

Dependencies are reviewed manually for now (see `CONTRIBUTING.md`); this
project does not yet run automated supply-chain scanning (e.g.
`cargo audit`) in CI. If you notice a dependency with a known advisory
that affects Ferrite, please report it the same way as above.
