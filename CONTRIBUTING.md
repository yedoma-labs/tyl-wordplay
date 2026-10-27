# Contributing to tyl-wordplay

1. Fork and clone the repository
2. Enable the commit hook: `git config core.hooksPath .githooks`
3. Create a branch, make your change, add tests (unit tests next to the code, end-to-end tests in `tests/`)
4. Run the checks CI runs:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-targets --all-features
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
   ```
5. Commit with a [Conventional Commit](https://www.conventionalcommits.org/) message and open a pull request

## Commit messages

`<type>(<scope>)?: <description>`, for example `feat(hazard): report inside matches that complete to a reading`. Types: `feat fix docs style refactor perf test build ci chore revert`. `fix:` bumps the patch version, `feat:` the minor version (while 0.x), `feat!:` or a `BREAKING CHANGE` footer the major version.

## Adding a form

A form is a module with

- a plain struct for the result (fields, not a string), with `impl From<&YourType> for Finding`
- a config struct with `Default` that bounds the work
- functions generic over `Lexicon + ?Sized`
- deterministic output (sort before returning; never depend on hash iteration order)
- tests that pin a German and an English example, and one that shows what must *not* match

Add the variant to `Form`, document the definition in the module docs and in the README table.
