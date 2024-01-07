# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added

### Changed

### Deprecated

### Removed

### Fixed

## [v0.4.0]
### Added
* Streaming API for scanning for hubs

### Changed
* Major refactor of API

## [v0.3.0] - 2022-12-10
### Changed
* Update btleplug to 0.10
* Full rewrite to simplified async architecture with only minor changes to
public API (i.e. most things still work just by adding an `.await` to the end)

### Security
* Update dependencies to clear RUSTSEC-2021-0119

## [v0.2.0] - 2021-12-06
### Added
* Example for tank steering remote control

### Changed
* Changed from anyhow to thiserror for compatibility

## [v0.1.0] - 2021-12-05
Initial release.
* Hub discovery
* Connect to discoverd hubs or by address
* Basic control of peripherals
