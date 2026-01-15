# zkfs 快速参考

## 命令速查表

| 命令 | 说明 | 示例 |
|------|------|------|
| `ls` | 列出子节点 | `zkfs ls /kafka` |
| `dir` | 列出子节点及详细信息 | `zkfs dir /` |
| `cat` | 显示节点数据 | `zkfs cat /config/app` |
| `stat` | 显示节点状态 | `zkfs stat /locks/lock1` |
| `rm` | 删除节点 | `zkfs rm /temp/node` |
| `rm -r` | 递归删除 | `zkfs rm -r /test` |
| `rm -rf` | 强制递归删除 | `zkfs rm -rf /cleanup` |
| `create`/`add` | 创建节点 | `zkfs create /new/node` |
| `set` | 设置节点数据 | `zkfs set /config/db -d "value"` |

## 连接选项

| 选项 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| `-s, --server` | `ZK_SERVER` | `localhost:2181` | Zookeeper服务器地址 |
| `-t, --timeout` | `ZK_TIMEOUT` | `10` | 连接超时（秒） |

## 节点类型

| 类型 | 简写 | 说明 |
|------|------|------|
| `persistent` | `p` | 持久节点（默认） |
| `ephemeral` | `e` | 临时节点 |
| `persistent-sequential` | `ps` | 持久顺序节点 |
| `ephemeral-sequential` | `es` | 临时顺序节点 |

## 数据输入

| 选项 | 说明 | 示例 |
|------|------|------|
| `-d, --data <DATA>` | 从参数输入 | `--data "hello"` |
| `-f, --file <FILE>` | 从文件读取 | `--file config.json` |
| 不指定 | 空数据 | - |

## 常用场景

### 浏览Zookeeper
```bash
zkfs ls /                          # 列出根节点
zkfs dir /kafka                    # 查看kafka详情
zkfs cat /config/database          # 查看配置
```

### 配置管理
```bash
zkfs create /app/config -d "{}"    # 创建配置节点
zkfs set /app/config -f app.json   # 从文件设置
zkfs cat /app/config               # 查看配置
```

### 分布式锁
```bash
zkfs create /locks/resource -t e   # 创建临时锁
zkfs stat /locks/resource          # 查看锁状态
zkfs rm /locks/resource            # 释放锁
```

### 队列操作
```bash
zkfs create /queue/task- -t ps -d "data"  # 添加任务
zkfs ls /queue                             # 查看队列
zkfs cat /queue/task-0000000001            # 读取任务
zkfs rm /queue/task-0000000001             # 删除任务
```

### 批量清理
```bash
zkfs ls /test                      # 查看测试节点
zkfs rm -rf /test                  # 递归删除
```

## 环境变量配置

```bash
# 设置环境变量
export ZK_SERVER=prod-zk:2181
export ZK_TIMEOUT=30

# 使用
zkfs ls /
zkfs cat /config/app
```

## 常见错误

| 错误 | 原因 | 解决方法 |
|------|------|----------|
| 连接失败 | Zookeeper不可达 | 检查服务器地址和网络 |
| NoNode | 节点不存在 | 确认路径正确 |
| NotEmpty | 节点有子节点 | 使用 `rm -r` 递归删除 |
| NodeExists | 节点已存在 | 使用 `set` 而非 `create` |

## 技巧

1. **使用别名**
   ```bash
   alias zk='zkfs -s prod-zk:2181'
   zk ls /
   ```

2. **管道处理**
   ```bash
   zkfs ls /kafka/brokers/ids | wc -l  # 统计broker数量
   zkfs cat /config/app | jq .          # JSON格式化
   ```

3. **脚本化**
   ```bash
   #!/bin/bash
   for node in $(zkfs ls /cleanup); do
     zkfs rm "/cleanup/$node"
   done
   ```

4. **快速测试**
   ```bash
   zkfs create /test/$(date +%s) -d "test"  # 创建测试节点
   zkfs rm -rf /test                        # 清理
   ```
