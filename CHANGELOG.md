# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.7.0] - 2026-08-13

### CI & Maintenance

- Bump actions/checkout from 5 to 7 ([a5365e2](https://github.com/kraytos17/rustdiff/commit/a5365e2f6b03fba78a75bb8981b9fc17f07527e3))

- Bump actions/upload-artifact from 4 to 7 ([b7973e8](https://github.com/kraytos17/rustdiff/commit/b7973e88b3dbf8c64b1421ff046528d69856c822))

- Bump softprops/action-gh-release from 2 to 3 ([775a622](https://github.com/kraytos17/rustdiff/commit/775a622d75977c864493e4266e603882ad230524))

- Bump actions/download-artifact from 4 to 8 ([c35c966](https://github.com/kraytos17/rustdiff/commit/c35c9660cf359b14e5daafd0b143832a576fd45a))


### Other

- Clean up HTML rendering, remove confusing render options ([93b0f5f](https://github.com/kraytos17/rustdiff/commit/93b0f5f936c98f7a5d5badc6ed3ea5ffe3e873d0))

- Major codebase refactor, drop memchr, add more tests ([21f8965](https://github.com/kraytos17/rustdiff/commit/21f89654564c04c836d57bf13891a1c2599b98b8))

- Imrpove HTML, CSS, JS for static page rendering ([334e002](https://github.com/kraytos17/rustdiff/commit/334e002f6766e215bb86cddc1807f91f15ce9b13))

## [0.6.0] - 2026-08-10

### Other

- Improve CI, add benchmarking, algo improvement WIP ([a278a04](https://github.com/kraytos17/rustdiff/commit/a278a0449f95bfd02b1881b4c346086aecd950e9))

- Main algo is now histogram, fallback to Myers ([c39d3e3](https://github.com/kraytos17/rustdiff/commit/c39d3e36af39bae8e137f0ffc2f8f2a61e5ed59a))

## [0.4.3] - 2025-11-10

### Other

- Dont color the diff output file, color should ideally be for terminal use only (as ANSI is used) ([1e642b1](https://github.com/kraytos17/rustdiff/commit/1e642b1920a98d9e7a1b70691593f348e38ab9dd))

## [0.4.2] - 2025-11-10

### Other

- CI fix ([207890b](https://github.com/kraytos17/rustdiff/commit/207890bf7f8235dafc1059f9e5301e7a42bf9ba0))

- CI fix ([606302f](https://github.com/kraytos17/rustdiff/commit/606302fda1d49bc9f383129a98f651cbb7cd5eb1))

- CI fix ([524974c](https://github.com/kraytos17/rustdiff/commit/524974c85a9a77d9d188ddfdde1c1024ffefa896))

## [0.4.1] - 2025-11-10

### Other

- Add dist profile in Cargo.toml and clean up CI ([bf1d616](https://github.com/kraytos17/rustdiff/commit/bf1d616cebc1b332778feb8fcce457d24e23cf31))

## [0.4] - 2025-11-10

### Other

- Update README.md ([bac37f8](https://github.com/kraytos17/rustdiff/commit/bac37f8d27c49d43a86850d66342f45c6c5ba85b))

- Add side-by-side diff rendering in html ([e1c30f6](https://github.com/kraytos17/rustdiff/commit/e1c30f64af3a85b3b719cd7b3080b287b3c01661))

- Clippy lint fix ([8b0fc76](https://github.com/kraytos17/rustdiff/commit/8b0fc76401654df74335bb3206d4009742878a5a))

- Clippy lint fix ([dcc7aaf](https://github.com/kraytos17/rustdiff/commit/dcc7aaf5ca117840eb0d0c1c40543bb3d3bf93bc))

- Update README.md ([41b9034](https://github.com/kraytos17/rustdiff/commit/41b9034a899003d7a54f1079fc05789e84347f66))

## [0.3] - 2025-11-09

### Other

- Add html flag to render the ANSI colors in html page, can be viewed in any browser, and colorize comapcted and unified diffs ([5af42bc](https://github.com/kraytos17/rustdiff/commit/5af42bc3165f562c09e37a0b44e7c68594fc34ad))

## [0.2.1] - 2025-11-08

### CI & Maintenance

- Ci improvement ([7f3f227](https://github.com/kraytos17/rustdiff/commit/7f3f2271fac60e1b4a43fe00a6c6eeb37a84ea94))

## [0.2] - 2025-11-08

### Bug Fixes

- Fix ci ([584905f](https://github.com/kraytos17/rustdiff/commit/584905fcc1bf5a704239cb2cfaececda460a6576))

- Fix spacing and readability of the --word diff generated ([430be84](https://github.com/kraytos17/rustdiff/commit/430be84af8f1586ff26bcc0dc411e5280de48ac6))

- Fix logical error in bactrack() and add unit tests for myer's impl ([498c72a](https://github.com/kraytos17/rustdiff/commit/498c72a6d41778e94273f1aadd55673f9529376c))

- Fix minor formatting issue, add another group for Cli struct ([eb97db9](https://github.com/kraytos17/rustdiff/commit/eb97db9de294adc297d8f48e12cf8afcdeaaa340))


### CI & Maintenance

- Ci fix ([66d8997](https://github.com/kraytos17/rustdiff/commit/66d899751ecf34fa343c7866579f7d26b0df370e))


### Other

- Correct logic issues in patience diffing and add unit tests for it ([77745b3](https://github.com/kraytos17/rustdiff/commit/77745b31f3eed36686b199c04830938e8e75c44b))

- Modify CI to better name shipped binary ([d7e2b2d](https://github.com/kraytos17/rustdiff/commit/d7e2b2d93981fd7f8eabfdbac0712ed4e562c4fa))

## [0.1] - 2025-11-08

### Other

- Initial commit ([949dbfb](https://github.com/kraytos17/rustdiff/commit/949dbfbf5c7dab01bb2cdd79fb50e28ea98bd2d4))

- Add draft impl, plain Myer's alog ([a0bb4a6](https://github.com/kraytos17/rustdiff/commit/a0bb4a60c7dfb35fcf88a845fd9cad0fd1aa12fd))

- Added patience diff, render for line, word, unified git-like diff ([efc650d](https://github.com/kraytos17/rustdiff/commit/efc650d4f819be0ac0a3dee270ad04d601ed0cf8))

- Update README.md ([7078d1d](https://github.com/kraytos17/rustdiff/commit/7078d1d2098b9535604e6fd48713a9d2c13d9df3))

- Add github actions CI ([91b16ee](https://github.com/kraytos17/rustdiff/commit/91b16eebfa3633e48aab384fa1393ab5f4dcacf7))

- Revert "update README.md" ([d183123](https://github.com/kraytos17/rustdiff/commit/d183123e2bb041247af07af5a4e937796e08e0bd))

- Update README and add CI ([721ec1d](https://github.com/kraytos17/rustdiff/commit/721ec1d53df204c70013bbc5db0a2ef0bec646e6))


### Refactoring

- Refactor exsiting code, add patience diffing (like git diff --patience) and various modes for diffing algos ([722b003](https://github.com/kraytos17/rustdiff/commit/722b003fd6c748c7adc9871f9d55bae8d6aecf65))

<!-- generated by git-cliff -->
