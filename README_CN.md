# KrKr2 Web

基于 WebAssembly 的 **KiriKiri2 / Kirikiroid2 引擎**移植，让吉里吉里游戏在浏览器中运行。

> 这是一个专注于 Web 平台的个人分支，不接受 Pull Request。

**语言 / Language**: 中文 | [English](README.md)

## 项目现状

项目持续依据 Kirikiroid2 1.3.9 的 Android / iOS、arm64 / armv7 四个参考二进制还原引擎与插件，目标包括源码结构、数据流、调用链和对象生命周期的一致性。当前实现仍在完善，不代表所有 KiriKiri2 游戏或插件都已兼容。

- Web 构建已使用 **JSPI + WebAssembly 原生异常 + pthreads**，Worker 按需创建并复用。
- **VirtualLazyFS** 统一读取本地文件、目录、ZIP 和远程资源；UI 资源通过 `assets.zip` 加载，不再生成 `index.data`。
- 可从本地目录、单个 XP3、游戏 ZIP 或 WebDAV 目录启动；支持存档空间、ZIP 存档导入导出和获授权目录的写回。
- MotionPlayer / EmotePlayer、PSB、AlphaMovie、LZFS 等插件已有实现与测试，仍在持续核对参考行为。
- Release 提供 PWA；开发验证包括 Web 脚本测试、原生单元测试和 Wasmtime / Android 差分测试。

## 浏览器与运行环境

当前 Web 构建以 **Chromium 137+（如桌面 Chrome / Edge）** 为主要目标。必须具备：

- WebAssembly JSPI：`WebAssembly.Suspending` 和 `WebAssembly.promising`。
- WebAssembly SIMD、原生异常、共享内存，以及 WebGL。
- `SharedArrayBuffer` 和跨域隔离；页面须运行于 `localhost` 或可信 HTTPS。
- ZIP 解压与缓存需要 OPFS（浏览器私有文件系统）和 `DecompressionStream('deflate-raw')`。

仅支持 WebAssembly + SharedArrayBuffer 并不足够。其他浏览器需满足上述能力；页面会检测 JSPI，不能把 Firefox、Safari 或移动端浏览器名称本身视为兼容保证。直接访问本地目录还需要 File System Access API，否则使用文件或文件夹选择入口。

## 编译

### 依赖与环境

