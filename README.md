# 🎮 CleanGal – 纯本地 Galgame 管理器

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-purple)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange)](https://www.rust-lang.org/)
[![Windows](https://img.shields.io/badge/Windows-11%20%7C%2010-blue)](https://www.microsoft.com/windows)

一款基于 **Tauri 2.0** 的纯本地 Galgame 管理工具，专为 Windows 11 设计。轻松整理、启动和记录您的 Galgame 收藏，所有数据完全离线存储，无任何网络请求，安全隐私。

![](https://github.com/2832886442/CleanGal/blob/main/Screenshoot/QQ20260717-163303.png)


---

## ✨ 主要功能

- 📂 **游戏入库** – 手动录入游戏信息，支持选择 `.exe` 启动程序和封面图片。
- 🏷️ **分类与标签** – 自定义分类和标签，快速筛选游戏。
- 🚀 **一键启动** – 双击卡片或点击按钮直接启动游戏，支持启动参数。
- 🎨 **现代化 UI** – 遵循 Win11 设计语言（圆角、亚克力磨砂、深色/浅色主题）。
- 💾 **本地数据存储** – 所有数据（游戏信息、封面、设置）均保存在 `%APPDATA%\CleanGal`，JSON 格式，可手动备份。
- 🧹 **数据清理** – 自动扫描并移除无效的游戏路径。
- 📤 **备份与恢复** – 一键导出/导入全部数据。
- 🖼️ **封面管理** – 支持 PNG、JPG、GIF、BMP 等常见格式，封面以 **Base64** 嵌入 JSON，无需额外文件管理。
- 🌙 **深色模式** – 跟随系统或手动切换。
- 🔒 **纯本地运行** – 无网络请求，无云端同步，完全离线。

---

## 🛠️ 技术栈

| 前端                      | 后端                               |
| ------------------------- | ---------------------------------- |
| HTML5 + CSS3              | Rust                               |
| 原生 JavaScript（无框架） | Tauri 2.0                          |
| Windows 11 风格适配       | Serde (序列化)                     |
| 无构建工具（无需 Vite）   | sysinfo, walkdir, rfd, auto-launch |

---

## 📦 下载与安装

### 预编译版本（推荐）

前往 [Releases](https://github.com/2832886442/CleanGal/releases) 下载最新 `.exe` 安装包，双击运行即可。

### 从源码构建

1. **克隆仓库**：

```bash
   git clone https://github.com/2832886442/CleanGal.git
   cd CleanGal
```

2. **安装 Rust**（如果尚未安装）：
   访问 [https://rustup.rs/](https://rustup.rs/) 安装 Rust。

3. **安装 Tauri CLI**：

```bash
   cargo install tauri-cli
```

4. **开发模式运行**：

```bash
   cargo tauri dev
```

5. **构建发行版**：

```bash
   cargo tauri build
```

   生成的 `.exe` 位于 `src-tauri/target/release/bundle/` 目录下。

> **注意**：本项目不依赖 Node.js/Vite 等前端构建工具，前端代码直接由 Tauri 加载，简化开发流程。

---

## 🚀 使用指南

### 添加游戏

1. 点击侧边栏 **“添加游戏”** 按钮。
2. 填写游戏名称、别名（可选）。
3. 点击 **“浏览”** 选择游戏的启动程序（`.exe`）。
4. 选择或输入分类（如“纯爱”、“拔作”）。
5. 输入标签（逗号分隔，例如 `纯爱, 汉化, 短篇`）。
6. 选择游玩状态（未游玩、游玩中、已通关、搁置）。
7. 可选封面图片（点击 **“选择”** 从本地选取）。
8. 填写简介，点击 **“确认添加”**。

### 管理游戏

- **筛选**：点击分类或标签快速筛选。
- **搜索**：顶部搜索框按名称或别名搜索。
- **视图切换**：网格/列表视图。
- **详情**：点击游戏卡片查看完整信息，并可启动、编辑或删除。
- **主题切换**：侧边栏底部月亮/太阳图标切换深色/浅色模式。

### 设置

- 外观：主题、窗口圆角、界面缩放。
- 数据管理：备份、恢复、清理无效条目。
- 启动选项：开机自启、关闭行为、默认视图。

---

## 📁 数据存储位置

- **游戏数据**：`%APPDATA%\CleanGal\game_list.json`
- **设置**：`%APPDATA%\CleanGal\setting.json`
- **封面图片**：以 **Base64 数据 URI** 直接嵌入 `game_list.json`，无需额外文件管理。

> 💡 卸载软件时，数据不会被自动删除，您可手动备份或删除上述目录以完全清理。

---

## 🐛 常见问题

- **图片无法加载**：若遇到封面不显示，请检查 CSP 设置。本项目已通过 Base64 嵌入彻底解决该问题。
- **启动游戏无反应**：请确认游戏路径正确且 `.exe` 文件可执行。
- **Windows 安全中心拦截**：请将应用添加至白名单。

---

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库。
2. 创建新分支 (`git checkout -b feature/your-feature`)。
3. 提交更改 (`git commit -m 'Add some feature'`)。
4. 推送到分支 (`git push origin feature/your-feature`)。
5. 创建 Pull Request。

---

## 📄 开源协议

本项目采用 **MIT License**，详情见 [LICENSE](LICENSE) 文件。

---

## 🙏 致谢

- [Tauri](https://tauri.app/) – 跨平台应用框架。
- [Rust](https://www.rust-lang.org/) – 安全高效的系统语言。
- 所有开源库的开发者们。
