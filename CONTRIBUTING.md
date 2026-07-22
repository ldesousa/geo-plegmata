Contributing to GeoPlegma
=========================

We welcome contributions to GeoPlegma and its associated code projects. These
can be in the form of issues, bug fixes, documentation or suggestions for
enhancements. This document sets out our guidelines and best practices for such
contributions.

## Code of Conduct

Contributors to this project are expected to act respectfully toward others in
accordance with the [Code of Conduct](CODE_OF_CONDUCT.md).

## Issues 

Any issue that is related to a implementation with a PR, should have the tag
`Feature`, `Task`, `Bug`. These tags are part of the default tag set from GitHub
with familiar meaning. These type of issues should also be related to a item on
the [kanban
board](https://github.com/GieoPlegma/GeoPlegma/projects?query=is%3Aopen). Other
matters that may not fit exactly with these familar tags should be tagged as
`Suggestion`.

## Branch model

GeoPlegma uses two long-lived branches:

- `develop` — the integration branch. All feature/fix/task work lands here first.
- `master` — always releasable. Nobody commits to it directly; it only moves
  forward via a "release PR" from `develop`, and via tags (see
  [Releasing](#releasing) below).

Both branches are protected on GitHub:

- **`master`**: requires a PR before merging, requires the `ci` status check to
  pass, requires at least 1 approving review, requires linear history, and
  disallows direct or force pushes.
- **`develop`**: requires the `ci` status check to pass before merging; review
  is not required.

(These are configured directly in the repo's branch protection settings by a
maintainer with admin rights — there's nothing to set up locally.)

Note: `cargo fmt --check` and `cargo clippy` currently run as advisory-only
(`continue-on-error`) in CI, since the codebase predates these checks and
isn't fully clean yet. Only `cargo test` is a hard requirement today. Once a
cleanup pass lands, fmt/clippy will flip to blocking too — see the `TODO`
comments in `.github/workflows/ci.yml`.

## PR Protocol

Considering the growing complexity of this project, any pull request must be
well traceable and linked to existing procedures. In particular, it must address
a known issue and provide accompanying documentation. To make sure a pull
request adheres to these requirements please follow these steps:

1. The implementation needs to be associated with an item in the project [kanban
   board](https://github.com/GieoPlegma/GeoPlegma/projects?query=is%3Aopen).
2. Create a new branch off `develop` (not `master`) and don't forget to always
   pull first:
```
git checkout develop
git pull origin develop
git checkout -b <name of the branch>
```
3. The name of the branch could have the initial context, which would be
   `feature`, `task`, `bug`, `refactor`, `hotfix`, something like:
   `feature/<branch_name>`
4. Make your changes and commit them. CI (`fmt`, `clippy`, `test`) runs on
   every PR — make sure it's green before requesting review.
5. Open a PR **into `develop`**. Add this checklist to the PR (I will create a
   template message so you dont need to add anything):
    - [ ] Link PR to the issue and kanban board item.
    - [ ] Write a list of what was done.
    - [ ] Add README documentation of what's done in the PR, if needed.
    - [ ] Request review
6. Wait for the review and any changes the reviewer(s) may require.
7. Squash and merge (never choose the other options, we dont want to join commit
history from the PR branch).

### Contributor Licence Agreement

Your contribution will be under the project licencing [licence](LICENCE.md) as
per [GitHub's terms of
service](https://help.github.com/articles/github-terms-of-service/#6-contributions-under-repository-license).

## Releasing

All publishable crates in the workspace share a single version, set once in
`[workspace.package].version` in the root `Cargo.toml`. To cut a release:

1. Open a "release PR" from `develop` into `master`. Unlike feature PRs, merge
   this one with a regular merge commit (not squash), so `master`'s history
   shows the batch of already-squashed commits that make up the release.
2. Once merged, from an up-to-date local `master`, run
   [`cargo-release`](https://github.com/crate-ci/cargo-release):
   ```
   cargo release <patch|minor|major>
   ```
   This bumps the shared version, commits, tags `vX.Y.Z`, and pushes the tag
   (config lives in `release.toml`).
3. The pushed tag triggers `.github/workflows/release.yml`, which re-runs
   `cargo test`/`cargo clippy`, publishes the crates that are eligible (any
   crate marked `publish = false` in its own `Cargo.toml` is skipped — today
   that's `geoplegma` and `gp-encoding`, blocked on unreleased git
   dependencies), and creates a GitHub Release with auto-generated notes.
   It also triggers the separate `release-js.yml` workflow, which builds the
   JS native bindings.



