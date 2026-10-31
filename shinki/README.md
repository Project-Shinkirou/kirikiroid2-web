# shinki-emu-kirikiroid2-web

ShinkiEMU native core (plugin `kirikiroid2-web`), exported as a Windows DLL or Linux shared library.
The module exports the unified C `shinki_emu_query` table.
`shinki_emu_kirikiroid2_web_query` is its uniquely named linked entry; the Rust
crate `shinki-emu-kirikiroid2-web` exposes that function as `query` and `QUERY`.
The native owner schedules the original VM and Cocos/OpenGL renderer.
Manager presents CPU leases and supplies optional typed window/modal services.
There is no child process, socket protocol, Host PCM requirement, library reload,
or second application event loop. Audio uses the original OpenAL playback device.

The Rust adapter uses edition 2024 and requires Rust 1.98 or newer.

## Source provenance

- Upstream repository: `fenghengzhi/kirikiroid2-web`, `web` branch
  `2028cbccaa91498bd36f270d349ae711d0e8fed3`, merged over the retained
  native repair and ShinkiEMU integration history.
- Hosted fork: `https://github.com/Project-Shinkirou/kirikiroid2-web`, branch `shinki`.
  The adaptation contains this `shinki/` directory (C++ adapter under `native/`,
  Rust query binding and lifecycle tests here),
  the `KRKR2_SHINKI_EMU` CMake option, the retained Windows/MSVC build seams,
  the reusable-session lifecycle and path-cache fixes, the WindowEx overlay
  owner initialization, and FFmpeg 3.3.9 dynamic linking (libbpg uses the
  public HEVC decoder lookup; the Windows FFmpeg port is no longer static-only).
  AlphaMovie uses the equivalent MSVC byte-swap intrinsic on Windows; other
  compilers retain upstream's builtin. Its JPEG coefficient-order table is
  local, preserving the decoder's saturation tail without importing libjpeg's
  private data symbol from a DLL.
- `layerExImage`: `23a9214f9b6e9bc3267b964707efa269c041b919`, compiled unchanged.
- `native/include/shinki_emu.h` is the generated shared header of
  `shinki-emu-abi` at ShinkiEMU `cb8d9dd22d7d0ab129085b324c94d6dc81fa19e3`
  (ABI v3). Cargo pins the shared ABI and build helper revisions in
  `Cargo.toml`; copy the header again whenever the ABI header changes,
  rather than editing this vendored copy independently.

## Ownership and reopen

Only one original VM may run per loaded module. Each successful open owns a fresh
native **thread**, application, VM, original worker set and GL context in the same
process. Close joins that thread; the next open uses the same resident DLL.
Renderer/uniform lookup caches are owner-thread-local and expire when that owner
joins. The general core changes retain static shutdown registrations, reset VM,
plugin registrations, options, storage caches, fonts, glyph textures, logging,
scene pointers and per-context render resources. Original `tTVPThread` workers
are joinable; termination and suspended-start wakeup are synchronized. The image
loader and sound/timer/video producers stop before VM teardown. A failed cleanup
poisons further opens rather than reusing partially destroyed globals.

The cancellation callback exists before synchronous open returns. VM bytecode
checks its owner cancellation every 1024 instructions using a silent exception
which bypasses script try/catch. Cancel invalidates host services independently
of the owner queue. Close returns after callbacks and workers finish. Partial
startup uses the same cleanup path. Native calls that perform blocking external
I/O must return before their owning worker can join; cancellation is cooperative.

A game that ends itself finishes the session without ending the process.
`System.exit`, `System.terminate`, a main-window close accepted by the script
and the exit after an error dialog all reach `TVPExitApplication`; in the
embedded build it records the exit and unwinds the current frame with `EAbort`
(the VM then stops with its silent exception). After that request the owner
treats the unwinding as the end of the frame, stops its loop, tears the native
session down normally and reports `SESSION_FINISHED` with flags 0. A later
pause, resume or modal wait cannot replace FINISHED or FAILED, and `close()`
still returns OK. The same exceptions without an exit request remain failures.
This state machine lives in `native/NativeLifecycle.h`.