- [Emscripten SDK](https://emscripten.org/docs/getting_started/downloads.html) **6.0.9**：CMake 和 CI 均固定此版本，其他版本会被配置检查拒绝。
- [vcpkg](https://learn.microsoft.com/en-us/vcpkg/get_started/get-started)、Ninja、CMake **3.31.1+**、Bison **3.8.2+**、Python 3。
- Linux 构建依赖还包括 `autoconf`、`automake`、`libtool`；具体 CI 配置见 [build-web.yml](.github/workflows/build-web.yml)。

在仓库根目录执行。以下环境设置适用于 Bash / Zsh：

```bash
git submodule update --init --recursive

export VCPKG_ROOT=/path/to/vcpkg
/path/to/emsdk/emsdk install 6.0.9
/path/to/emsdk/emsdk activate 6.0.9
source /path/to/emsdk/emsdk_env.sh
```

子模块包含 `layerExImage`；未初始化时 CMake 会跳过该插件。Windows 使用对应的 `emsdk.bat` / `emsdk_env.bat`，并确保 `EMSDK_PYTHON` 指向有效的 Python 3。macOS 若选中了系统旧版 Bison，配置时通过 `-DBISON_EXECUTABLE=/实际路径/bison` 指定新版。

先按 CI 方式预编译 Emscripten ports，再配置项目：

```bash
embuilder build libpng libpng-mt libpng-legacysjlj libpng-mt-legacysjlj
embuilder build freetype freetype-legacysjlj
embuilder build harfbuzz harfbuzz-mt
embuilder build sdl2 sdl2-mt
embuilder build sdl2_ttf sdl2_ttf-mt

cmake --preset "Web Release Config"
cmake --build out/web/release
```

调试版：

```bash
cmake --preset "Web Debug Config"
cmake --build out/web/debug
```

普通 Debug 不启用 ASan；需要时使用 `Web Debug Asan Config`，输出到 `out/web/debug-asan`，内存需求更高。源码映射可通过 `-DKRKR2_WEB_ENABLE_SOURCE_MAP=ON` 开启。

切换 SDK 后使用新的构建目录并重编译所有依赖，不复用旧 SDK 静态库。修改 CMake 配置或源文件清单后重新运行配置命令。重新构建前停止 `coi-server.py`，构建完成后再启动。

### 产物

Debug / Release 的核心产物分别位于 `out/web/debug/` 和 `out/web/release/`：

```text
index.html
index.js
index.wasm
vlfs.js
assets.zip
```

Release 还提供 `manifest.webmanifest`、`sw.js` 和 `pwa/` 图标；Debug 不启用 PWA，并清理当前站点已有的 Service Worker 缓存。`index.worker.js`、符号表和源码映射等额外文件取决于 SDK 和构建选项。

## 本地运行与部署

```bash
python3 coi-server.py out/web/release
```

打开 `http://localhost:8080/index.html`。可用 `out/web/debug` 替换目录；完整参数通过 `python3 coi-server.py --help` 查看。

- HTTP 默认监听 `8080`；只有脚本同级目录存在 `server.crt` 和 `server.key` 时，才同时启用 HTTPS `8443`。
- 局域网访问须使用客户端信任的 HTTPS 证书；普通局域网 HTTP 地址不能满足共享内存要求。
- 自行部署时配置 `Cross-Origin-Opener-Policy: same-origin` 和 `Cross-Origin-Embedder-Policy: require-corp`。仓库的 [_headers](_headers) 可用于 Cloudflare Pages，其他服务器需配置等效响应头。
- 将同一次构建的 HTML、JS、Wasm、资源和 PWA 文件一起部署，保留构建版本查询参数，避免新旧产物混用。

Release 的 Service Worker 预缓存引擎资源，HTML 使用网络优先策略，同源资源使用缓存优先策略；更新提示接受后会重载。页面的 **Force Update** 可清理应用缓存并重新加载，保留 IndexedDB 存档。PWA 缓存不代表远程游戏已完整下载或可离线运行。

## 加载游戏

优先选择**完整游戏目录**或包含全部游戏资源的 ZIP。只提供多 XP3 游戏中的一个归档，通常会缺失脚本、补丁或素材。

### 本地文件与目录

- **Open Game Directory** 使用 File System Access API，读取获授权目录并将引擎写入同步回该目录。
- 文件或文件夹选择入口使用浏览器提供的文件；启动前可选择命名存档空间。
- 本地 ZIP 选择按钮在 Debug 默认显示，Release 可用 `?pickZip=1` 显示。
- 包含多个 XP3 时会显示入口选择器；选择入口不会丢弃其他游戏资源。

### 通过开发服务器托管

完整游戏 ZIP：

```bash
python3 coi-server.py out/web/release --zip /path/to/game.zip --entry data.xp3
```

`--entry` 可省略；服务器将 ZIP 映射到 `/game.zip` 并打印启动 URL。游戏确实只需要一个 XP3 时，也可使用：

```bash
python3 coi-server.py out/web/release --xp3 /path/to/game/data.xp3
```

该文件映射到 `/data.xp3`。单 XP3 URL 加载优先使用 HTTP Range，不可用时回退为完整 Blob 下载。

### URL 参数与 WebDAV

游戏来源选用 `xp3`、`game`、`webdavUrl` 之一；参数值须进行 URL 编码。

| 参数 | 说明 |
|------|------|
| `xp3=<url>` | 加载单个 XP3；独立于 `loadMode`，优先使用 Range |
| `game=<url>` | 加载包含游戏目录的 ZIP |
| `entry=<name>` | 为 ZIP 或 WebDAV 指定 XP3 入口文件名 |
| `loadMode=eager` | 默认模式：启动前下载并准备 ZIP / WebDAV 资源 |
| `loadMode=lazy` | ZIP / WebDAV 按需读取；远程 Range 不可用时可能回退到完整文件下载 |
| `webdavUrl=<url>` | 递归扫描并加载 WebDAV collection |
| `webdavUsername=<user>` / `webdavPassword=<password>` | 可选的 WebDAV HTTP Basic 认证信息 |
| `webdavWrite=1` | 开启存档写回 WebDAV，默认关闭 |
| `pickZip=1` | 显示本地 ZIP 选择按钮 |
| `renderer=software` / `renderer=opengl` | 覆盖渲染器设置 |
| `fps=<正数>` | 设置浏览器主循环目标帧率，默认 15；实际帧率受设备与游戏负载影响 |

例如：

```text
http://localhost:8080/index.html?game=/game.zip&entry=data.xp3&loadMode=lazy
http://localhost:8080/index.html?webdavUrl=https%3A%2F%2Fdav.example.com%2Fgame%2F&loadMode=lazy
```

跨域资源服务器必须配置 CORS。远程按需读取需要正确的 Range 响应及可读的长度/范围响应头；ZIP 缓存复用还依赖强 ETag 和条件请求。WebDAV 需允许 `PROPFIND`；写回还需要 `MKCOL`、`PUT`、`DELETE` 及相应认证权限。

WebDAV 认证参数会被页面从地址栏移除，但初次页面请求仍可能记录在服务器或代理日志中；使用 HTTPS，不要分享含凭据的 URL。

### 存档与游戏缓存

命名存档空间通过 IndexedDB 持久化，可导入和导出 ZIP。手动加载时若跳过存档空间、也未使用可写目录，存档只保留在当前会话中。URL 自动启动不会弹出存档空间选择器：仅在不支持 File System Access API 时尝试恢复上次空间，因此不能假定 URL 启动总会持久化本地存档。

远程 ZIP 的默认 eager 模式可将全部条目存入 OPFS，并在下次启动通过强 ETag / `304` 校验后复用；lazy 模式按需读取、解压。OPFS 游戏缓存与 IndexedDB 存档是独立数据；清除站点数据会影响二者，重要存档应先导出。

## 开发与验证

| 路径 | 内容 |
|------|------|
| `cpp/core/` | TJS2、存储、图形、音频及平台适配 |
| `cpp/plugins/` | 内置插件；`motionplayer/` 包含 MotionPlayer / EmotePlayer |
| `platforms/web/` | 页面、VirtualLazyFS JS、JSPI / pthread 适配和 PWA |
| `platforms/wasmtime/` | 完整引擎的无窗口 Wasm guest |
| `tests/web/` | Web 加载、缓存、调度器与产物一致性测试 |
| `tests/unit-tests/` | 原生 Catch2 单元测试 |
| `tests/differential/` | 标量用例及 Android / Wasmtime motion 差分 |
| `tools/` | `xp3`、`xp3pack`、`ksdec`、`tjsdump`、`mtndump`、`motionsim` |
| `analysis/` | 参考二进制取证与实现对照笔记 |

Web 脚本测试使用 Node.js；前三条无需引擎构建，最后一条检查已有 Release 产物：

```bash
node tests/web/jspi_mainloop_scheduler.test.mjs
node tests/web/pthread_jspi.test.mjs
node tests/web/vlfs_zip_cache.test.mjs
node tests/web/web_release_asset_pair.test.mjs out/web/release
```

启用 Emscripten 环境并安装 Wasmtime 44+（CI 固定 48.0.0）后，可运行五组标量测试：

```bash
python3 tests/differential/run_all.py
```

详见 [Wasmtime 测试说明](tests/differential/wasmtime/README.md) 和 [Android oracle 说明](tests/differential/oracle_runner/README.md)。完整 motion 差分需要额外的参考引擎与游戏物料，比较节点状态和渲染语义事件，尚不能证明最终画面逐像素一致。`reference/` 不随仓库分发。

Web preset 关闭原生测试和工具。仓库另有 macOS 原生 preset；工具输出到 `tools/bin/<mac|linux|win>/<rel|dbg>/`，目前没有现成的 Linux / Windows 原生 preset。修改参考实现前请阅读 [CLAUDE.md](CLAUDE.md) 的四文件取证要求。

## 游戏兼容性与许可证

[历史游戏记录](doc/support_games.txt) 不是当前 Web 版本的完整兼容性测试矩阵；实际运行仍取决于游戏脚本、插件、资源完整性和浏览器能力。

MIT 许可证，详见 [LICENSE](LICENSE)。
