# Changelog

## 0.5.1

- Eliminate reliance on `once_cell` ([115](https://github.com/trailofbits/build-wrap/pull/115))

## 0.5.0

- FEATURE: Support `$HOME/.config/build-wrap/allow.txt`. A package whose name appears in this file will be built as though `BUILD_WRAP_ALLOW` were set to `1`. ([104](https://github.com/trailofbits/build-wrap/pull/104))

## 0.4.1

- Unset `CARGO_TARGET_DIR` when building the wrapper package ([aa4a646](https://github.com/trailofbits/build-wrap/commit/aa4a646d8eee4e209140f12fe47554f5c3e913a8))

## 0.4.0

- FEATURE: Rename the original build script and refer to it from the "wrapper" built script, rather than include the original build script as a byte array ([86](https://github.com/trailofbits/build-wrap/pull/86) and [89](https://github.com/trailofbits/build-wrap/pull/89))

## 0.3.2

- Update documentation ([41a6361](https://github.com/trailofbits/build-wrap/commit/41a6361466840db58c3853992ff0826d230040bc). [56aded5](https://github.com/trailofbits/build-wrap/commit/56aded59a8630bacfe8298bee759b459948fa374), and [f08ed71](https://github.com/trailofbits/build-wrap/commit/f08ed71f1f5c8857a4733196a2a0a692d7091ceb))
- Check for Bubblewrap AppArmor profile before declaring `build-wrap` enabled on Ubuntu 24.04 ([81](https://github.com/trailofbits/build-wrap/pull/81))

## 0.3.1

- Reduce error message verbosity ([58](https://github.com/trailofbits/build-wrap/pull/58))

## 0.3.0

- FEATURE: Show whether `build-wrap` is enabled in help message ([72a5991](https://github.com/trailofbits/build-wrap/commit/72a5991c7cdc55250f78692598cc9ff48e23d338))
- FEATURE: Add `BUILD_WRAP_ALLOW` environment variable. When set, if running a build script under `BUILD_WRAP_CMD` fails, the failure is reported and the build script is rerun normally. ([639b21b](https://github.com/trailofbits/build-wrap/commit/639b21b5fe1711967c969ba9ffd6afabe0ffa44d))

## 0.2.1

- If `TMPDIR` is set to a path in `/private`, then `PRIVATE_TMPDIR` is treated as though it is set to that path when `BUILD_WRAP_CMD` is expanded. This is needed for some build scripts that use [`cc-rs`](https://github.com/rust-lang/cc-rs). ([ff75d98](https://github.com/trailofbits/build-wrap/commit/ff75d98b2ea9ad63d8361e94c13ec0e6678d22e5))

## 0.2.0

- Change how the `BUILD_WRAP_CMD` environment variable is expanded ([500f5c1](https://github.com/trailofbits/build-wrap/commit/500f5c1f127697bfbe683e0278f6dd8be32e0bb5))
  - Split at whitespace before replacing environment variables, instead of after
  - Allow escaping whitespace with a backslash (`\`)
- Preliminary macOS support ([4b72e78](https://github.com/trailofbits/build-wrap/commit/4b72e784656e4eb31a3937ebc3d2ccc2a25123e9))

## 0.1.1

- Respect `CARGO` environment variable, if set ([3512a63](https://github.com/trailofbits/build-wrap/commit/3512a636868e1e871ce4544f5bd425fbcf88b444))
- `cd` into the directory in which the wrapper package is being built. This avoids any `.cargo/config.toml` that may be in ancestors of the directory from which `build-wrap` was invoked. ([57775ac](https://github.com/trailofbits/build-wrap/commit/57775acff06ab59eccf78e17c819f960954fc9b0))

## 0.1.0

- Initial release
