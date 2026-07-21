# Changelog

All notable changes to KurisuGal (formerly CleanGal) will be documented in this file.

---

## [v1.2.5] - 2026-07-21

### Changed
- **项目重命名**：CleanGal → KurisuGal，涉及所有配置文件、源码、文档及 UI
- 版本号升级至 v1.2.5
- 侧边栏品牌名称与徽标布局优化，修复因名称变长导致的换行问题

### Added
- 关于界面新增 GitHub 仓库超链接
- CHANGELOG.md 更新日志

### Changed
- 图标文件重组：仅保留 32x32 / 48x48 / 64x64 / 128x128 / 256x256 五组 .ico 图标
- 更新 USER_AGENT 标识与各模块日志路径
- Cargo.toml 包名与 lib 名同步重命名
- tauri.conf.json 的 productName、title、identifier 全部更新

---

## [v1.2.4] - 2026-07-20

### Added
- **可配置数据存储路径**：支持在设置中通过文件浏览器修改数据目录
- **数据迁移机制**：修改路径时弹出确认对话框，自动迁移全部数据（游戏列表、封面、缓存）
- **路径管理模块** (`path_manager.rs`)：统一管理 C 盘配置 vs 程序目录数据
- Bangumi API 双轨策略：v0 API + 旧版 API 自动回退，确保 NSFW 游戏可查询
- 封面独立存储：从 Base64 内嵌改为 `CoverArt/{game_id}.jpg` 文件
- 数据备份与恢复功能（完整 JSON 导出/导入）
- 无效数据自动清理

### Changed
- 游戏数据默认路径改为程序目录 `Data/` 子文件夹
- C 盘仅存系统配置：`%APPDATA%/CleanGal/Config/`
- 数据分类存储：Saves / CoverArt / Cache 子目录

### Fixed
- 路径验证与安全加固
- Mutex 锁中毒处理
- 批量操作性能优化（HashSet 去重）
- 子窗口关闭逻辑：修复鼠标拖拽到窗口外误触发关闭的问题
- 文件选择浏览按钮自定义样式缺失

---

## [v1.2.3] - 2026-07

### Added
- Bangumi (bgm.tv) 游戏数据查询与自动填充
- 封面下载与 Base64 转换

### Changed
- 字体系统重构：自定义字体 + unicode-range 限制日文字符
- CSS 动画效果增强

---

## [v1.2.2] - 2026-07

### Added
- 网格 / 列表双视图切换
- 游玩状态管理（未游玩 | 游玩中 | 已通关 | 搁置）
- 批量选择模式：全选、批量移动分类
- 文件夹递归扫描 .exe 文件

### Changed
- UI 全面升级：赛博朋克风格暗色主题 + 亮色主题
- 模块化 CSS 架构：7 个独立样式文件 + CSS 自定义属性体系

---

## [v1.0.0] - 2026-07

### Added
- 初始版本发布
- 游戏库基本 CRUD 操作
- 一键启动游戏与进程监控
- 游玩时间统计
- 分类与标签系统
- 三主题支持（亮色 / 暗色 / 跟随系统）
- 窗口圆角、界面缩放、开机自启等设置
- 系统托盘支持
