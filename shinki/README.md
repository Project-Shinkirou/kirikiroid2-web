# shinki-emu-kirikiroid2-web

ShinkiEMU native core (plugin `kirikiroid2-web`), exported as a Windows DLL or Linux shared library.
The module exports the unified C `shinki_emu_query` table.
`shinki_emu_kirikiroid2_web_query` is its uniquely named linked entry; the Rust
crate `shinki-emu-kirikiroid2-web` exposes that function as `query` and `QUERY`.
The native owner schedules the original VM and Cocos/OpenGL renderer.
Manager presents CPU leases and supplies optional typed window/modal services.
There is no child process, socket protocol, Host PCM requirement, library reload,
or second application event loop. Audio uses the original OpenAL playback device.
The embedded context API uses the platform OpenAL header's device/context types,
including Apple's opaque typedefs, shared by the native owner and wave mixer.
Apple links retain the NCB registrar archive as a CMake target dependency and
force-load its objects into the final image, including embedded adapter builds.
The Cocos audio patch renames only its macOS compatibility class; iOS keeps
AVFoundation's real `AVAudioSession` for activation, categories and interruptions.
macOS platform sources are selected from CMake's target system name, including
adapter builds that do not use the standalone application's configure preset.
The selected macOS implementation routes embedded exit and modal/input requests
through the existing session/typed Host tables; standalone input uses AppKit.
iOS FFmpeg cross builds specify the Darwin target OS together with the existing
architecture, device/simulator compiler target and SDK; assembly remains enabled.
Apple FFmpeg pkg-config searches only the selected target installation, including
its matching debug/release libraries. Host Homebrew/MacPorts XCB dependencies
cannot leak into the core; native Apple video/audio and codec support remain.
The shared OpenGL header includes Apple's `TargetConditionals.h` itself and
selects OpenGLES for iOS or OpenGL for macOS. Motion's stencil renderer does not
depend on another header accidentally defining the platform macros first.
LayerExDraw's libgdiplus matrix predicates use the provider's `gboolean` output
ABI on every platform, including Apple headers where Objective-C `BOOL` is a
different type. Their status and actual matrix result are preserved.
Apple libarchive uses iconv headers and libraries from the same selected SDK;
its upstream `/opt/local` search preference cannot introduce host symbol names
or silently lose cross-architecture archive filename conversion.
The macOS adapter is built with its installed `@loader_path` runtime search path,
so the exact inspected build output does not retain absolute vcpkg directories.
Apple plugin compilation prioritizes the linked LZ4 target's headers over host
package-manager prefixes; the reference LZ4 1.7.4 compile-time guard stays intact.

iOS builds retain only the uniquely named query entry and record the actual `native-static` archive and runtime assets. The Manager packages those assets under `cores/kirikiroid2-web/Resources`; the linked native owner resolves that bundle path. Android and iOS share `static_link.rs` and `static_link_plan.rs`: CMake's actual selected archive plan is partially linked into one relocatable object, whose private definitions/references are isolated before the final archive enters Manager. Apple SDK libraries/frameworks remain ordinary imports; unrelated installed libraries do not enter this closure.
iOS sessions own a real offscreen EAGL/OpenGLES context and complete color/depth/stencil framebuffer on the existing native worker. The view honors the requested multisampling storage and fails on an incomplete framebuffer. It does not include the desktop GLFW view or create a second UIKit window/event owner; the original game texture/readback leases still feed Manager, and teardown releases the worker's GL objects and context.
Uchardet's Apple pkg-config template names the SDK's libc++ runtime, matching the actual AppleClang build, instead of upstream's hardcoded GNU libstdc++. The corrected private closure propagates through ordinary pkg-config static linking; no runtime alias or skipped native library is introduced.
Apple targets share the existing POSIX/Foundation/Mach platform provider instead of leaving iOS's filesystem, clock, memory, locale and session-exit symbols undefined. AppKit and desktop directory discovery remain macOS-only; iOS uses its actual sandbox/bundle prefixes and the installed typed modal Host. Directory creation retains `std::filesystem`'s true-on-success result rather than negating it. No iOS desktop-UI substitute or silent modal fallback is installed.
The SDL2 overlay retains the exact 2.32.10 recipe and original source/license. iOS compilation applies a `ShinkiKrkr` macro prefix to every declared SDL Objective-C class and all of its compiled references, matching the existing Renpy SDL source-level mechanism. Impacto SDL3 keeps its original class names; Android's existing private SDL/Cocos Java names and JNI tables are unchanged.

