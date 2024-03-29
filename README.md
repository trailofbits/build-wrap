# build-wrap

A linker replacement to help protect against malicious build scripts

`build-wrap` "re-links" a build script so that it is executed under another command. By default the command is [Bubblewrap], though this is configurable. See [Environment variables] and [How it works] for more information.

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

## Environment variables

- `BUILD_WRAP_CMD`: Command used to execute a build script. Default:

  ```sh
  bwrap
    --ro-bind / /              # Allow read-only access everywhere
    --dev-bind /dev /dev       # Allow device access
    --bind {OUT_DIR} {OUT_DIR} # Allow write access to `OUT_DIR`
    --unshare-net              # Deny network access
    {}                         # Build script path
  ```

  Note that `bwrap` is [Bubblewrap].

- `BUILD_WRAP_LD`: Linker to use. Default: `cc`

Note that the above environment variables are read **when the build script is linked**. So, for example, changing `BUILD_WRAP_CMD` will not change the command used to execute already linked build scripts.

## How `BUILD_WRAP_CMD` is expanded

- `{}` is replaced with the path of a copy of the original build script.
- `{VAR}` is replaced with the value of environment variable `VAR`.
- `{{` is replaced with `{`.
- `}}` is replaced with `}`.

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

[Bubblewrap]: https://github.com/containers/bubblewrap
[Environment variables]: #environment-variables
[How it works]: #how-it-works
[manner described above]: #how-build_wrap_cmd-is-expanded
