# Ferrite roadmap

Ferrite is pre-1.0 and actively evolving. This roadmap records the next
small, user-visible steps; it is a guide rather than a promise or release
schedule.

## Near term

- Add a small command-line option surface for selecting or inspecting a
  configuration file without changing the existing dashboard defaults.
- Improve manual terminal coverage on Linux and macOS and record the results
  before making cross-platform support claims stronger.
- Capture and publish a genuine dashboard screenshot from a supported,
  privacy-reviewed terminal session.

## Later

- Add opt-in data export with a documented format and tests for its stability.
- Consider decoupling collection from rendering if profiling shows that the
  synchronous refresh loop limits responsiveness.
- Gather real user feedback before committing to a stable 1.0 configuration
  or library API.

## Out of scope for now

Ferrite is not currently planned as a GUI, web dashboard, remote telemetry
service, or privileged monitoring agent. See the [README](README.md) for the
implemented feature set and [CONTRIBUTING.md](CONTRIBUTING.md) for ways to
help with focused improvements.
