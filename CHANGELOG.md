# Changelog

All notable changes to the **SuZapret** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added the first responsive `webui/` control center with a compact 3:4 dark layout, animated SuZapret branding, strategy selection, GameFilter controls, status indicator, and event log.
- Added client-side tab navigation, strategy search, start/stop state preview, and platform detection for the web UI shell.
- Added a cross-platform Rust backend for launching `winws.exe` on Windows and `nfqws` with Linux firewall integration.
- Added the SuZapret TOML strategy model, validation, strategy store, and native BAT-to-TOML conversion inside the application binary.
- Added bundled Windows and Linux zapret runtime files under `runtime/`.
- Added 25 converted strategies under `strategies/`.
- Added automated tests for strategy validation, strategy storage, BAT parsing, path normalization, and TOML serialization.

### Fixed
- Preserved separate Linux and Windows runtime layouts while keeping strategy files platform-neutral.

### Changed
- Tuned the native-style layout with a static charcoal background, larger readable controls, a larger dock, a compact contour-tracing logo glow, and an action button outside the settings card.
- Refined the web UI into a borderless native-style page, removed the strategy tab and auxiliary labels, centered the compact status line, and simplified the cyan logo glow animation.
- Replaced the temporary Perl strategy importer with the Rust converter module.

---
