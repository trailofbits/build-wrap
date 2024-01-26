# link-wrap

Link Rust build scripts so that they are executed under another command, such as Linux's `unshare`

## Environment variables

Note: These are read **when the build script is linked**. So, for example, changing `LINK_WRAP_CMD` will not change the command used to execute already linked build scripts.

- `LINK_WRAP_CMD`: Command used to execute a build script. Default: `unshare --map-root-user --net`
- `LINK_WRAP_LD`: Linker to use. Default: `cc`

## Recommended usage

1. Install `link-wrap`:
   ```sh
   cargo install link-wrap
   ```
2. In your home directory, create a `.cargo/config.toml` file with the following contents:
   ```toml
   [target.'cfg(all())']
   linker = "link-wrap"
   ```
