# Coalesce

Coalesce turns a Rust project's source files into a single AI-readable Markdown document.

## Run

```sh
cargo run --release
```

Select a project directory in the operating system's folder picker. Coalesce creates `coalesced/<project-name>/<project-name>.md` next to the running executable (when using `cargo run --release`, that is normally `target/release/coalesced/`).

The export includes a directory tree and the text contents of `.rs`, `.toml`, `.lock`, `.wgsl`, `.md`, and `.json` files. It skips common generated directories, symlinks, and selected secret-file names. Review exports before sharing: source files may still contain credentials or private information.

Edit `INCLUDED_EXTENSIONS`, `EXCLUDED_DIRS`, and `EXCLUDED_FILES` in `src/main.rs` to customize scanning.
