# Development and release

The desktop is a Tauri 2 WebKitGTK window with a React UI in `ui/`.
The same `ws` binary embeds the built UI and retains every JSON CLI command.
The existing ksni tray and D-Bus single-instance service stay in Rust.

Source builds need Rust, Node.js 22.12 or newer, pnpm and the system development packages:

```sh
bash scripts/system-deps.sh --build  # prints the exact distro install command
pnpm -C ui install --frozen-lockfile
./ops ui:check
./ops build
./ops check
./ops ui:test
./ops gui:smoke --output /tmp/window-side-tauri-check
```

`./ops ui:dev` runs Vite on 5199. To open the development webview explicitly,
run `./ops ui:dev --window` inside OmaBox with the Vite server reachable at
5199. Ordinary `./ops dev` builds the embedded UI and opens the app; agents
must run it inside OmaBox. `./ops ui:bindings` generates typed IPC from a Rust
test, and `./ops ui:bindings:check` rejects stale output. No code generation
runs during app startup. Lint uses strict type-aware oxlint, with no formatter.

`./ops icons` regenerates every application and tray size from
`icons/source/window-side.svg` with the dev-only `rsvg-convert` (librsvg) and
`magick` (ImageMagick) tools.
The PNGs retain an opaque alpha channel for Tauri. `./ops icons:check`, also run by
`./ops check`, regenerates into a temporary directory and compares the exact bytes.

Runtime packages are WebKitGTK 4.1 and GTK 3 (`webkit2gtk-4.1` on Arch,
`libwebkit2gtk-4.1-0` on Debian/Ubuntu, `webkit2gtk4.1` on Fedora), plus OpenSSL.
`install.sh` bootstraps these if the release binary cannot load its libraries.
The release archive includes `scripts/system-deps.sh`; source builds install
frontend dependencies and build the UI before Cargo embeds it.

CLI fixtures fake libvirt and the guest agent. Browser interaction tests use
synthetic mocks. Native fixture modes are `setup`, `status`, `stopped`,
`first-boot-start`, `first-boot-ready` and `unavailable` through
`WS_GUI_FIXTURE`; they keep all command effects in memory, without VM, setup
or keyring access.
`gui:smoke` creates an invisible OmaBox, checks the real WebKitGTK canvas and
bundled Lottie renderer and clipboard fallback, and saves a screenshot and
JSON evidence.
`guest:check` requires an already running QEMU guest and reads script syntax,
VM guards and driver readiness without applying provisioning.

CI runs only on manual dispatch and `v*` tags to save GitHub Actions minutes;
branch pushes and pull requests do not start it. Run `./ops check` locally before
pushing. After pushing to `main`, `./ops ci:run` dispatches and watches debug
checks; `./ops ci:run --release` checks the optimized build and warms its separate
cache. These commands require authenticated `gh` and `jq`. Weekly pacman packages,
the lockfile-keyed pnpm store and Rust dependencies/builds are cached. Tags restore
default-branch caches without saving tag-only entries. Warm the release cache
manually before tagging when dependencies change.

Tag runs build on Arch, verify the tag against the binary version and
create a **draft prerelease** with the archive and checksum. Release promotion
and AUR publication remain manual. Packaging refuses dirty source unless
`--allow-dirty` explicitly marks a local preview. See `plan/github-release/summary.md` and `plan/v1/` for decisions,
verification evidence and remaining runtime work.

The public bootstrap is `get.sh`; advance its selected version only when the new release is ready. `tests/bootstrap.sh` covers verification, failure propagation and forwarded installer options. The archive includes the README, linked guides and visual assets.
