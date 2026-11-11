# Code signing (free, via SignPath Foundation)

This project's releases are built and signed by GitHub Actions
([`.github/workflows/release.yml`](.github/workflows/release.yml)). Signing uses
[**SignPath Foundation**](https://signpath.org/), which provides **free code-signing certificates
for open-source projects**. Until it's configured, releases are built **unsigned** (Windows
SmartScreen will warn); once configured, tagged releases are signed automatically.

## One-time setup

### 1. Apply to SignPath Foundation
Apply at **https://signpath.org/apply**. They review each project for eligibility (must be open
source with an OSI license — this repo is MIT ✓). Approval is a manual step and can take a while;
a brand-new repo may be asked to show some traction first.

### 2. Configure the SignPath project (after approval)
In the SignPath web dashboard:
1. **Organization** — note your **Organization ID** (a GUID).
2. **Project** — create one with slug **`checklists`**, linked to `github.com/synju/checklists`.
3. **Trusted Build System** — add **GitHub Actions** and connect this repository, so SignPath only
   signs artifacts that came from this repo's CI.
4. **Artifact configuration** — add one for a single Authenticode `.exe` (the default is fine).
5. **Signing policy** — create one with slug **`release-signing`**.
6. **API token** — create a user API token (used by CI).

> The workflow hard-codes `project-slug: checklists` and `signing-policy-slug: release-signing`.
> If you name yours differently, edit those two lines in `release.yml`.

### 3. Add the CI credentials to GitHub
In the repo: **Settings → Secrets and variables → Actions**
- **Variable** `SIGNPATH_ORGANIZATION_ID` = your Organization ID (GUID).
- **Secret** `SIGNPATH_API_TOKEN` = the SignPath API token.

The workflow only runs the signing step when `SIGNPATH_ORGANIZATION_ID` is set, so nothing breaks
before you've finished setup.

## Cutting a release
1. Bump the version in [`app/package.json`](app/package.json) (e.g. `0.1.1`).
2. Tag and push:
   ```
   git tag v0.1.1
   git push origin v0.1.1
   ```
3. GitHub Actions builds the Rust core + the installer, submits it to SignPath, waits for the
   signed artifact, and publishes it to a **GitHub Release** for that tag.

(You can also run the workflow manually from the **Actions** tab to build without releasing — the
unsigned installer is saved as a run artifact.)

## Notes
- SignPath Foundation signs **only** artifacts produced by the trusted CI build — you can't sign a
  local build with it. That's by design (reproducible, auditable releases).
- An OV/SignPath signature builds SmartScreen reputation over time; it isn't instant like an EV
  certificate, but it's free and removes the "unknown publisher" line.
