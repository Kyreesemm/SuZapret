# Changelog

All notable changes to the **SuZapret** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added a cross-platform Rust backend for launching `winws.exe` on Windows and `nfqws` with Linux firewall integration.
- Added the SuZapret TOML strategy model, validation, strategy store, and native BAT-to-TOML conversion inside the application binary.
- Added bundled Windows and Linux zapret runtime files under `runtime/`.
- Added 25 converted strategies under `strategies/`.
- Added automated tests for strategy validation, strategy storage, BAT parsing, path normalization, and TOML serialization.

### Fixed
- Preserved separate Linux and Windows runtime layouts while keeping strategy files platform-neutral.

### Changed
- Replaced the temporary Perl strategy importer with the Rust converter module.

---