Three CPU slots bound frame storage. A published slot is immutable while leased;
when all reusable slots are leased the native canvas stays dirty until a slot is
returned. Lease release owns only its CPU pool, never the closed VM, GL context,
Host callbacks or session. Keep the DLL resident until every lease is released.
Input uses the displayed frame identity and converts raster pixels to authored
stage coordinates once. Stale identities fail explicitly. Horizontal wheel is
unsupported; vertical wheel preserves fractional line detents.
The final game canvas declares ignored alpha: Manager presents its original RGB
as a complete image, rather than compositing it as a straight-alpha layer.
This matches the pinned core's `TVPWindowLayer::UpdateDrawBuffer` in
`cpp/core/environ/cocos2d/MainScene.cpp`: it sends the same texture to
`frame_ready` (lines 1112–1116) and presents it with
`DrawSprite->setBlendFunc(BlendFunc::DISABLE)` (line 1221).

`startup.tjs` directories and conventional `data.xp3` directories use the
original selector entry rule. Only Cocos UI/cache files use a session-owned
writable directory; cleanup removes only that directory. Original
`ApplicationSpecialPath` game `savedata` and `Kirikiroid2Preference.xml` locations
are unchanged. Never run acceptance against the sole commercial save copy.
Only compiled original plugins are registered; arbitrary legacy TPM/DLL loading
is not introduced. Unsupported platform requests fail instead of inventing a
modal answer. No text-hook capability is advertised.
The descriptor names the emulator `KiriKiri2` and the engine `KiriKiri`. It
declares no configuration and no render-extent capability; open rejects a
nonzero `render_extent` with `KRKR_RENDER_EXTENT_UNSUPPORTED`.

Host request failures retain their original typed status, code and message for
the current native operation. If TJS catches a window failure and then requests
an error-reporting modal, a second Host refusal cannot replace the first error.
An error handled by the script is cleared before the next native operation.
Modal caption/message diagnostics retain the script's original error context;
the Host still refuses unavailable window/modal operations without answering them.

## Build prerequisites

Windows x64, MSVC 14.51 (or a compatible verified toolset), CMake >=3.28, Ninja,
WinFlexBison 2.5.25/Bison 3.8.2, native OpenGL and an OpenAL playback device.
Use the retained original vcpkg manifest/overlays: FFmpeg 3.3.9 with Yasm,
Cocos 3.17.2, SevenZip 26.04, UnRAR and all original plugins. Do not substitute
SDK FFmpeg, skip layerExImage, or use the Artemis renderer.

Linux Cocos dependency builds use CMake's built-in `FindFontconfig` module.
The overlay removes Cocos 3.17.2's older pkg-config-only finder so shared
and static builds receive the `Fontconfig::Fontconfig` imported target.

Linux packaging materializes the pinned Cocos FMOD runtime as `libfmod.so.6`
beside the core, with `$ORIGIN` runtime lookup. See `licenses/FMOD.txt` for
the exact binary/archive provenance, original Firelight attribution and
the project owner's redistribution authorization. FMOD remains a separate
binary SDK component; the Cocos MIT license does not relabel that binary.

The Android OpenCV overlay uses vcpkg's `CpuFeatures::ndk_compat` library
and locates its `ndk_compat/cpu-features.h` directory explicitly. The NDK
compatibility API is retained; it does not rely on NDK r30 carrying the
removed `sources/android/cpufeatures` headers.

Android builds target API 28, matching the ShinkiEMU vcpkg triplets and their
bionic iconv dependency. The adapter preserves the supplied vcpkg triplet and
builds a shared `.so` for standalone core packages on both arm64-v8a and x86_64.
Breakpad remains enabled. The manifest supplies Android's Oboe audio library
required by the original sound module. Its pinned vcpkg port installs headers and
the library without a CMake package config; the sound target locates those real
inputs and retains Oboe's `log`/`OpenSLES` link dependencies.
Cocos UI resources are included in the package.
The libgdiplus overlay fixes up its exported CMake configuration so dependency
paths follow the selected install root when restored from a binary cache.
The Android owner creates an OpenGL ES 2 EGL pbuffer on its native thread,
uses the original Cocos renderer and reads back the game canvas for Manager.
It owns and destroys its EGL context/surface without terminating the process's
shared display. CoreApi supplies input; no GLFW window or Java event loop is used.
Its Cocos Android support archive is linked normally, preserving the required
support objects without forcing the separate Java-Activity bootstrap and its
competing `JNI_OnLoad` into the CoreApi-owned module. SDL keeps its own loader.

FFmpeg is dynamically linked (LGPL-2.1): the `x64-windows` triplet builds
`avcodec-57.dll`, `avformat-57.dll`, `avutil-55.dll` and the other FFmpeg DLLs,
and the core imports them. libbpg selects the HEVC decoder through the public
`avcodec_find_decoder_by_name("hevc")`, and `NativeSession` calls
`TVPInitLibAVCodec()` during single-threaded startup so the decoders are
registered before any BPG image loads. Ship every FFmpeg DLL that
`dumpbin /dependents shinki_emu_kirikiroid2_web.dll` lists (from the vcpkg
`installed/x64-windows/bin/` directory) in the core directory.

