# Changelog

All notable changes to zkfs will be documented in this file.

## [0.2.0] - 2026-03-11

### ✨ Added
- **Tab 自动补全功能**
  - 命令补全：输入命令前缀按 Tab 自动补全
  - ZooKeeper 节点路径补全：支持补全当前目录下的节点
  - 支持相对路径和绝对路径补全
  - 支持 `.` 和 `..` 特殊路径补全
- **交互模式改进**
  - 命令行历史记录（保存在 `.zkfs_history`）
  - 使用 ↑/↓ 箭头键浏览历史命令
  - 循环补全模式

### 🐛 Fixed
- 修复 Tab 补全时的 tokio runtime 冲突问题
- 使用 `block_in_place` 解决异步上下文中的阻塞操作

### 📚 Documentation
- 添加 `TAB_COMPLETE.md` 详细说明 Tab 补全功能
- 更新使用说明和示例

### 🔧 Technical
- 集成 `rustyline` 库实现命令行编辑
- 使用 `Arc<Mutex<String>>` 共享当前路径状态
- 优化路径解析逻辑

---

## [0.1.0] - 2026-03-10

### ✨ Added
- 初始版本发布
- 支持 ZooKeeper 基本操作：`ls`, `dir`, `cat`, `stat`, `rm`, `create`, `set`
- 支持交互模式和单次命令模式
- 类似 Linux 文件系统的使用体验
