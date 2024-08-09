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

### Security

### Breaking Changes


## Program [0.1.0] [PR #16](https://github.com/jup-ag/distributor/pull/16)

### Added
- Disitrbutor supports 2 modes now defined by `activation_type`. If `activation_type == 0`, activation and bonus are calculated based on slot. If `activation_type == 1`, activation and bonus are calculated based on timestamp. 

### Changed
- merkle tree state added a new field `activation_type`
- Rename `enable_slot` to `activation_point` and `airdrop_bonus.vesting_slot_duration` to `airdrop_bonus.vesting_duration`

### Breaking Changes

- Program endpoint `new_distributor` and `new_distributor2`. User need to send `activation_point` and `activation_type`
- Program endpoint `set_enable_slot` is renamed to `set_activation_point`