The Rust adapter uses edition 2024 and requires Rust 1.99 or newer.

The Apple FFmpeg overlay configures before cleaning its first architecture;
later architectures clean only an existing configured Makefile. The libarchive
overlay declares its complete compression/XML dependency closure so cross
builds do not discover host MacPorts libraries.
On iOS it builds the embedded archive library without the standalone
`bsdtar`, `bsdcpio`, `bsdcat` and `bsdunzip` command applications. Archive
readers/writers, compression support and the core's in-process extraction
remain enabled; desktop command-tool configuration is unchanged.

## Android Manager static embedding

The `static` feature builds the actual VM, Cocos EGL renderer and original
audio implementation into an archive. CMake's File API supplies the resolved
transitive archive closure. NDK `ld.lld -r` selects that closure without allowing
duplicate definitions; `llvm-objcopy` namespaces every defined private symbol
and its references before the final archive enters the Manager link. SDL,
Cocos JNI bootstrap, private Lua and FFmpeg cannot interpose on another core.
Archive selection preserves CMake's whole-archive scopes so static NCB plugin
registration objects are not discarded just because they have no named caller.
Only the unique query, resource-root setter and Android host initializer retain
their public names.
Android platform and C++/OpenMP runtime imports remain normal shared imports.
Project archives come from CMake's structured target artifact records; relative
link paths use the recorded generator's real command directory. Windows
fragments use the Windows argv parser, not POSIX shell escaping. Native
pkg-config `-l` dependencies enter the same private archive closure.
`static_link_plan.rs` consumer regressions compile and partially link real NDK
archives/callers for paths with spaces/backslashes, Ninja/Makefile directories,
and missing or undeclared archive rejection.

The Manager stages `Resources` and authentic dependency notices through
`native-static` build metadata, extracts APK assets into app-private storage,
then sets the absolute resource directory before registering the query table.
Standalone shared builds retain the generic dynamic query. Static embedding
does not enable downloadable Android dynamic-core installation.

Java consumers are staged from the SHA512-verified Cocos 3.17.2 and SDL 2.32.10
source archives already prepared by vcpkg. Digest bytes are compared to the
pinned hex checksums without relying on an array formatting trait.
SDL uses private `org.krkrsd.app`
classes and Cocos uses `org.krkrc2dx.lib`; equal-width ELF class-name rewriting
matches the staged Java namespaces. SDL's original native registration table
runs through a uniquely named explicit initializer, not a second exported
`JNI_OnLoad`. Cocos helper, bitmap and sensor methods use explicit
`RegisterNatives` tables with their original signatures.

Manager calls `initialize_android` while attached to its AndroidApp JavaVM,
before registry initialization or the first native session. Cocos retains the
application Context/classloader, and owner-thread JNI calls use its original
attach/TLS-detach path. Java helpers borrow the Manager Activity, receive
pause/resume, and release Activity/audio/sensor references after native teardown.
The original Helper `init(Activity)` signature is preserved. A recreated
NativeActivity rebinds Java services after the previous owner has joined; native
VM/classloader setup is process-scoped, and initializer local references are
bounded by a JNI local frame. No Activity local reference is retained by C++.
The Android Cocos overlay queries JavaVM for the current environment on every
entry, detaches only Cocos-owned attachments, and replaces classloader/context
global references without leaking their predecessors. SDL's original setup
runs once on Android's process-attached UI thread, outside scoped Rust
attachments; the original native registration tables and helper code remain.
An embedded exit without an active owner throws instead of falling back to
process termination.
`KR2Activity` is a ContextWrapper service, not another Activity entry point.
Its Android-ID JNI service is named `getAndroidDeviceId` to avoid colliding
with the SDK's inherited `ContextWrapper.getDeviceId(): int`; standalone
platform lookup keeps its original name and signature.
Queued Java callbacks drain on the original core owner; modal requests and
game exit use the existing embedded host rather than the old Activity/process
exit path. Standalone-only advertisement dispatch is excluded from embedding;
the upstream handler itself has no implementation, and no replacement ad
backend is introduced.

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

