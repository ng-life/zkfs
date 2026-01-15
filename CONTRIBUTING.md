# 贡献指南

感谢您对 zkfs 项目的关注！我们欢迎各种形式的贡献。

## 如何贡献

### 报告Bug

如果您发现了bug，请创建一个issue并包含以下信息：

- 简洁明了的标题
- 详细的bug描述
- 重现步骤
- 期望的行为
- 实际的行为
- 您的环境信息（操作系统、Rust版本、Zookeeper版本等）

### 提出新功能

如果您有新功能的想法：

1. 首先创建一个issue讨论该功能
2. 等待维护者的反馈
3. 如果获得批准，您可以开始实现

### 提交Pull Request

1. **Fork 仓库**

2. **创建分支**
   ```bash
   git checkout -b feature/your-feature-name
   # 或
   git checkout -b fix/your-bug-fix
   ```

3. **进行开发**
   - 编写清晰的代码
   - 添加必要的注释
   - 遵循现有的代码风格

4. **运行测试和检查**
   ```bash
   # 格式化代码
   cargo fmt
   
   # 运行clippy
   cargo clippy -- -D warnings
   
   # 运行测试
   cargo test
   
   # 构建项目
   cargo build --release
   ```

5. **提交更改**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   # 或
   git commit -m "fix: fix bug description"
   ```

   提交信息格式：
   - `feat:` 新功能
   - `fix:` bug修复
   - `docs:` 文档更新
   - `style:` 代码格式调整
   - `refactor:` 代码重构
   - `test:` 测试相关
   - `chore:` 构建/工具相关

6. **推送到您的Fork**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **创建Pull Request**
   - 在GitHub上创建PR
   - 填写PR模板
   - 等待review

## 代码规范

### Rust代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### 文档

- 为公共API添加文档注释
- 更新README.md（如果需要）
- 中英文文档都应保持最新

### 测试

- 为新功能添加测试
- 确保所有测试通过
- 保持测试覆盖率

## 开发环境设置

### 要求

- Rust 1.70 或更高版本
- Zookeeper 服务器（用于集成测试）

### 设置

1. 克隆仓库
   ```bash
   git clone https://github.com/ng-life/zkfs.git
   cd zkfs
   ```

2. 构建项目
   ```bash
   cargo build
   ```

3. 运行测试
   ```bash
   cargo test
   ```

4. 运行本地版本
   ```bash
   cargo run -- --help
   ```

## 行为准则

请遵守以下准则：

- 尊重他人
- 接受建设性批评
- 专注于对项目最有利的事情
- 对社区其他成员表现出同理心

## 获取帮助

如果您在贡献过程中遇到问题：

- 查看现有的issues
- 在issue中提问
- 查看项目文档

## 许可证

通过贡献代码，您同意您的贡献将在MIT许可证下发布。
