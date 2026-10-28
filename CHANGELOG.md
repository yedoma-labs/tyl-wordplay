# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/yedoma-labs/tyl-wordplay/compare/v0.1.0...v0.1.1) - 2026-10-28

### Added

- *(playground)* configurable detector limits and try_scan input cap
- *(lexicon)* load frequency lists with a minimum count

### Added

- `WordSet::from_frequency_lines` and `WordSet::from_frequency_file` load `word count` frequency lists and keep the words at a minimum count.
- `PlaygroundConfig` with `Playground::with_config` and `Playground::config`: sets the limits of the hazard, reparse and nest detectors.
- `Playground::try_scan` refuses text longer than `PlaygroundConfig::max_input_chars` (default 256) with `ScanError::TooLong`.
