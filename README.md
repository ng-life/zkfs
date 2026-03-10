# zkfs - Zookeeper 文件系统命令行工具

[![CI](https://github.com/ng-life/zkfs/workflows/CI/badge.svg)](https://github.com/ng-life/zkfs/actions)
[![Security Audit](https://github.com/ng-life/zkfs/workflows/Security%20Audit/badge.svg)](https://github.com/ng-life/zkfs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个用 Rust 编写的 Zookeeper 命令行工具，提供类似 Linux 文件系统命令的操作体验。

## 功能特性

- ✅ 支持通过命令行参数或环境变量配置 Zookeeper 连接
- ✅ 实现类似 Linux 的命令：`ls`、`dir`、`cat`、`stat`、`rm`、`create`/`add`、`set`
- ✅ 支持递归删除（`rm -r`）和强制删除（`rm -f`）
- ✅ 支持多种节点类型（持久、临时、顺序节点）
- ✅ 支持从文件读取或命令行参数写入数据
- ✅ **交互模式**（类似 telnet，保持长连接）
- ✅ 异步高性能操作
- ✅ 友好的中文输出

## 安装

### 从源码编译

```bash
git clone https://github.com/ng-life/zkfs.git
cd zkfs
cargo build --release
```

编译后的二进制文件位于 `target/release/zkfs`

### 系统要求

- Rust 1.70 或更高版本
- 可访问的 Zookeeper 服务器

## 使用方法

### 基本语法

```bash
# 单次命令模式
zkfs [选项] <命令> [参数]

# 交互模式（类似 telnet）
zkfs -i [选项]
```

### 连接配置

可以通过以下方式配置 Zookeeper 连接：

**方式 1：命令行参数**
```bash
zkfs -s localhost:2181 ls /
```

**方式 2：环境变量**
```bash
export ZK_SERVER=localhost:2181
export ZK_TIMEOUT=10
zkfs ls /
```

### 可用选项

- `-s, --server <SERVER>` - Zookeeper 服务器地址（默认：localhost:2181）
- `-t, --timeout <TIMEOUT>` - 连接超时时间，单位秒（默认：10）
- `-i, --interactive` - **交互模式**（保持长连接，类似 telnet）
- `-h, --help` - 显示帮助信息
- `-V, --version` - 显示版本信息

## 交互模式 🎯

交互模式是 zkfs 的核心特性，类似 telnet 客户端，保持与 Zookeeper 的长连接，无需每次重新连接。

### 启动交互模式

```bash
# 使用默认服务器（localhost:2181）
zkfs -i

# 指定服务器
zkfs -i -s zk1.example.com:2181

# 使用环境变量
export ZK_SERVER=prod-zk.example.com:2181
zkfs -i
```

### 交互模式示例

```bash
$ zkfs -i -s localhost:2181
✓ 已连接到 Zookeeper: localhost:2181

交互式 Zookeeper 文件系统 (类似 telnet)
输入命令执行操作，输入'quit'或'exit'退出，输入'help'查看帮助

zkfs> ls /
config
brokers
controller
admin

zkfs> dir /kafka
路径：/kafka
子节点数量：5
数据版本：1
创建时间：1705334400000
修改时间：1705334400000

子节点列表:
  brokers (版本：2, 子节点：3, 数据大小：0 bytes)
  config (版本：0, 子节点：2, 数据大小：0 bytes)
  controller (版本：1, 子节点：0, 数据大小：256 bytes)

zkfs> cat /kafka/controller
{"version":1,"brokerid":1,"timestamp":"1705334400000"}

zkfs> create /config/app -d "mysql://localhost:3306"
✓ 成功创建节点：/config/app
  写入数据：25 bytes

zkfs> set /config/app -d "mysql://prod-db:3306"
✓ 成功设置节点数据：/config/app
  写入数据：24 bytes

zkfs> stat /config/app
节点路径：/config/app
创建事务 ID: 100
修改事务 ID: 150
创建时间：1705334400000
修改时间：1705334500000
数据版本：2
子节点版本：0
ACL 版本：0
临时节点所有者：0
数据长度：24 bytes
子节点数量：0
子节点修改事务 ID: 100

zkfs> rm /config/app
✓ 成功删除：/config/app

zkfs> quit
再见！👋
```

### 交互模式命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `ls [路径]` | 列出子节点 | `ls /`, `ls /kafka/brokers` |
| `dir [路径]` | 列出子节点及详细信息 | `dir /`, `dir /kafka` |
| `cat <路径>` | 显示节点数据 | `cat /config/database` |
| `stat <路径>` | 显示节点状态 | `stat /kafka/controller` |
| `rm [-r] [-f] <路径>` | 删除节点 | `rm /temp`, `rm -rf /test` |
| `create <路径> [选项]` | 创建节点 | `create /app -d "data"` |
| `set <路径> [选项>` | 设置节点数据 | `set /app -d "new data"` |
| `help`, `h`, `?` | 显示帮助 | `help` |
| `quit`, `exit`, `q` | 退出 | `quit` |

### create 命令选项

```bash
zkfs> create /config/app -d "mysql://localhost:3306"
zkfs> create /locks/service-1 -t ephemeral
zkfs> create /queue/task- -t persistent-sequential -d "task data"
```

- `-d, --data <数据>` - 节点数据
- `-f, --file <文件>` - 从文件读取数据
- `-t, --node-type <类型>` - 节点类型（persistent/ephemeral/persistent-sequential/ephemeral-sequential）

### set 命令选项

```bash
zkfs> set /config/database -d "mysql://new-host:3306"
zkfs> set /config/settings -f config.json
```

- `-d, --data <数据>` - 节点数据
- `-f, --file <文件>` - 从文件读取数据

## 单次命令模式

如果只需要执行单个命令，可以使用单次命令模式（默认模式）：

```bash
# 列出根节点
zkfs -s localhost:2181 ls /

# 查看节点数据
zkfs -s localhost:2181 cat /kafka/controller

# 创建节点
zkfs -s localhost:2181 create /config/app -d "data"

# 删除节点
zkfs -s localhost:2181 rm -rf /test
```

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

### cat - 显示节点数据

显示指定节点的数据内容。

```bash
# 查看节点数据
zkfs cat /config/topics/my-topic
```

### stat - 显示节点状态

显示节点的详细状态信息。

```bash
# 查看节点状态
zkfs stat /kafka/controller
```

### rm - 删除节点

删除指定的 Zookeeper 节点。

```bash
# 删除单个节点
zkfs rm /config/temp

# 递归删除节点及其所有子节点
zkfs rm -r /test/data

# 强制删除，忽略错误
zkfs rm -f /maybe/not/exist

# 递归强制删除
zkfs rm -rf /test/complete-removal
```

**⚠️ 警告：** 删除操作是不可逆的！使用 `-r` 和 `-rf` 选项时要特别小心。

### create / add - 创建节点

创建新的 Zookeeper 节点，支持多种节点类型。

```bash
# 创建空的持久节点（默认）
zkfs create /config/app

# 创建带数据的节点
zkfs create /config/database -d "mysql://localhost:3306"

# 从文件读取数据创建节点
zkfs create /config/settings -f config.json

# 创建临时节点（ephemeral）
zkfs create /locks/service-1 -t ephemeral

# 创建持久顺序节点
zkfs create /queue/task- -t persistent-sequential -d "task data"
```

**节点类型：**
- `persistent` (默认，简写 `p`) - 持久节点
- `ephemeral` (简写 `e`) - 临时节点，客户端断开后自动删除
- `persistent-sequential` (简写 `ps`) - 持久顺序节点
- `ephemeral-sequential` (简写 `es`) - 临时顺序节点

### set - 设置节点数据

修改已存在节点的数据。

```bash
# 设置节点数据
zkfs set /config/database -d "mysql://new-host:3306"

# 从文件读取数据设置
zkfs set /config/settings -f new-config.json
```

## 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `ZK_SERVER` | Zookeeper 服务器地址 | localhost:2181 |
| `ZK_TIMEOUT` | 连接超时时间（秒） | 10 |

## 使用示例

### 交互模式管理 Kafka 集群

```bash
$ zkfs -i -s zk1.example.com:2181
✓ 已连接到 Zookeeper: zk1.example.com:2181

zkfs> ls /kafka/brokers/ids
0
1
2

zkfs> cat /kafka/brokers/ids/0
{"version":4,"host":"kafka1.example.com","port":9092,"endpoints":["PLAINTEXT://kafka1.example.com:9092"]}

zkfs> ls /kafka/config/topics
my-topic
test-topic

zkfs> cat /kafka/config/topics/my-topic
{"version":1,"partitions":{"0":[1,2,3]}}

zkfs> quit
```

### 单次命令查看集群状态

```bash
# 列出所有 broker
zkfs -s zk1.example.com:2181 ls /kafka/brokers/ids

# 查看 broker 详情
zkfs -s zk1.example.com:2181 cat /kafka/brokers/ids/0

# 查看 topic 配置
zkfs -s zk1.example.com:2181 ls /kafka/config/topics
```

### 分布式锁示例（交互模式）

```bash
$ zkfs -i

zkfs> create /locks/resource-1 -t ephemeral -d "client-id-123"
✓ 成功创建节点：/locks/resource-1

zkfs> stat /locks/resource-1
节点路径：/locks/resource-1
...
临时节点所有者：123456789

# 锁会在客户端断开时自动释放（临时节点特性）
zkfs> quit
```

### 队列操作示例

```bash
$ zkfs -i

# 创建顺序节点作为队列元素
zkfs> create /queue/task- -t persistent-sequential -d "task-1-data"
✓ 成功创建节点：/queue/task-0000000001

zkfs> create /queue/task- -t persistent-sequential -d "task-2-data"
✓ 成功创建节点：/queue/task-0000000002

zkfs> create /queue/task- -t persistent-sequential -d "task-3-data"
✓ 成功创建节点：/queue/task-0000000003

# 查看队列
zkfs> ls /queue
task-0000000001
task-0000000002
task-0000000003

# 读取并处理第一个任务
zkfs> cat /queue/task-0000000001
task-1-data

zkfs> rm /queue/task-0000000001
✓ 成功删除：/queue/task-0000000001
```

## 依赖库

- [clap](https://github.com/clap-rs/clap) - 命令行参数解析
- [zookeeper-client](https://github.com/kezhuw/zookeeper-client-rust) - Zookeeper 客户端
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

## 常见问题

### Q: 交互模式和单次命令模式有什么区别？

A: 
- **交互模式** (`zkfs -i`)：建立一次连接后保持长连接，可以连续执行多个命令，类似 telnet 客户端。适合需要多次操作的场景。
- **单次命令模式**（默认）：每次执行命令都重新建立连接，执行完后断开。适合脚本调用或一次性操作。

### Q: 连接失败怎么办？

A: 请检查：
1. Zookeeper 服务器地址是否正确
2. Zookeeper 服务是否正在运行
3. 网络连接是否正常
4. 防火墙是否允许连接到 2181 端口

### Q: 支持认证吗？

A: 当前版本暂不支持 Zookeeper 认证，未来版本会添加此功能。

## 更新日志

### v0.2.0 (2026-03-10)
- ✅ 添加交互模式（类似 telnet，保持长连接）
- ✅ 支持在交互模式中使用所有命令
- ✅ 添加简洁的命令行提示符
- ✅ 优化输出格式，使用✓和⚠符号

### v0.1.0 (2026-01-15)
- ✅ 初始版本发布
- ✅ 实现 ls、dir、cat、stat、rm、create/add、set 命令
- ✅ 支持递归删除（rm -r）和强制删除（rm -f）
- ✅ 支持多种节点类型（持久、临时、顺序节点）
- ✅ 支持从文件或命令行参数读写数据
- ✅ 支持环境变量配置
- ✅ 异步操作支持

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
