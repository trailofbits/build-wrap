# Changelog

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