From a VS x64 developer shell in the ShinkiEMU checkout, prepare dependencies
explicitly, then build with Ninja (avoids MSBuild's long-path file-tracking limit):

```powershell
$env:CMAKE_GENERATOR = "Ninja"
cargo xtask deps --target x86_64-pc-windows-msvc --core kirikiroid2-web
cargo xtask build --target x86_64-pc-windows-msvc --core kirikiroid2-web --release
```

The adapter's Cargo entry runs the root CMake project with `KRKR2_SHINKI_EMU=ON`.
It builds the original shared core and the lifecycle test library with at most
four Cargo jobs (`CARGO_BUILD_JOBS=4`). CMake runs offline, with vcpkg installation
disabled; missing dependencies fail with a preparation diagnostic.
The upstream Python parser/resource generators remain build tools.

For direct Cargo invocation, use a VS x64 developer shell and set `VCPKG_ROOT`,
`SHINKI_EMU_NATIVE_VCPKG_ROOT` (the directory containing `kirikiroid2-web/installed`),
`SHINKI_EMU_VCPKG_TRIPLET=x64-windows`, and `BISON_EXECUTABLE`.
The build script requires matching native Windows x64/MSVC or Linux x64/GNU `HOST` and `TARGET`.
`KRKR2_SHINKI_EMU` defaults to `OFF`; Web and other platform builds are unaffected.

Cargo owns the native tree at `OUT_DIR/native`. The target writes
`shinki_emu_kirikiroid2_web.dll`/`.lib`, app-local dependencies,
the test-only static library and a copied `Resources/` into its `shinki/`
subdirectory. `OUT_DIR/core-build.json` records the real DLL, built resource
directory and source/vcpkg licenses for xtask. No prebuilt DLL override or
source-tree resource installation is used.

The Linux entry uses `SHINKI_EMU_VCPKG_TRIPLET=x64-linux-dynamic`, system Bison,
and CMake >=3.28. It records `libshinki_emu_kirikiroid2_web.so` and the test-only
`.a` library. UTF-8 validation, module paths and hidden GLFW windows no longer
depend on Win32; Linux modal requests and exit use the embedded session owner.
Linux dependency preparation and package/load validation are not complete;
the ShinkiEMU build inventory therefore retains its Linux block.

The Rust crate links the import library and checks the C query table. Its unit
tests also link `shinki_emu_kirikiroid2_web_lifecycle_test.lib`, built from
`native/NativeLifecycleTest.cpp`. It drives the production `NativeLifecycle.h`
state machine without opening a VM or GL context; the shipped DLL neither
contains nor exports the harness. For a direct Cargo test run, add the native
output directory and the MSVC OpenMP redist (`vcomp140.dll`) to `PATH`, then run
`cargo test --manifest-path shinki/Cargo.toml --locked --release -p shinki-emu-kirikiroid2-web`.

A ShinkiEMU core directory (`shinki_emu_kirikiroid2_web/` with `core.json`)
contains the DLL, `Resources/`, every app-local DLL of the output directory,
the FFmpeg 3.3.9 DLLs and their `iconv-2`, `liblzma`, `bz2`, `zlib1`
dependencies from the vcpkg `bin/`, and `vcomp140.dll`. `msvcp140.dll`,
`vcruntime140.dll` and `vcruntime140_1.dll` ship at the package root, not in
the core directory. Ship `LICENSE`, `cpp/plugins/layerExImage/LICENSE.TXT`,
`ui/cocos-studio/DroidSansFallback.LICENSE.txt` and the vcpkg
`share/*/copyright` files of the installed dependencies with it.

## Manual Release acceptance

Acceptance runs only in the real Release Manager. The user selects the game,
operates native menus and save/load, then closes and reopens it. Scripted input,
capture/reopen drivers and the native lifecycle harness have been removed.
Build success is not a manual-playability result.

The production adapter preserves the first Host service error, the original
OpenGL/OpenAL pipeline, and `ALPHA_IGNORED` for the final native canvas.
WindowEx now initializes its optional overlay owner before destruction; the
previous uninitialized pointer could cause an invalid delete during shutdown.
This generic lifetime fix does not alter game scripts or use process isolation.

Earlier development observations and raw diagnostics remain in the source
project's history. They are not re-run automatically. Original game
files and existing saves must remain unchanged; user-reported manual
interruptions are not engine failures or completed routes.
