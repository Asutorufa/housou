---
description: Check and fix code formatting and linting issues for both Rust and frontend code
---

# Lint Check & Fix Workflow

This workflow mirrors the GitHub Actions CI pipeline defined in `.github/workflows/lint-check.yml` and `.github/workflows/lint-fix.yml`.

## Check Steps (what CI runs on PRs)

The following checks are performed on every pull request targeting `main`:

### 1. Check Rust Formatting
// turbo
```bash
cargo fmt --all -- --check
```
This checks that all Rust code is formatted according to `rustfmt` conventions. It does **not** modify files — it only reports diffs.

### 2. Check Rust Lints
// turbo
```bash
cargo clippy -- -D warnings
```
This runs `clippy` with strict mode (`-D warnings`), treating all warnings as errors.

### 3. Check Frontend Formatting
// turbo
```bash
cd web && npm run check-format
```
This runs `prettier --check .` in the `web/` directory.

### 4. Check Frontend Lints
// turbo
```bash
cd web && npm run lint
```
This runs the ESLint checks defined in the frontend project.

---

## Fix Steps (to auto-fix issues locally)

Run these commands to automatically fix formatting and lint issues before committing:

### 1. Fix Rust Formatting
```bash
cargo fmt --all
```

### 2. Fix Frontend Formatting
```bash
cd web && npm run format
```
This runs `prettier --write .` in the `web/` directory.

### 3. Fix Frontend Lints
```bash
cd web && npm run lint:fix
```

---

## Notes

- The GitHub Actions **Lint Check** workflow (`.github/workflows/lint-check.yml`) runs on PRs to `main` and will **fail the build** if any check does not pass.
- The GitHub Actions **Lint Fix** workflow (`.github/workflows/lint-fix.yml`) runs on pushes to `main` and on manual dispatch. It auto-fixes issues and opens a PR with the changes.
- Always run the **Check** steps locally before pushing to avoid CI failures.
- Rust toolchain components `rustfmt` and `clippy` must be installed (`rustup component add rustfmt clippy`).
- The Rust build target `wasm32-unknown-unknown` is required for clippy to work on this project (`rustup target add wasm32-unknown-unknown`).
