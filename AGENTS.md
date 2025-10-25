# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` wires CLI parsing (via `clap`) into the domain logic.
- Feature flows live in `src/commands/` (`git_clean.rs`, `git_repos.rs`) and should stay thin, delegating to services.
- Git orchestration code sits in `src/git/` with shared helpers plus `tests.rs` for low-level fixtures; extend those when touching git plumbing.
- Cross-cutting utilities (`fs_utils.rs`, `env.rs`, `task_result.rs`, `ui.rs`) host reusable pieces; prefer augmenting these before adding new ad-hoc helpers.
- Installation helpers (`install.sh`, `update`) target local setups; build artifacts land in `target/` as usual for Cargo.

## Build, Test, and Development Commands
- `cargo fmt` – run before every commit to keep rustfmt-style formatting consistent.
- `cargo clippy --all-targets -- -D warnings` – lint for regressions; new code should pass cleanly.
- `cargo test` – execute the full suite, including the git module tests.
- `cargo run -- git-clean` / `cargo run -- git-repos` – exercise the CLI against a sandbox repository while developing.
- `./install.sh ~/.local/bin` – optional helper to install the binary in a custom prefix for manual testing.

## Coding Style & Naming Conventions
- Follow Rust 2024 defaults: four-space indentation, snake_case modules/functions, PascalCase types, SCREAMING_SNAKE_CASE constants.
- Keep modules focused; prefer `mod tests` colocated with implementation unless shared fixtures belong in `src/git/tests.rs`.
- Handle fallible operations with `anyhow::Result` and provide actionable context via `with_context`.
- User-facing prompts live in `ui.rs`; update copy there to keep messaging centralized.

## Testing Guidelines
- Favor deterministic unit tests that stub filesystem/git interactions via `tempfile` and helper builders in `src/git/tests.rs`.
- Name tests after the scenario, e.g., `cleans_branch_without_upstream`, and document edge cases with comments sparingly.
- Add regression tests whenever touching branch comparison logic or multi-repo traversal to prevent silent git changes.
- Run `cargo test -- --nocapture` when debugging interactive flows to inspect prompt text.

## Commit & Pull Request Guidelines
- Follow the existing history: short, imperative commit messages (`Add tests`, `Rename binary`) that describe the change’s intent.
- Ensure each commit formats (`cargo fmt`) and lints (`cargo clippy -D warnings`) cleanly before pushing.
- Pull requests should include: overview of the user-facing impact, mention of new commands/flags, test evidence (`cargo test` output), and screenshots/terminal excerpts for interactive flows when relevant.
- Cross-link related issues and call out any manual steps (e.g., rerunning `install.sh`) so downstream users know what to expect post-merge.