The cocos2dx overlay accepts iOS triplets and maps CMake's `iOS` system name
to the retained upstream UIKit/OpenGLES platform sources, rather than the
macOS desktop implementation. Device and both simulator triples use their
target SDK through the ShinkiEMU Apple toolchain.
On Apple x86/x64 targets, FFmpeg selects the vcpkg-host Yasm executable
explicitly, retaining assembly support instead of relying on NASM 3's
compatibility with the original FFmpeg 3.3.9 configure probe.
Apple adapter builds consume the original Cocos/core/plugin closure through
a CMake interface target; they do not require the standalone app entrypoint.
Linux Cocos dependency builds use CMake's built-in `FindFontconfig` module.
The overlay removes Cocos 3.17.2's older pkg-config-only finder so shared
and static builds receive the `Fontconfig::Fontconfig` imported target.
The Linux gdiplus text renderer links Pango's Cairo and FreeType modules as
well as its base library, so its `pango_cairo_*` and `pango_fc_*` imports
resolve without symbols borrowed from another consumer.
The Linux Cocos port disables the optional `BUILD_EDITOR_SPINE` module.
The core/plugins have no Spine calls or registrations, and all 35 Cocos Studio
UI descriptions and their 35 compiled resources use ordinary UI nodes, PNG
images and TTF fonts. CocoStudio timelines and the real MotionPlayer/Emote
implementation remain enabled. The original Cocos source archive and its
per-file license exceptions are retained; they are not blanket MIT.
The Linux binary's Cocos notice set excludes the unused Spine module.
`licenses/LICENSE_spine.txt` remains intact in the full upstream source tar
as documentation of that uncompiled component, not as a linked-runtime claim.

Linux packaging materializes the pinned Cocos FMOD runtime as `libfmod.so.6`
beside the core, with `$ORIGIN` runtime lookup. See `licenses/FMOD.txt` for
the exact binary/archive provenance, original Firelight attribution and
the project owner's redistribution authorization. FMOD remains a separate
binary SDK component; the Cocos MIT license does not relabel that binary.

ShinkiEMU's iOS triplets distinguish the Autoconf target system
(`aarch64-apple-ios` or `x86_64-apple-ios`) from the Darwin build host.
This applies to libffi, gettext and other Autotools ports rather than
copying individual ports to override the same platform identity.
The original FFI/closure/assembly and localization implementations remain
enabled; desktop and Android triplets are unchanged.
GLib's iOS intl dependency carries CoreFoundation through both Meson
consumers and generated pkg-config metadata, as required by static gettext's
locale-preference implementation. GIO, localization and native tools remain
enabled; this dependency does not change desktop or Android builds.
Libgdiplus selects Carbon monitor-DPI detection and desktop linker flags
only for macOS. iOS keeps its existing portable Cairo/Pango drawing and text
path, including the library's normal logical DPI; image codecs stay enabled.
OpenCV's iOS OpenMP feature installs the standalone LLVM 18.1.6 CPU runtime,
matching the registry baseline's LLVM version. The runtime builds as a thin
static library with the selected target SDK and keeps its upstream license.
CMake's standard FindOpenMP performs the actual compiler/link probes; no
OpenMP detection results or OpenCV configuration-verification checks are forced.

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
`installed-x64-windows/x64-windows/bin/` directory) in the core directory.

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
`SHINKI_EMU_NATIVE_VCPKG_ROOT` (the directory containing `kirikiroid2-web/installed-<triplet>`),
`SHINKI_EMU_VCPKG_TRIPLET=x64-windows`, and `BISON_EXECUTABLE`.
Windows builds require native x64/MSVC `HOST` and `TARGET`; cross-target Android
builds require the pinned Android SDK/NDK and their own prepared triplet.
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

The common desktop/Android branch preserves the accepted Linux, Apple and Android
histories through real merges. Android's process-lifetime Java host and private
resource root are selected only for Android static embedding. Apple's existing
static bundle-relative resource path remains separate; merging these histories
does not accept or enable a new iOS product.
The Apple platform uses the shared `Platform.h` embedded-modal helper, identical
to its former local implementation, so the Android/shared-header merge does not
introduce a second conflicting definition or remove host modal dispatch.
