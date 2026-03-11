# Tab 自动补全功能

zkfs 现在支持 Tab 自动补全功能！

## 功能特性

### 1. 命令补全
在输入命令时，按 Tab 键可以自动补全命令：
- `l` + Tab → `ls`
- `c` + Tab → 循环显示 `cat`, `cd`, `create`
- `r` + Tab → `rm`
- `q` + Tab → `quit`
- `h` + Tab → `help`

### 2. 支持的命令
- `ls` - 列出子节点
- `dir` - 列出子节点及详细信息
- `cat` - 显示节点数据
- `stat` - 显示节点状态
- `rm` - 删除节点
- `create` / `add` - 创建节点
- `set` - 设置节点数据
- `cd` - 切换路径
- `pwd` - 显示当前路径
- `quit` / `exit` / `q` - 退出
- `help` / `h` - 帮助

### 3. 历史记录
- 命令历史记录保存在 `.zkfs_history` 文件中
- 使用 ↑/↓ 箭头键可以浏览历史命令
- 历史记录的 Tab 补全由 rustyline 自动处理

## 使用方法

```bash
# 进入交互模式
./target/release/zkfs -i -s <zookeeper-server>

# 例如：
./target/release/zkfs -i -s localhost:2181
```

在交互模式中：
1. 输入命令的前几个字母
2. 按 Tab 键自动补全
3. 如果有多个匹配项，多次按 Tab 循环选择

## 技术实现

使用 [rustyline](https://github.com/kkawakam/rustyline) 库实现：
- 自定义 `Completer` trait 实现命令补全
- 支持循环补全模式（Circular completion）
- 命令行历史记录持久化

## 示例

```
zkfs:/> l[TAB]     # 按 Tab 补全为 ls
zkfs:/> ls

zkfs:/> c[TAB]     # 第一次按 Tab
zkfs:/> cat        # 显示 cat
zkfs:/> c[TAB][TAB] # 再次按 Tab 循环
zkfs:/> cd
zkfs:/> c[TAB][TAB]
zkfs:/> create

zkfs:/> quit       # 输入 quit 退出
再见！👋
```

## 依赖

```toml
[dependencies]
rustyline = "14"
```
