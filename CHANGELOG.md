# Changelog

## 0.1.1

- Respect `CARGO` environment variable, if set ([3512a63](https://github.com/trailofbits/build-wrap/commit/3512a636868e1e871ce4544f5bd425fbcf88b444))
- `cd` into the directory in which the wrapper package is being built. This avoids any `.cargo/config.toml` that may be in ancestors of the directory from which `build-wrap` was invoked. ([57775ac](https://github.com/trailofbits/build-wrap/commit/57775acff06ab59eccf78e17c819f960954fc9b0))

## 0.1.0

- Initial release
