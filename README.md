# KrKr2 Web

A WebAssembly port of the **KiriKiri2 / Kirikiroid2 engine**, bringing KiriKiri games to the browser.

> This is a personal fork focused on the Web platform. Pull requests are not accepted.

**语言 / Language**: [中文](README_CN.md) | English

## Project status

The project is actively reconstructing the engine and plugins against four Kirikiroid2 1.3.9 reference binaries: Android and iOS, each on arm64 and armv7. This includes source structure, data flow, call chains, and object lifetimes. Implementation is ongoing; compatibility with every KiriKiri2 game or plugin is not guaranteed.

- The Web build uses **JSPI, native WebAssembly exceptions, and pthreads**, with workers created on demand and reused.
- **VirtualLazyFS** handles local files, directories, ZIPs, and remote resources. UI resources load through `assets.zip`; there is no longer an `index.data` bundle.
- Games can start from a local directory, a single XP3, a game ZIP, or a WebDAV directory. Save spaces, ZIP save import/export, and write-back to authorized directories are available.
- MotionPlayer / EmotePlayer, PSB, AlphaMovie, and LZFS have implementations and tests, with reference-behavior checks still ongoing.
- Release builds include a PWA. Development checks include Web script tests, native unit tests, and Wasmtime / Android differential tests.

## Browser and runtime requirements

The current Web build primarily targets **Chromium 137+ (such as desktop Chrome / Edge)**. It requires:

- WebAssembly JSPI: `WebAssembly.Suspending` and `WebAssembly.promising`.
- WebAssembly SIMD, native exceptions, shared memory, and WebGL.
- `SharedArrayBuffer` and cross-origin isolation, served on `localhost` or trusted HTTPS.
- OPFS (Origin Private File System) and `DecompressionStream('deflate-raw')` for ZIP extraction and caching.

WebAssembly and SharedArrayBuffer support alone is insufficient. Other browsers must provide these capabilities; the page checks for JSPI at startup. Firefox, Safari, or a mobile browser name alone does not establish compatibility. Direct local-directory access additionally requires the File System Access API; file and folder selection provide alternative entry points.

## Build

### Dependencies and environment

