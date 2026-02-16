# release-scholar

A CLI tool that validates, audits, and packages scholarly software releases for Zenodo/DOI publication.

`release-scholar` automates the tedious parts of publishing research software: metadata validation, security auditing, size analysis, deterministic archiving, Zenodo deposit creation, and forge mirroring — so you get a DOI with minimal friction.

## Why?

If you write research software, you should be citing it. That means:
- A `CITATION.cff` so others know how to cite your work
- A Zenodo deposit so your release has a persistent DOI
- An archive that's deterministic and verifiable
- A security audit so you don't accidentally publish credentials

`release-scholar` handles all of this in a single workflow.

## Install

**From source (requires Rust toolchain):**
```bash
git clone https://codeberg.org/research_coder/release-scholar.git
cd release-scholar
cargo install --path .
```

## First-Time Setup

### 1. Global configuration

Create a global config with your author info (used across all projects):

**macOS:**
```bash
mkdir -p ~/Library/Application\ Support/release-scholar
```

**Linux:**
```bash
mkdir -p ~/.config/release-scholar
```

Then create `config.toml` in that directory:

```toml
[author]
name = "Jane Smith"
orcid = "https://orcid.org/0000-0002-1234-5678"
email = "jane@example.com"

[mirrors]
codeberg_user = "janesmith"
codeberg_token = "your-codeberg-pat"
github_user = "janesmith"
github_token = "your-github-pat"
gitlab_user = "janesmith"
gitlab_token = "your-gitlab-pat"
```

Lock down the file (it contains tokens):
```bash
chmod 600 ~/Library/Application\ Support/release-scholar/config.toml   # macOS
chmod 600 ~/.config/release-scholar/config.toml                         # Linux
```

### 2. Zenodo tokens

