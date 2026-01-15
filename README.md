# zkfs - Zookeeper 文件系统命令行工具

[![CI](https://github.com/ng-life/zkfs/workflows/CI/badge.svg)](https://github.com/ng-life/zkfs/actions)
[![Security Audit](https://github.com/ng-life/zkfs/workflows/Security%20Audit/badge.svg)](https://github.com/ng-life/zkfs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个用Rust编写的Zookeeper命令行工具，提供类似Linux文件系统命令的操作体验。

## 功能特性

- ✅ 支持通过命令行参数或环境变量配置Zookeeper连接
- ✅ 实现类似Linux的命令：`ls`、`dir`、`cat`、`stat`、`rm`、`create`/`add`、`set`
- ✅ 支持递归删除（`rm -r`）和强制删除（`rm -f`）
- ✅ 支持多种节点类型（持久、临时、顺序节点）
- ✅ 支持从文件读取或命令行参数写入数据
- ✅ 异步高性能操作
- ✅ 友好的中文输出

## 安装

### 从源码编译

```bash
git clone <your-repo>
cd zkfs
cargo build --release
```

编译后的二进制文件位于 `target/release/zkfs`

### 系统要求

- Rust 1.70 或更高版本
- 可访问的Zookeeper服务器

## 使用方法

### 基本语法

```bash
zkfs [选项] <命令> [参数]
```

### 连接配置

可以通过以下方式配置Zookeeper连接：

**方式1：命令行参数**
```bash
zkfs -s localhost:2181 ls /
```

**方式2：环境变量**
```bash
export ZK_SERVER=localhost:2181
export ZK_TIMEOUT=10
zkfs ls /
```

### 可用选项

- `-s, --server <SERVER>` - Zookeeper服务器地址（默认: localhost:2181）
- `-t, --timeout <TIMEOUT>` - 连接超时时间，单位秒（默认: 10）
- `-h, --help` - 显示帮助信息
- `-V, --version` - 显示版本信息

## 命令详解

### ls - 列出子节点

列出指定路径下的所有子节点名称。

```bash
# 列出根节点下的子节点
zkfs ls /

# 列出指定路径的子节点
zkfs ls /kafka/brokers
```

**示例输出：**
```
config
brokers
controller
admin
```

### dir - 详细列表

列出子节点及其详细信息，包括版本、子节点数量、数据大小等。

```bash
# 查看详细信息
zkfs dir /

# 查看指定路径
zkfs dir /kafka
```

**示例输出：**
```
路径: /
子节点数量: 4
数据版本: 0
创建时间: 1705334400000
修改时间: 1705334400000

子节点列表:
  zookeeper (版本: 0, 子节点: 2, 数据大小: 0 bytes)
  kafka (版本: 1, 子节点: 5, 数据大小: 0 bytes)
  config (版本: 0, 子节点: 0, 数据大小: 0 bytes)
  brokers (版本: 3, 子节点: 3, 数据大小: 0 bytes)
```

### cat - 显示节点数据

显示指定节点的数据内容。

```bash
# 查看节点数据
zkfs cat /config/topics/my-topic
```

**示例输出：**
```json
{"version":1,"partitions":{"0":[1,2,3]}}
```

如果数据是二进制格式，将以十六进制显示：
```
(二进制数据，十六进制表示)
48 65 6c 6c 6f 20 57 6f 72 6c 64 0a
```

### stat - 显示节点状态

显示节点的详细状态信息。

```bash
# 查看节点状态
zkfs stat /kafka/controller
```

**示例输出：**
```
节点路径: /kafka/controller
创建事务ID: 100
修改事务ID: 150
创建时间: 1705334400000
修改时间: 1705334500000
数据版本: 5
子节点版本: 0
ACL版本: 0
临时节点所有者: 0
数据长度: 256 bytes
子节点数量: 0
子节点修改事务ID: 100
```

### rm - 删除节点

删除指定的Zookeeper节点。

```bash
# 删除单个节点（节点必须没有子节点）
zkfs rm /config/temp

# 递归删除节点及其所有子节点
zkfs rm -r /test/data

# 强制删除，忽略错误
zkfs rm -f /maybe/not/exist

# 递归强制删除
zkfs rm -rf /test/complete-removal
```

**选项说明：**
- `-r, --recursive` - 递归删除节点及其所有子节点
- `-f, --force` - 强制删除，遇到错误时继续执行（显示警告但不中断）

**示例输出：**
```
成功删除: /config/temp
```

递归删除时：
```
成功删除: /test/data
```

强制模式遇到错误时：
```
警告: 删除 /not/exist 时出错: NoNode
```

**⚠️ 警告：** 删除操作是不可逆的！使用 `-r` 和 `-rf` 选项时要特别小心，特别是在生产环境中。

### create / add - 创建节点

创建新的Zookeeper节点，支持多种节点类型。

```bash
# 创建空的持久节点（默认）
zkfs create /config/app

# 使用别名add
zkfs add /config/app

# 创建带数据的节点
zkfs create /config/database -d "mysql://localhost:3306"

# 从文件读取数据创建节点
zkfs create /config/settings -f config.json

# 创建临时节点（ephemeral）
zkfs create /locks/service-1 -t ephemeral

# 创建持久顺序节点
zkfs create /queue/task- -t persistent-sequential -d "task data"

# 创建临时顺序节点
zkfs create /workers/worker- -t ephemeral-sequential
```

**节点类型：**
- `persistent` (默认, 简写 `p`) - 持久节点，客户端断开后仍然存在
- `ephemeral` (简写 `e`) - 临时节点，客户端断开后自动删除
- `persistent-sequential` (简写 `ps`) - 持久顺序节点，自动添加递增序列号
- `ephemeral-sequential` (简写 `es`) - 临时顺序节点，自动添加递增序列号

**选项说明：**
- `-d, --data <DATA>` - 节点数据（字符串）
- `-f, --file <FILE>` - 从文件读取数据
- `-t, --node-type <TYPE>` - 节点类型

**示例输出：**
```
成功创建节点: /config/app
写入数据: 25 bytes
```

顺序节点输出：
```
成功创建节点: /queue/task-0000000001
写入数据: 9 bytes
```

**注意：** 不能同时指定 `--data` 和 `--file` 参数。

### set - 设置节点数据

修改已存在节点的数据。

```bash
# 设置节点数据
zkfs set /config/database -d "mysql://new-host:3306"

# 从文件读取数据设置
zkfs set /config/settings -f new-config.json

# 清空节点数据（不指定-d或-f）
zkfs set /config/temp
```

**选项说明：**
- `-d, --data <DATA>` - 节点数据（字符串）
- `-f, --file <FILE>` - 从文件读取数据

**示例输出：**
```
成功设置节点数据: /config/database
写入数据: 28 bytes
```

**注意：** 
- 节点必须已存在，否则操作失败
- 不能同时指定 `--data` 和 `--file` 参数
- 如果都不指定，则设置为空数据

## 使用示例

### 连接远程Zookeeper集群

```bash
zkfs -s zk1.example.com:2181,zk2.example.com:2181,zk3.example.com:2181 ls /
```

### 使用环境变量

```bash
# 设置环境变量
export ZK_SERVER=prod-zk.example.com:2181
export ZK_TIMEOUT=30

# 直接使用命令
zkfs ls /production/services
zkfs cat /production/config/database
zkfs stat /production/locks/service-1
```

### 查看Kafka集群信息

```bash
# 列出所有broker
zkfs ls /kafka/brokers/ids

# 查看broker详情
zkfs cat /kafka/brokers/ids/0

# 查看topic配置
zkfs ls /kafka/config/topics
zkfs cat /kafka/config/topics/my-topic
```

### 管理临时测试数据

```bash
# 创建测试后清理（假设你已经创建了测试节点）
zkfs ls /test
zkfs rm -rf /test

# 删除单个配置
zkfs rm /config/obsolete-setting

# 清理特定服务的锁
zkfs stat /locks/service-1
zkfs rm /locks/service-1
```

### 批量操作示例

```bash
# 列出所有要删除的节点
zkfs ls /cleanup/targets

# 逐个删除（非递归）
zkfs rm /cleanup/targets/item1
zkfs rm /cleanup/targets/item2

# 或者直接递归删除整个目录
zkfs rm -rf /cleanup/targets
```

### 配置管理工作流

```bash
# 创建配置节点
zkfs create /app/config/database -d "host=localhost;port=3306"
zkfs create /app/config/redis -d "host=localhost;port=6379"

# 查看配置
zkfs cat /app/config/database

# 更新配置
zkfs set /app/config/database -d "host=prod-db;port=3306"

# 从文件更新配置
echo '{"host":"prod-db","port":3306}' > db.json
zkfs set /app/config/database -f db.json

# 删除配置
zkfs rm /app/config/redis
```

### 分布式锁示例

```bash
# 创建临时节点作为分布式锁
zkfs create /locks/resource-1 -t ephemeral -d "client-id-123"

# 查看锁状态
zkfs stat /locks/resource-1

# 锁会在客户端断开时自动释放（临时节点特性）
```

### 队列操作示例

```bash
# 创建顺序节点作为队列元素
zkfs create /queue/task- -t persistent-sequential -d "task-1-data"
zkfs create /queue/task- -t persistent-sequential -d "task-2-data"
zkfs create /queue/task- -t persistent-sequential -d "task-3-data"

# 查看队列
zkfs ls /queue

# 输出:
# task-0000000001
# task-0000000002
# task-0000000003

# 读取并处理第一个任务
zkfs cat /queue/task-0000000001
zkfs rm /queue/task-0000000001
```

## 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `ZK_SERVER` | Zookeeper服务器地址 | localhost:2181 |
| `ZK_TIMEOUT` | 连接超时时间（秒） | 10 |

## 依赖库

- [clap](https://github.com/clap-rs/clap) - 命令行参数解析
- [zookeeper-client](https://github.com/kezhuw/zookeeper-client-rust) - Zookeeper客户端
- [tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [anyhow](https://github.com/dtolnay/anyhow) - 错误处理

## 开发

### 运行测试

```bash
cargo test
```

### 运行开发版本

```bash
cargo run -- -s localhost:2181 ls /
```

### 代码格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

### CI/CD

本项目使用GitHub Actions进行持续集成和持续部署。详见 [.github/ACTIONS.md](.github/ACTIONS.md)。

## 常见问题

### Q: 连接失败怎么办？

A: 请检查：
1. Zookeeper服务器地址是否正确
2. Zookeeper服务是否正在运行
3. 网络连接是否正常
4. 防火墙是否允许连接到2181端口

### Q: 如何查看完整的错误信息？

A: 可以设置`RUST_LOG`环境变量：
```bash
RUST_LOG=debug zkfs ls /
```

### Q: 支持认证吗？

A: 当前版本暂不支持Zookeeper认证，未来版本会添加此功能。

## 许可证

MIT License

## 贡献

欢迎提交Issue和Pull Request！

详细的贡献指南请参阅 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 更新日志

### v0.1.0 (2026-01-15)
- ✅ 初始版本发布
- ✅ 实现 ls、dir、cat、stat、rm、create/add、set 命令
- ✅ 支持递归删除（rm -r）和强制删除（rm -f）
- ✅ 支持多种节点类型（持久、临时、顺序节点）
- ✅ 支持从文件或命令行参数读写数据
- ✅ 支持环境变量配置
- ✅ 异步操作支持
