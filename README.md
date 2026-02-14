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

### Ubuntu 24.04

Ubuntu's default AppArmor profiles [changed with version 24.04]. The changes [affect Bubblewrap], which in turn affect `build-wrap`. Thus, installing `build-wrap` on Ubuntu 24.04 requires some additional steps:

```sh
sudo apt install apparmor-profiles
sudo cp /usr/share/apparmor/extra-profiles/bwrap-userns-restrict /etc/apparmor.d
sudo systemctl reload apparmor
```

Note that following these additional steps, Bubblewrap still runs unprivileged. More information on AppArmor profiles can be found on [Ubuntu Server] and the [Ubuntu Community Wiki].

## Environment variables that `build-wrap` reads

Note that the below environment variables are read **when a build script is linked**. So, for example, changing `BUILD_WRAP_CMD` will not change the command used to execute already linked build scripts.

- `BUILD_WRAP_ALLOW`: When set to a value other than `0`, `build-wrap` uses the following weakened strategy. If running a build script under `BUILD_WRAP_CMD` fails, report the failure and rerun the build script normally.

  Note that to see the reported failures, you must invoke Cargo with the `-vv` (["very verbose"]) flag, e.g.:

  ```sh
  BUILD_WRAP_ALLOW=1 cargo build -vv
  ```

  To disable sandboxing entirely for specific directories or packages, use [`$HOME/.config/build-wrap/config.toml`] (see below).

- `BUILD_WRAP_CMD`: Command used to execute a build script. Linux default:
  - With comments:

    ```sh
    bwrap
      --ro-bind / /              # Allow read-only access everywhere
      --dev-bind /dev /dev       # Allow device access
      --bind {OUT_DIR} {OUT_DIR} # Allow write access to `OUT_DIR`
      --bind /tmp /tmp           # Allow write access to /tmp
      --unshare-net              # Deny network access
      {}                         # Build script path
    ```

  - On one line (for copying-and-pasting):

    ```sh
    bwrap --ro-bind / / --dev-bind /dev /dev --bind {OUT_DIR} {OUT_DIR} --bind /tmp /tmp --unshare-net {}
    ```

  Note that `bwrap` is [Bubblewrap].

  macOS default:

  ```sh
  sandbox-exec -f {BUILD_WRAP_PROFILE_PATH} {}
  ```

  See [Environment variables that `build-wrap` treats as set] regarding `BUILD_WRAP_PROFILE_PATH`.

- `BUILD_WRAP_LD`: Linker to use. Default: `cc`

- `BUILD_WRAP_PROFILE`: macOS only. `build-wrap` expands `BUILD_WRAP_PROFILE` [as it would `BUILD_WRAP_CMD`], and writes the results to a temporary file. `BUILD_WRAP_PROFILE_PATH` then expands to the absolute path of that temporary file. Default:

  ```
  (version 1)
  (deny default)
  (allow file-read*)                               ;; Allow read-only access everywhere
  (allow file-write* (subpath "/dev"))             ;; Allow write access to /dev
  (allow file-write* (subpath "{OUT_DIR}"))        ;; Allow write access to `OUT_DIR`
  (allow file-write* (subpath "{TMPDIR}"))         ;; Allow write access to `TMPDIR`
  (allow file-write* (subpath "{PRIVATE_TMPDIR}")) ;; Allow write access to `PRIVATE_TMPDIR` (see below)
  (allow process-exec)                             ;; Allow `exec`
  (allow process-fork)                             ;; Allow `fork`
  (allow sysctl-read)                              ;; Allow reading kernel state
  (deny network*)                                  ;; Deny network access
  ```

## `$HOME/.config/build-wrap/config.toml`

If a file at `$HOME/.config/build-wrap/config.toml` exists, `build-wrap` reads it to determine which directories and packages should be allowed to build without sandboxing.

The file supports `[allow]` and `[ignore]` sections, which are treated as synonyms:

```toml
[allow]
directories = ["/home/user/project-a"]
packages = ["aws-lc-fips-sys"]

[ignore]
directories = ["/home/user/project-b"]
packages = ["svm-rs-builds"]
```

- `directories`: A list of directory paths. If `cargo build` is run from within a listed directory (or any subdirectory), `build-wrap` will not sandbox the build scripts.
- `packages`: A list of package names. Build scripts belonging to listed packages will not be sandboxed.

Both sections are merged, so entries from `[allow]` and `[ignore]` are combined.

For example, if you frequently build in a project that has dependencies requiring unrestricted build scripts:

```sh
mkdir -p "$HOME/.config/build-wrap"
cat > "$HOME/.config/build-wrap/config.toml" << 'EOF'
[allow]
packages = ["svm-rs-builds"]
EOF
```

## Environment variables that `build-wrap` treats as set

Note that we say "treats as set" because these are considered only when [`BUILD_WRAP_CMD` is expanded].

- `BUILD_WRAP_PROFILE_PATH`: Expands to the absolute path of a temporary file containing the expanded contents of `BUILD_WRAP_PROFILE`.

- `PRIVATE_TMPDIR`: If `TMPDIR` is set to a path in `/private` (as is typical on macOS), then `PRIVATE_TMPDIR` expands to that path. This is needed for some build scripts that use [`cc-rs`], though the exact reason it is needed is still unknown.

## How `BUILD_WRAP_CMD` is expanded

- `{}` is replaced with the path of a renamed copy of the original build script.
- `{VAR}` is replaced with the value of environment variable `VAR`.
- `{{` is replaced with `{`.
- `}}` is replaced with `}`.
- `\` followed by a whitespace character is replaced with that whitespace character.
- `\\` is replaced with `\`.

## How `build-wrap` works

When invoked, `build-wrap` does the following:

1. Link normally using `BUILD_WRAP_LD`.
2. Parse the arguments to determine whether the output file is a build script.
3. If not, stop; otherwise, proceed.
4. Let `B` be the build script's original name.
5. Rename the build script to a fresh, unused name `B'`.
6. At `B`, create a "wrapped" version of the build script whose behavior is described next.

The "wrapped" version of the build script does the following when invoked:

1. Expand `BUILD_WRAP_CMD` in the [manner described above], with `{}` expanding to `B'`.
2. Execute the expanded command.

## Goals

- Aside from configuration and dealing with an occasional warning, `build-wrap` should not require a user to adjust their normal workflow.

["very verbose"]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script
[Bubblewrap]: https://github.com/containers/bubblewrap
[Environment variables that `build-wrap` reads]: #environment-variables-that-build-wrap-reads
[Environment variables that `build-wrap` treats as set]: #environment-variables-that-build-wrap-treats-as-set
[How `build-wrap` works]: #how-build-wrap-works
[Ubuntu Community Wiki]: https://help.ubuntu.com/community/AppArmor
[Ubuntu Server]: https://documentation.ubuntu.com/server/how-to/security/apparmor/
[`$HOME/.config/build-wrap/config.toml`]: #homeconfigbuild-wrapconfigtoml
[`BUILD_WRAP_CMD` is expanded]: #how-build_wrap_cmd-is-expanded
[`cc-rs`]: https://github.com/rust-lang/cc-rs
[`sandbox-exec`]: https://keith.github.io/xcode-man-pages/sandbox-exec.1.html
[affect Bubblewrap]: https://github.com/containers/bubblewrap/issues/505#issuecomment-2093203129
[as it would `BUILD_WRAP_CMD`]: #how-build_wrap_cmd-is-expanded
[changed with version 24.04]: https://ubuntu.com/blog/ubuntu-23-10-restricted-unprivileged-user-namespaces
[manner described above]: #how-build_wrap_cmd-is-expanded
