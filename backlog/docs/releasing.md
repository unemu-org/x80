# Releasing

How to release `x80-core` to crates.io. The process is fixed in
[decision-21](<../decisions/decision-21 - Release-process-publishing-x80-core-to-crates-io.md>)
and [decision-22](<../decisions/decision-22 - GitHub-is-the-primary-forge.md>);
the [release workflow](../../.github/workflows/release.yml) and the
[CI section of commands](commands.md#ci) describe what runs. Only `x80-core` is
published. Releases are a maintainer step: an agent prepares the release commit,
and the maintainer tags, approves and publishes.

## One-time setup

1. **The `release` environment** on GitHub (Settings → Environments). It must
   exist with the maintainer as a required reviewer, or `verify` refuses every
   tag. "Prevent self-review" stays off, because the maintainer both tags and
   approves. Deployments are limited to tags matching `v*`. Done on 2026-11-16;
   check it with:

   ```sh
   gh api repos/unemu-org/x80/environments/release \
     --jq '[.protection_rules[] | {type, reviewers: [.reviewers[]?.reviewer.login]}]'
   ```

2. **The first version by hand.** crates.io accepts a trusted publisher only for
   a crate that already exists, so 0.1.0 is published from a local checkout; see
   [The first release](#the-first-release-010).
3. **The trusted publisher** on crates.io, after the first publication: the
   crate's Settings → Trusted Publishing → Add → GitHub, with owner `unemu-org`,
   repository `x80`, workflow `release.yml` and environment `release`. From then
   on the workflow publishes without a stored token.

## Every release

1. **Choose the version.** SemVer as Cargo applies it: before 1.0 a minor
   version may break the API and a patch version may not.
2. **Release commit on `main`**, landed like any bead:
   - set `version` in the root `Cargo.toml` (`[workspace.package]`);
   - refresh the lock file for the workspace members only:
     `cargo update --workspace --offline`; the gates use `--locked`;
   - date the section in `CHANGELOG.md`: `## [X.Y.Z] - Unreleased` becomes
     `## [X.Y.Z] - YYYY-MM-DD`, with the day of the tag;
   - wait for green CI on that commit.
3. **Rehearse** (optional, recommended after workflow changes): run
   `gh workflow run release.yml --ref main`. It runs the gates, the CPU test
   programs and `cargo publish --dry-run`, and publishes nothing.
4. **Tag and push:**

   ```sh
   git tag -s vX.Y.Z -m "x80-core X.Y.Z"
   git push origin vX.Y.Z
   ```

5. **Approve.** `verify`, `ci` and `cpu-tests` take five to ten minutes. Then
   `publish` waits in the Actions run under "Review deployments"; approve
   `release`.
6. **Check** <https://crates.io/crates/x80-core>, the GitHub Release and, a
   little later, the documentation on docs.rs.

After a release, open the next section as `## [X.Y.Z] - Unreleased` in
`CHANGELOG.md` when the first change worth listing lands.

## When something fails

- **Before approval** (`verify`, `ci` or `cpu-tests` failed): nothing is
  published. Fix it on `main`, then move the tag to the fixed commit:
  `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`, and tag again.
- **After publication**: never move or reuse the tag; crates.io never accepts the
  same version twice. "Re-run failed jobs" is safe: `publish` skips a version
  crates.io already has, and the GitHub Release step skips an existing release.
- **A broken published version**: release a fixed patch version. If the broken
  one must not be chosen any more, yank it with `cargo yank --version X.Y.Z
  x80-core`, using a short-lived API token. A yank does not delete the version.

## The first release (0.1.0)

1. Release commit, as above, plus one README change: the sentence
   "`x80-core` is not on crates.io yet; use it as a path dependency." becomes a
   dependency line such as `x80-core = "0.1"`.
2. On crates.io, create an API token with the scope `publish-new`, limited to the
   crate `x80-core`, with a short expiry.
3. From a clean checkout of that commit on macOS or Linux (the licence files in
   `x80-core/` are symlinks; a checkout without symlink support packages them
   wrongly), run `cargo publish -p x80-core --locked` with the token
   (`CARGO_REGISTRY_TOKEN` or `cargo login`).
4. Revoke the token, then configure the trusted publisher (one-time setup,
   item 3).
5. Tag `v0.1.0` and push it. The workflow runs every check, skips the upload
   because crates.io already has 0.1.0, and creates the GitHub Release. The
   approval in step 5 above is still required.

Trusted Publishing is first used by the release after 0.1.0.