Create personal access tokens at [zenodo.org](https://zenodo.org) (and optionally [sandbox.zenodo.org](https://sandbox.zenodo.org)) with `deposit:actions` and `deposit:write` scopes.

Save them alongside your config:

```bash
echo "YOUR_TOKEN" > ~/Library/Application\ Support/release-scholar/token          # production
echo "YOUR_TOKEN" > ~/Library/Application\ Support/release-scholar/sandbox-token  # sandbox
chmod 600 ~/Library/Application\ Support/release-scholar/*token
```

### 3. Choose your primary forge

Each project's `.release-scholar.toml` specifies the primary forge:

```toml
forge = "codeberg"   # or "github" or "gitlab"
```

This affects `init` templates (repository URL patterns) and will determine the mirror direction in the `mirror` command.

## Workflow

```
init → edit metadata → commit → tag → check → fix issues → build → publish → mirror → push
```

### 1. Initialize metadata

```bash
cd /path/to/your/project
release-scholar init --project-dir .
```

Creates (if missing):
- **CITATION.cff** — citation metadata, pre-filled from your global config
- **CHANGELOG.md** — [Keep a Changelog](https://keepachangelog.com/) template
- **LICENSE** — Apache-2.0 (default)
- **.release-scholar.toml** — per-project configuration

Your name, ORCID, and email are automatically filled from the global config.

### 2. Edit your metadata

Open `CITATION.cff` and fill in:
- Project title and description
- License ([SPDX identifier](https://spdx.org/licenses/))
- Repository URL
- Keywords (include programming languages, e.g. `Python`, `Java`)
- Version (must match your git tag, without the `v` prefix)

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
repository-code: "https://codeberg.org/janesmith/my-project"
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

The tag **must** be semver format: `vX.Y.Z`.

### 4. Validate

```bash
release-scholar check --project-dir .
```

This runs five audit categories:

- **Git** — clean working directory, semver tag on HEAD
- **Files** — required files exist (LICENSE, README.md, CHANGELOG.md, CITATION.cff)
- **Citation** — CITATION.cff structure, ORCID format, version matches tag
- **Security** — scans tracked files for secrets (private keys, API tokens, passwords), flags sensitive files, audits git history, checks .gitignore coverage
- **Gitignore** — detects project ecosystem (Java, Python, Rust, Node.js) and warns about missing build artifact patterns
- **Size** — total repo size, large files (>1 MB warning, >10 MB failure), binary/vendor file detection

Fix `[FAIL]` items before proceeding. `[WARN]` items are advisory.

### 5. Build release bundle

```bash
release-scholar build --project-dir .
```

Creates `release/vX.Y.Z/` containing:

| File | Purpose |
|------|---------|
| `project-vX.Y.Z.tar.gz` | Deterministic archive of git-tracked files at tag |
| `checksums.txt` | SHA256 hash |
| `metadata.json` | Zenodo-ready deposit metadata |
| `CITATION.cff` | Citation metadata copy |

The archive is **deterministic** — the same tag always produces the same checksum, regardless of when or where you build it.

### 6. Publish to Zenodo

**Test on sandbox first (recommended for first use):**
```bash
release-scholar publish --project-dir . --sandbox
```

**Create a draft on production (review before minting DOI):**
```bash
release-scholar publish --project-dir .
```

**Publish for real (mints a permanent DOI):**
```bash
release-scholar publish --project-dir . --confirm
```

Production publishes have safety prompts:
- Drafts ask for `y/N` confirmation
- Final publish requires typing `publish` to confirm

After publishing with `--confirm`, the tool automatically adds a DOI badge to your README.md.

### 7. Set up forge mirrors

```bash
release-scholar mirror --project-dir .
```

Sets up push mirrors from Codeberg to GitHub and GitLab via the Codeberg API. Requires:
- The `[mirrors]` section in your global config
- Target repos must already exist on GitHub/GitLab
- Mirrors sync every 8 hours and on push

### 8. Push

```bash
git add README.md
git commit -m "Add DOI badge for v0.1.0"
git push origin main
git push origin v0.1.0
```

Mirrors will automatically propagate to GitHub and GitLab.

## Commands

| Command | Description |
|---------|-------------|
| `init` | Scaffold metadata files (CITATION.cff, CHANGELOG.md, LICENSE, config) |
| `check` | Validate release readiness (git, files, citation, security, size) |
| `build` | Create deterministic archive + metadata bundle |
| `publish` | Upload to Zenodo — draft or final, sandbox or production |
| `mirror` | Set up Codeberg → GitHub/GitLab push mirrors |

All commands accept `--project-dir <path>` (defaults to `.`).

### `publish` flags

| Flag | Effect |
|------|--------|
| *(none)* | Production draft — creates deposit, reserves DOI, does not publish |
| `--confirm` | Production publish — mints a permanent DOI |
| `--sandbox` | Sandbox draft — for testing, no real DOI |
| `--sandbox --confirm` | Sandbox publish — for testing the full flow |

## Configuration

### Per-project: `.release-scholar.toml`

Lives in your project root. Committed to git.

```toml
forge = "codeberg"                # codeberg, github, or gitlab
language = "eng"                  # ISO 639-3 language code
archive_dir = "release"           # where build output goes
required_files = ["LICENSE", "README.md", "CHANGELOG.md", "CITATION.cff"]
```

### Global config

Lives in your OS config directory. **Never committed to git.**

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/release-scholar/config.toml` |
| Linux | `~/.config/release-scholar/config.toml` |

Contains author defaults and forge credentials:

```toml
[author]
name = "Jane Smith"
orcid = "https://orcid.org/0000-0002-1234-5678"
email = "jane@example.com"

[mirrors]
codeberg_user = "janesmith"
codeberg_token = "your-codeberg-pat"
github_user = "janesmith"
github_token = "your-github-pat"
gitlab_user = "janesmith"
gitlab_token = "your-gitlab-pat"
```

Per-project config overrides global config. Author fields merge (project fields take priority, global fills gaps).

### Zenodo tokens

| File | Purpose |
|------|---------|
| `token` | Production Zenodo API token |
| `sandbox-token` | Sandbox Zenodo API token |

Stored alongside the global config. Can also be set via environment variables: `ZENODO_TOKEN` / `ZENODO_SANDBOX_TOKEN`.

## What the `check` command audits

| Category | Checks |
|----------|--------|
| **Git** | Clean working directory, HEAD has semver tag |
| **Files** | LICENSE, README.md, CHANGELOG.md, CITATION.cff exist |
| **Citation** | Valid YAML, required fields, ORCID format, version matches tag |
| **Security** | Private keys, API tokens (FAIL); password patterns (WARN); sensitive files; git history scan; .gitignore coverage |
| **Gitignore** | Missing security patterns; ecosystem-specific build artifacts (auto-detects Java, Python, Rust, Node.js) |
| **Size** | Total size (>50 MB warn, >200 MB fail); large files (>1 MB warn, >10 MB fail); binary/vendor files |

## Recommended .gitignore additions

```gitignore
# release-scholar
release/

# Security
.env
*.pem
*.key
id_rsa
```

The `check` command will warn you about additional patterns relevant to your detected language ecosystem.

## Zenodo metadata

The `build` and `publish` commands generate Zenodo metadata from your `CITATION.cff`:

| CITATION.cff field | Zenodo field |
|-------------------|--------------|
| `title` | `metadata.title` |
| `abstract` | `metadata.description` |
| `authors` | `metadata.creators` (with ORCID) |
| `keywords` | `metadata.keywords` |
| `license` | `metadata.license` |
| `version` | `metadata.version` |
| `date-released` | `metadata.publication_date` |
| `repository-code` | `metadata.related_identifiers` |
| config `language` | `metadata.language` |

## ORCID integration

After publishing to Zenodo:
1. Link your ORCID to Zenodo (Zenodo → Settings → Linked Accounts)
2. On ORCID, use **Works → Add → Search & Link → DataCite** to import your Zenodo publications
3. Authorise DataCite for auto-updates so future publications appear automatically

## License

Apache-2.0
