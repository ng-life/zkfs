# GitHub Actions 配置说明

本项目配置了三个GitHub Actions工作流，用于自动化构建、测试和发布。

## 工作流说明

### 1. CI 工作流 (`ci.yml`)

**触发条件：**
- 推送到 `main`, `master`, `develop` 分支
- 针对这些分支的Pull Request

**包含的任务：**
- ✅ **Check**: 代码检查
- ✅ **Test**: 在多平台运行测试 (Linux, macOS, Windows)
- ✅ **Fmt**: 代码格式检查
- ✅ **Clippy**: 代码质量检查
- ✅ **Build**: 多平台构建并上传构建产物

### 2. Release 工作流 (`release.yml`)

**触发条件：**
- 推送标签 `v*` (例如: v0.1.0)

**包含的任务：**
- 📦 创建GitHub Release
- 🔨 在多个平台构建二进制文件：
  - Linux AMD64 (glibc)
  - Linux AMD64 (musl, 静态链接)
  - macOS AMD64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - Windows AMD64
- 📤 上传构建产物到Release
- 📦 发布到 crates.io (可选)

**使用方法：**
```bash
# 创建新版本标签
git tag v0.1.0
git push origin v0.1.0
```

### 3. Security 工作流 (`security.yml`)

**触发条件：**
- 每周一自动运行
- Cargo.toml 或 Cargo.lock 文件变更时
- Pull Request 修改依赖时

**包含的任务：**
- 🔒 安全审计 (cargo-audit)
- 📋 依赖审查 (dependency-review)

## 必需的Secrets配置

如果要发布到crates.io，需要在GitHub仓库设置中添加：

1. 进入仓库设置: `Settings` → `Secrets and variables` → `Actions`
2. 添加以下secrets:
   - `CARGO_TOKEN`: 从 https://crates.io/me 获取的API token

## 本地测试

在推送代码前，可以在本地运行相同的检查：

```bash
# 代码检查
cargo check

# 运行测试
cargo test

# 格式检查
cargo fmt --all -- --check

# Clippy检查
cargo clippy -- -D warnings

# 构建release版本
cargo build --release
```

## 发布流程

1. **更新版本号**
   ```bash
   # 编辑 Cargo.toml，更新 version 字段
   vim Cargo.toml
   ```

2. **更新CHANGELOG**（如果有）

3. **提交更改**
   ```bash
   git add .
   git commit -m "chore: bump version to 0.1.0"
   ```

4. **创建并推送标签**
   ```bash
   git tag v0.1.0
   git push origin main
   git push origin v0.1.0
   ```

5. **等待GitHub Actions完成构建和发布**

6. **检查Release页面**
   访问 `https://github.com/ng-life/zkfs/releases` 查看发布结果

## 故障排查

### 构建失败
- 检查 Actions 标签页的详细日志
- 确保本地 `cargo build --release` 成功
- 确保代码通过 `cargo clippy` 和 `cargo fmt` 检查

### Release失败
- 确保标签格式正确 (v开头，如 v0.1.0)
- 确保有权限创建Release
- 如果发布到crates.io失败，检查CARGO_TOKEN是否正确配置

### 安全审计失败
- 查看具体的漏洞报告
- 运行 `cargo audit` 查看详情
- 更新有漏洞的依赖包
