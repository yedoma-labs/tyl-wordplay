# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Scope

`tyl-wordplay` is a pure text library: no `unsafe` code (`#![forbid(unsafe_code)]`), no network, no dependencies. The relevant risks are resource use on hostile input and misleading results:

- Search work is bounded by `ReparseConfig::max_scan`, `max_results` and the lexicon's longest word. `nests_of`, `wheels` and `swaps` are quadratic in word length and linear in the lexicon size, so cap the length of untrusted input before scanning. `Playground::try_scan` enforces that cap (`PlaygroundConfig::max_input_chars`); `scan` does not.
- `WordSet::from_file` reads the whole file into memory.
- `hazard::scan` is a heuristic aid, not a moderation system. It only finds what is on your blocklist and in your lexicon.

## Reporting a Vulnerability

**Do not** open a public issue for security vulnerabilities. Use a private GitHub Security Advisory on this repository, or email yedoma-labs@proton.me. Include a description, steps to reproduce and the impact. We acknowledge within 48 hours.
