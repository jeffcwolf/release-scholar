# Quick Start Guide

This guide assumes you've already installed `release-scholar` and configured:
- Global config with your author info (`~/.config/release-scholar/config.toml` or `~/Library/Application Support/release-scholar/config.toml`)
- Zenodo API tokens (`token` and optionally `sandbox-token`)
- Forge credentials in global config if using mirrors

If you haven't done this yet, see the [First-Time Setup](README.md#first-time-setup) section in the README.

---

## Workflow: Publishing Your First Release

### 1. Initialize metadata files

```bash
cd /path/to/your/project
release-scholar init --project-dir .
```

This creates (if missing):
- `CITATION.cff` — citation metadata
- `CHANGELOG.md` — changelog template
- `LICENSE` — Apache-2.0 license
- `.release-scholar.toml` — project config

### 2. Edit CITATION.cff

Open `CITATION.cff` and fill in:
- `title` — your project name
- `abstract` — brief description
- `repository-code` — your repo URL (e.g., `https://codeberg.org/username/project`)
- `version` — must match the git tag you'll create (e.g., `0.1.0` for tag `v0.1.0`)
- `keywords` — include your programming language(s) and relevant topics
- `license` — SPDX identifier (e.g., `Apache-2.0`, `MIT`, `GPL-3.0`)

Author info is pre-filled from your global config.

### 3. Update CHANGELOG.md

Add your release notes under `[0.1.0]` (or whatever version you're releasing).

### 4. Commit and tag

```bash
git add CITATION.cff CHANGELOG.md LICENSE .release-scholar.toml
git commit -m "Add scholarly release metadata for v0.1.0"
git tag v0.1.0
```

The tag **must** be semver format: `vX.Y.Z` (the `v` prefix is required).

### 5. Validate

```bash
release-scholar check --project-dir .
```

Review the output:
- **[FAIL]** items must be fixed before publishing
- **[WARN]** items are advisory but should be reviewed
- Common warnings: missing `.gitignore` patterns, large files, `.DS_Store` files

Fix any failures, then re-run `check` until everything passes.

### 6. Build release bundle

```bash
release-scholar build --project-dir .
```

Creates `release/vX.Y.Z/` with:
- `project-vX.Y.Z.tar.gz` — deterministic archive
- `checksums.txt` — SHA256 hash
- `metadata.json` — Zenodo-ready metadata
- `CITATION.cff` — copy for reference

### 7. Publish to Zenodo

**Option A: Test on sandbox first (recommended for first use)**
```bash
release-scholar publish --project-dir . --sandbox
```

**Option B: Create production draft (reserves DOI, doesn't publish yet)**
```bash
release-scholar publish --project-dir .
```

**Option C: Publish to production (mints permanent DOI)**
```bash
release-scholar publish --project-dir . --confirm
```

After `--confirm` succeeds, a DOI badge is automatically added to your `README.md`.

### 8. Push to your forge

```bash
git add README.md
git commit -m "Add DOI badge for v0.1.0"
git push origin main
git push origin v0.1.0
```

### 9. (Optional) Set up forge mirrors

If you want to mirror from Codeberg → GitHub/GitLab (or create mirrors on other forges):

**First, create empty repos on target forges** with the same name (no README, no license, completely blank).

**Then run:**
```bash
release-scholar mirror --project-dir .
```

This uses your global config credentials to set up automatic push mirrors. Mirrors sync on every push and every 8 hours.

---

## Next Release

For subsequent releases:

1. Update `CHANGELOG.md` with new changes
2. Update `version` in `CITATION.cff` to match your new tag
3. Update `date-released` in `CITATION.cff`
4. Commit, tag (e.g., `v0.2.0`)
5. Run `check` → `build` → `publish --confirm`
6. Push

---

## Common Issues

**"HEAD has no semver tag"** — You forgot to create the git tag, or it's not on HEAD. Run `git tag v0.1.0` (matching the version in CITATION.cff).

**"version in CITATION.cff (0.1.0) does not match git tag (v0.2.0)"** — The version field in CITATION.cff should be `0.1.0` (no `v` prefix), while the git tag should be `v0.1.0` (with `v` prefix).

**"Working directory has uncommitted changes"** — Commit or stash your changes before running `check` or `build`.

**Security warnings about passwords** — False positives are common in web apps. Review the flagged files; if they're legitimate code (not hardcoded secrets), you can proceed.

**"Missing .gitignore patterns"** — Add recommended patterns to `.gitignore` (especially `release/`, `*.pem`, `*.key`, `target/`, `__pycache__/`, etc.).

**Mirror setup fails with 403** — Check that:
- Target repos exist and are empty
- Tokens have correct scopes (`repo` for GitHub, `write_repository` for GitLab, `repository` for Codeberg)
- Usernames in global config match your actual forge usernames

---

## Full Documentation

For detailed explanations, configuration options, and advanced usage, see the [README](README.md).
