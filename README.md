# release-scholar

Validate, audit, and package scholarly software releases for Zenodo/DOI publication.

`release-scholar` automates the tedious parts of publishing research software: metadata validation, security auditing, deterministic archiving, and Zenodo deposit creation — so you get a DOI with minimal friction.

## Install

```bash
cargo install --path .
```

## Quick Start

### 1. Initialize metadata

```bash
cd /path/to/your/project
release-scholar init --project-dir .
```

This creates (if missing):
- **CITATION.cff** — citation metadata (pre-filled from git config)
- **CHANGELOG.md** — Keep a Changelog template
- **LICENSE** — Apache-2.0 (default)
- **.release-scholar.toml** — tool configuration

### 2. Edit your metadata

Open `CITATION.cff` and fill in:
- Your name, ORCID, and email
- Project title and description
- License (SPDX identifier)
- Repository URL
- Keywords (include programming languages here, e.g. `Python`, `Java`)

Example:
```yaml
cff-version: 1.2.0
title: "my-project"
type: software
authors:
  - family-names: "Smith"
    given-names: "Jane"
    email: "jane@example.com"
    orcid: "https://orcid.org/0000-0002-1234-5678"
version: "0.1.0"
license: Apache-2.0
date-released: "2026-02-16"
repository-code: "https://codeberg.org/user/my-project"
abstract: "A brief description of what this software does."
keywords:
  - research-software
  - Python
```

### 3. Commit and tag

```bash
git add CITATION.cff CHANGELOG.md LICENSE .release-scholar.toml
git commit -m "Add scholarly release metadata"
git tag v0.1.0
```

The tag **must** be semver format: `vX.Y.Z`. The `version` field in `CITATION.cff` must match (without the `v` prefix).

### 4. Validate

```bash
release-scholar check --project-dir .
```

This runs:
- **Git checks** — clean working directory, semver tag on HEAD
- **File checks** — required files exist (LICENSE, README.md, CHANGELOG.md, CITATION.cff)
- **Citation validation** — CITATION.cff structure, ORCID format, version match
- **Security audit** — scans tracked files for secrets, flags sensitive files, checks git history
- **Gitignore audit** — checks for missing security patterns and build artifact patterns (auto-detects Java, Python, Rust, Node.js ecosystems)

Fix any `[FAIL]` items before proceeding. `[WARN]` items are advisory.

### 5. Build release bundle

```bash
release-scholar build --project-dir .
```

Creates `release/vX.Y.Z/` containing:

| File | Purpose |
|------|---------|
| `project-vX.Y.Z.tar.gz` | Deterministic archive of tracked files at tag |
| `checksums.txt` | SHA256 hash |
| `metadata.json` | Zenodo-ready deposit metadata |
| `CITATION.cff` | Citation metadata copy |

The archive is deterministic — same tag always produces the same checksum.

### 6. Publish to Zenodo

**Prerequisites:**
- Zenodo account at [zenodo.org](https://zenodo.org)
- Personal access token with `deposit:actions` and `deposit:write` scopes
- Token saved to `~/.config/release-scholar/token` (Linux) or `~/Library/Application Support/release-scholar/token` (macOS)

**Create a draft (review first):**
```bash
release-scholar publish --project-dir .
```

**Publish for real (mints a DOI):**
```bash
release-scholar publish --project-dir . --confirm
```

**Test on sandbox first:**
```bash
release-scholar publish --project-dir . --sandbox
```

After publishing with `--confirm`, the tool:
- Prints your DOI
- Automatically adds a DOI badge to your README.md
- Tells you the git command to commit the badge

### 7. Commit the badge and push

```bash
git add README.md
git commit -m "Add DOI badge for v0.1.0"
git push origin main
git push origin v0.1.0
```

## Configuration

`.release-scholar.toml` in your project root:

```toml
forge = "codeberg"                # codeberg, github, or gitlab
language = "eng"                  # ISO 639-3 language code
archive_dir = "release"           # where build output goes
required_files = ["LICENSE", "README.md", "CHANGELOG.md", "CITATION.cff"]

[author]
name = "Jane Smith"
orcid = "https://orcid.org/0000-0002-1234-5678"
email = "jane@example.com"
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Scaffold metadata files |
| `check` | Validate release readiness |
| `build` | Create archive + metadata bundle |
| `publish` | Upload to Zenodo (draft or final) |

All commands accept `--project-dir <path>` (defaults to `.`).

## Typical Workflow

```
init → edit metadata → commit → tag → check → fix issues → build → publish → commit badge → push
```

## Recommended .gitignore additions

Add these to your project's `.gitignore`:

```gitignore
# release-scholar
release/

# Security
.env
*.pem
*.key
id_rsa
```

The `check` command will warn you about missing patterns relevant to your detected language ecosystem.