- [Emscripten SDK](https://emscripten.org/docs/getting_started/downloads.html) **6.0.9**: pinned in both CMake and CI; configuration rejects other versions.
- [vcpkg](https://learn.microsoft.com/en-us/vcpkg/get_started/get-started), Ninja, CMake **3.31.1+**, Bison **3.8.2+**, and Python 3.
- Linux builds also need `autoconf`, `automake`, and `libtool`. See [build-web.yml](.github/workflows/build-web.yml) for the CI configuration.

Run commands from the repository root. The environment setup below uses Bash / Zsh:

```bash
git submodule update --init --recursive

export VCPKG_ROOT=/path/to/vcpkg
/path/to/emsdk/emsdk install 6.0.9
/path/to/emsdk/emsdk activate 6.0.9
source /path/to/emsdk/emsdk_env.sh
```

The submodule provides `layerExImage`; CMake skips that plugin if it is absent. On Windows, use `emsdk.bat` / `emsdk_env.bat` and ensure `EMSDK_PYTHON` points to a working Python 3 interpreter. If macOS selects its older system Bison, pass `-DBISON_EXECUTABLE=/actual/path/to/bison` during configuration.

Prebuild the Emscripten ports as CI does, then configure and build:

```bash
embuilder build libpng libpng-mt libpng-legacysjlj libpng-mt-legacysjlj
embuilder build freetype freetype-legacysjlj
embuilder build harfbuzz harfbuzz-mt
embuilder build sdl2 sdl2-mt
embuilder build sdl2_ttf sdl2_ttf-mt

cmake --preset "Web Release Config"
cmake --build out/web/release
```

For Debug:

```bash
cmake --preset "Web Debug Config"
cmake --build out/web/debug
```

The regular Debug preset disables ASan. Use `Web Debug Asan Config` for an instrumented build in `out/web/debug-asan`, which needs more memory. Source maps can be enabled with `-DKRKR2_WEB_ENABLE_SOURCE_MAP=ON`.

After changing SDK versions, use a fresh build directory and rebuild all dependencies instead of reusing old SDK static libraries. Reconfigure after changing CMake settings or source lists. Stop `coi-server.py` before rebuilding and restart it afterward.

### Output

Core artifacts are generated in `out/web/debug/` or `out/web/release/`:

```text
index.html
index.js
index.wasm
vlfs.js
assets.zip
```

Release additionally provides `manifest.webmanifest`, `sw.js`, and icons in `pwa/`. Debug disables the PWA and clears existing Service Worker caches for the site. Extra files such as `index.worker.js`, symbol maps, and source maps depend on the SDK and build options.

## Run locally and deploy

```bash
python3 coi-server.py out/web/release
```

Open `http://localhost:8080/index.html`. Substitute `out/web/debug` for a Debug build. Run `python3 coi-server.py --help` for all options.

- HTTP defaults to port `8080`. HTTPS on port `8443` starts only when `server.crt` and `server.key` exist beside the script.
- LAN access requires HTTPS with a certificate trusted by the client. Plain HTTP on a LAN address does not satisfy the shared-memory requirement.
- Configure `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` on your host. The repository's [_headers](_headers) works with Cloudflare Pages; other servers need equivalent response headers.
- Deploy HTML, JS, Wasm, resources, and PWA files from the same build together. Preserve build-version query parameters to avoid mixing old and new artifacts.

The Release Service Worker precaches engine assets, uses network-first navigation and cache-first same-origin resources, and reloads after an accepted update prompt. **Force Update** clears app caches and reloads while retaining IndexedDB saves. PWA caching does not mean a remote game has been fully downloaded or is playable offline.

## Load a game

Prefer a **complete game directory** or a ZIP containing all game resources. Supplying just one archive from a multi-XP3 game typically leaves scripts, patches, or assets missing.

### Local files and directories

- **Open Game Directory** uses the File System Access API to read an authorized directory and sync engine writes back to it.
- File and folder selection use browser-provided files and allow choosing a named save space before starting.
- The local ZIP picker is visible by default in Debug; enable it in Release with `?pickZip=1`.
- Multiple XP3 files trigger an entry selector. Choosing an entry retains the other game resources.

### Serve a game through the development server

For a complete game ZIP:

```bash
python3 coi-server.py out/web/release --zip /path/to/game.zip --entry data.xp3
```

`--entry` is optional. The server maps the ZIP to `/game.zip` and prints its startup URL. For a game that needs only one XP3:

```bash
python3 coi-server.py out/web/release --xp3 /path/to/game/data.xp3
```

The file is mapped to `/data.xp3`. Single-XP3 URL loading prefers HTTP Range and falls back to downloading a complete Blob when Range is unavailable.

### URL parameters and WebDAV

Choose one game source: `xp3`, `game`, or `webdavUrl`. URL-encode parameter values.

| Parameter | Description |
|-----------|-------------|
| `xp3=<url>` | Load a single XP3; prefers Range independently of `loadMode` |
| `game=<url>` | Load a ZIP containing the game directory |
| `entry=<name>` | Select an XP3 entry filename for ZIP or WebDAV loading |
| `loadMode=eager` | Default: download and prepare ZIP / WebDAV resources before starting |
| `loadMode=lazy` | Read ZIP / WebDAV resources on demand; unavailable remote Range support can cause a full-file fallback |
| `webdavUrl=<url>` | Recursively scan and load a WebDAV collection |
| `webdavUsername=<user>` / `webdavPassword=<password>` | Optional WebDAV HTTP Basic credentials |
| `webdavWrite=1` | Enable save write-back to WebDAV; disabled by default |
| `pickZip=1` | Show the local ZIP picker |
| `renderer=software` / `renderer=opengl` | Override the renderer setting |
| `fps=<positive number>` | Set the browser main-loop target rate, default 15; actual FPS depends on the device and game workload |

Examples:

```text
http://localhost:8080/index.html?game=/game.zip&entry=data.xp3&loadMode=lazy
http://localhost:8080/index.html?webdavUrl=https%3A%2F%2Fdav.example.com%2Fgame%2F&loadMode=lazy
```

Cross-origin resource servers must allow CORS. Remote on-demand reads need correct Range responses and exposed length/range headers. ZIP cache reuse also depends on strong ETags and conditional requests. WebDAV requires `PROPFIND`; write-back additionally needs `MKCOL`, `PUT`, `DELETE`, and suitable credentials/permissions.

The page removes WebDAV credentials from the address bar, but the initial page request can still appear in server or proxy logs. Use HTTPS and do not share URLs containing credentials.

### Saves and game caches

Named save spaces persist in IndexedDB and support ZIP import/export. During manual loading, skipping the save space without using a writable directory leaves saves in the current session only. URL autostart does not show a save-space picker: it only attempts to restore the previous space when the File System Access API is unavailable. Do not assume URL autostart always persists local saves.

The default eager mode for remote ZIPs can store all entries in OPFS and reuse them after a strong ETag / `304` validation on the next launch. Lazy mode reads and extracts on demand. OPFS game caches and IndexedDB saves are separate; clearing site data affects both, so export important saves first.

## Reusable native sessions

The native core supports sequential sessions in one resident library. Each
session uses a fresh owner thread; close joins that owner and its image, wave,
timer, graphics-preload and binary-stream workers before a new session starts.
Renderer/uniform lookup caches belong to the owner thread, not to a process-wide
GL context. Do not reuse a closed owner thread for a different context.

`TVPBeginSystemSession` resets shutdown state before application startup.
`TVPSystemUninit` closes event publication, drains callbacks, invalidates registered
window owners (including script/native reference cycles), unregisters loaded
plugins and destroys the VM. Cancelled script finalizers still invalidate their
native instances. Shutdown registrars remain registered for subsequent sessions.
The optional `tjsExecutionControl.h` cancellation callback covers worker VMs too;
remove it only after every native worker has joined.

Keep the original GL context current while releasing scene references, call
`TVPUninitUIExtension`, recycle textures and release render managers, then destroy
the Director/context. The UI loader must not outlive the ObjectFactory belonging
to that Director. Release the native audio renderer after its sources and workers.
The optional `KRKR2_SHINKI_EMU` CMake option builds the ShinkiEMU core library
from [`shinki/`](./shinki/README.md); it supplies host platform/presentation
endpoints without replacing the VM, OpenGL, FFmpeg, OpenAL or compiled plugins.
Game save locations and per-game preferences retain their original semantics.
The application-path cache also expires with the owner thread, so opening a
different game cannot retain the previous game's save/plugin search root.

---
## Development and verification

| Path | Contents |
|------|----------|
| `cpp/core/` | TJS2, storage, graphics, audio, and platform integration |
| `cpp/plugins/` | Built-in plugins; `motionplayer/` contains MotionPlayer / EmotePlayer |
| `platforms/web/` | Page, VirtualLazyFS JS, JSPI / pthread integration, and PWA |
| `platforms/wasmtime/` | Full-engine headless Wasm guest |
| `tests/web/` | Web loading, caching, scheduler, and artifact consistency tests |
| `tests/unit-tests/` | Native Catch2 unit tests |
| `tests/differential/` | Scalar cases and Android / Wasmtime motion comparisons |
| `tools/` | `xp3`, `xp3pack`, `ksdec`, `tjsdump`, `mtndump`, and `motionsim` |
| `analysis/` | Reference-binary evidence and implementation comparisons |

Web script tests use Node.js. The first three need no engine build; the last checks existing Release artifacts:

```bash
node tests/web/jspi_mainloop_scheduler.test.mjs
node tests/web/pthread_jspi.test.mjs
node tests/web/vlfs_zip_cache.test.mjs
node tests/web/web_release_asset_pair.test.mjs out/web/release
```

With Emscripten active and Wasmtime 44+ installed (CI pins 48.0.0), run all five scalar families:

```bash
python3 tests/differential/run_all.py
```

See the [Wasmtime test guide](tests/differential/wasmtime/README.md) and [Android oracle guide](tests/differential/oracle_runner/README.md). Full motion comparisons require additional reference-engine and game fixtures. They compare node state and semantic render events, and do not yet prove pixel-for-pixel final output. `reference/` is not distributed with this repository.

Web presets disable native tests and tools. macOS native presets are also available. Tools are emitted to `tools/bin/<mac|linux|win>/<rel|dbg>/`; there are currently no ready-made Linux / Windows native presets. Read the four-binary evidence requirements in [CLAUDE.md](CLAUDE.md) before changing reference implementations.

## Game compatibility and license

The [historical game list](doc/support_games.txt) is not a complete compatibility matrix for the current Web build. Results depend on game scripts, plugins, complete resources, and browser capabilities.

MIT License. See [LICENSE](LICENSE).
