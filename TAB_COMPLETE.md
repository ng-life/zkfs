# Tab 自动补全功能

zkfs 现在支持完整的 Tab 自动补全功能！

## 功能特性

### 1. 命令补全
在输入命令时，按 Tab 键可以自动补全命令：
- `l` + Tab → `ls`
- `c` + Tab → 循环显示 `cat`, `cd`, `create`
- `r` + Tab → `rm`
- `q` + Tab → `quit`
- `h` + Tab → `help`

### 2. ZooKeeper 路径补全 ✨
在需要路径的命令后，按 Tab 可以自动补全 ZooKeeper 节点路径：
- `ls /[TAB]` → 列出根目录下的节点
- `cd /[TAB]` → 补全根目录下的节点
- `cat /[TAB]` → 补全路径
- 支持相对路径补全（基于当前路径）
- 支持 `. `和`..` 补全

#### 路径补全示例
```
zkfs:/> cd /[TAB][TAB]
# 显示根目录下的所有节点：zookeeper, services, config...

zkfs:/services> ls a[TAB]
# 自动补全为：ls /services/api

zkfs:/services> cd ..[TAB]
# 自动补全为：cd ..
```

### 3. 支持的命令（带路径补全）
- `ls [路径]` - 列出子节点
- `dir [路径]` - 列出子节点及详细信息
- `cat <路径>` - 显示节点数据
- `stat <路径>` - 显示节点状态
- `rm [-r] [-f] <路径>` - 删除节点
- `create <路径>` - 创建节点
- `set <路径>` - 设置节点数据
- `cd [路径]` - 切换路径

### 4. 历史记录
- 命令历史记录保存在 `.zkfs_history` 文件中
- 使用 ↑/↓ 箭头键可以浏览历史命令

## 使用方法

```bash
# 进入交互模式
./target/release/zkfs -i -s <zookeeper-server>

# 例如：
./target/release/zkfs -i -s localhost:2181
```

在交互模式中：
1. **命令补全**：输入命令的前几个字母，按 Tab 补全
2. **路径补全**：输入命令后，输入路径前缀，按 Tab 补全 ZooKeeper 节点
3. **循环选择**：多个匹配项时，多次按 Tab 循环选择

## 技术实现

使用 [rustyline](https://github.com/kkawakam/rustyline) 库实现：
- 自定义 `Completer` trait 实现命令和路径补全
- 支持循环补全模式（Circular completion）
- 命令行历史记录持久化
- 异步 ZooKeeper 客户端同步调用（通过 tokio runtime）

## 完整示例

```
$ ./target/release/zkfs -i -s localhost:2181

交互式 Zookeeper 文件系统 (类似 telnet)
输入命令执行操作，输入 'quit' 或 'exit' 退出，输入 'help' 查看帮助
支持 Tab 自动补全（命令、ZooKeeper 路径）

zkfs:/> l[TAB]           # 补全为 ls
zkfs:/> ls
zookeeper
services
config

zkfs:/> cd s[TAB]        # 补全为 cd services
zkfs:/services> ls
api
cache
db

zkfs:/services> cd a[TAB]  # 补全为 cd api
zkfs:/services/api> pwd
/services/api

zkfs:/services/api> cat c[TAB]  # 补全路径
# 如果有 config 节点，会补全为 cat /services/api/config

zkfs:/services/api> quit
再见！👋
```

## 依赖

```toml
[dependencies]
rustyline = "14"
tokio = { version = "1", features = ["full"] }
```
