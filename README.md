# build-wrap

A linker replacement to help protect against malicious build scripts

`build-wrap` "re-links" a build script so that it is executed under another command. By default, the command is [Bubblewrap] (Linux) or [`sandbox-exec`] (macOS), though this is configurable. See [Environment variables that `build-wrap` reads] and [How `build-wrap` works] for more information.

## Installation

Installing `build-wrap` requires two steps:

1. Install `build-wrap` with Cargo:
   ```sh
   cargo install build-wrap
   ```
2. Create a `.cargo/config.toml` file in your home directory with the following contents:
   ```toml
   [target.'cfg(all())']
   linker = "build-wrap"
   ```

## Environment variables that `build-wrap` reads

- `BUILD_WRAP_ALLOW`: When set to a value other than `0`, `build-wrap` uses the following weakened strategy. If a running a build script under `BUILD_WRAP_CMD` fails, report the failure and rerun the build script normally.

- `BUILD_WRAP_CMD`: Command used to execute a build script. Linux default:

  ```sh
  bwrap
    --ro-bind / /              # Allow read-only access everywhere
    --dev-bind /dev /dev       # Allow device access
    --bind {OUT_DIR} {OUT_DIR} # Allow write access to `OUT_DIR`
    --bind /tmp /tmp           # Allow write access to /tmp
    --unshare-net              # Deny network access
    {}                         # Build script path
  ```

  Note that `bwrap` is [Bubblewrap].

  macOS default:

  ```sh
  sandbox-exec -p
  (version\ 1)\
  (deny\ default)\
  (allow\ file-read*)\                                 # Allow read-only access everywhere
  (allow\ file-write*\ (subpath\ "/dev"))\             # Allow write access to /dev
  (allow\ file-write*\ (subpath\ "{OUT_DIR}"))\        # Allow write access to `OUT_DIR`
  (allow\ file-write*\ (subpath\ "{TMPDIR}"))\         # Allow write access to `TMPDIR`
  (allow\ file-write*\ (subpath\ "{PRIVATE_TMPDIR}"))\ # Allow write access to `PRIVATE_TMPDIR` (see below)
  (allow\ process-exec)\                               # Allow `exec`
  (allow\ process-fork)\                               # Allow `fork`
  (allow\ sysctl-read)\                                # Allow reading kernel state
  (deny\ network*)                                     # Deny network access
  {}                                                   # Build script path
  ```

  Note that `(version\ 1)\ ... (deny\ network*)` expands to a single string (see [How `BUILD_WRAP_CMD` is expanded] below).

- `BUILD_WRAP_LD`: Linker to use. Default: `cc`

Note that the above environment variables are read **when the build script is linked**. So, for example, changing `BUILD_WRAP_CMD` will not change the command used to execute already linked build scripts.

## Environment variables that `build-wrap` treats as set

Note that we say "treats as set" because these are considered only when [`BUILD_WRAP_CMD` is expanded].

- `PRIVATE_TMPDIR`: If `TMPDIR` is set to a path in `/private` (as is typical on macOS), then `PRIVATE_TMPDIR` expands to that path. This is needed for some build scripts that use [`cc-rs`], though the exact reason it is needed is still unknown.

## How `BUILD_WRAP_CMD` is expanded

- `{}` is replaced with the path of a copy of the original build script.
- `{VAR}` is replaced with the value of environment variable `VAR`.
- `{{` is replaced with `{`.
- `}}` is replaced with `}`.
- `\` followed by a whitespace character is replaced with that whitespace character.
- `\\` is replaced with `\`.

## How `build-wrap` works

When invoked, `build-wrap` does the following:

1. Link normally using `BUILD_WRAP_LD`.
2. Parse the arguments to determine whether the output file is a build script.
3. If so, replace the build script `B` with its "wrapped" version `B'`, described next.

Given a build script `B`, its "wrapped" version `B'` contains a copy of `B` and does the following when invoked:

1. Create a temporary file with the contents of `B`. (Recall: `B'` contains a copy of `B`).
2. Make the temporary file executable.
3. Expand `BUILD_WRAP_CMD` in the [manner described above].
4. Execute the expanded command.

## Goals

- Aside from configuration and dealing with an occasional warning, `build-wrap` should not require a user to adjust their normal workflow.

[Bubblewrap]: https://github.com/containers/bubblewrap
[Environment variables that `build-wrap` reads]: #environment-variables-that-build-wrap-reads
[How `BUILD_WRAP_CMD` is expanded]: #how-build_wrap_cmd-is-expanded
[How `build-wrap` works]: #how-build-wrap-works
[`BUILD_WRAP_CMD` is expanded]: #how-build_wrap_cmd-is-expanded
[`cc-rs`]: https://github.com/rust-lang/cc-rs
[`sandbox-exec`]: https://keith.github.io/xcode-man-pages/sandbox-exec.1.html
[manner described above]: #how-build_wrap_cmd-is-expanded
