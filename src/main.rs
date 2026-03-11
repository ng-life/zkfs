use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context as RlContext, Helper, Result as RlResult, Config, Editor};
use rustyline::error::ReadlineError;
use std::collections::BTreeSet;
use zookeeper_client::{Acls, Client, CreateMode, Stat};

/// Zookeeper 文件系统命令行工具
#[derive(Parser, Debug)]
#[command(name = "zkfs")]
#[command(about = "Zookeeper 命令行工具，支持交互模式", long_about = None)]
struct Cli {
    /// Zookeeper 服务器地址 (例如：localhost:2181)
    #[arg(short, long, env = "ZK_SERVER", default_value = "localhost:2181")]
    server: String,

    /// 连接超时时间（秒）
    #[arg(short, long, env = "ZK_TIMEOUT", default_value = "10")]
    timeout: u64,

    /// 交互模式（类似 telnet，保持连接）
    #[arg(short = 'i', long, default_value = "false")]
    interactive: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// 列出指定路径下的子节点（类似 ls 命令）
    Ls {
        /// Zookeeper 路径
        #[arg(default_value = "/")]
        path: String,
    },
    /// 列出指定路径下的子节点及详细信息（类似 dir 命令）
    Dir {
        /// Zookeeper 路径
        #[arg(default_value = "/")]
        path: String,
    },
    /// 显示节点数据内容（类似 cat 命令）
    Cat {
        /// Zookeeper 路径
        path: String,
    },
    /// 显示节点状态信息（类似 stat 命令）
    Stat {
        /// Zookeeper 路径
        path: String,
    },
    /// 删除节点（类似 rm 命令）
    Rm {
        /// Zookeeper 路径
        path: String,
        /// 递归删除子节点
        #[arg(short, long)]
        recursive: bool,
        /// 强制删除，忽略错误
        #[arg(short, long)]
        force: bool,
    },
    /// 创建节点
    #[command(alias = "add")]
    Create {
        /// Zookeeper 路径
        path: String,
        /// 节点数据（如果不指定则从--file 读取，都不指定则为空数据）
        #[arg(short, long)]
        data: Option<String>,
        /// 从文件读取数据
        #[arg(short, long)]
        file: Option<String>,
        /// 节点类型：persistent(默认), ephemeral, persistent-sequential, ephemeral-sequential
        #[arg(short = 't', long, default_value = "persistent")]
        node_type: String,
    },
    /// 设置节点数据
    Set {
        /// Zookeeper 路径
        path: String,
        /// 节点数据（如果不指定则从--file 读取）
        #[arg(short, long)]
        data: Option<String>,
        /// 从文件读取数据
        #[arg(short, long)]
        file: Option<String>,
    },
    /// 切换当前路径（类似 cd 命令）
    Cd {
        /// Zookeeper 路径（支持 . 和 ..）
        path: String,
    },
    /// 显示当前路径（类似 pwd 命令）
    Pwd,
    /// 退出交互模式
    Quit,
    /// 显示帮助信息
    Help,
}

/// 交互模式状态
struct InteractiveState {
    current_path: String,
}

/// Tab 补全器
struct ZkFsCompleter {
    commands: BTreeSet<&'static str>,
}

impl ZkFsCompleter {
    fn new() -> Self {
        let mut commands = BTreeSet::new();
        commands.insert("ls");
        commands.insert("dir");
        commands.insert("cat");
        commands.insert("stat");
        commands.insert("rm");
        commands.insert("create");
        commands.insert("add");
        commands.insert("set");
        commands.insert("cd");
        commands.insert("pwd");
        commands.insert("quit");
        commands.insert("exit");
        commands.insert("q");
        commands.insert("help");
        commands.insert("h");
        commands.insert("ll");
        
        Self { commands }
    }
}

impl Completer for ZkFsCompleter {
    type Candidate = String;
    
    fn complete(&self, line: &str, pos: usize, _ctx: &RlContext<'_>) -> RlResult<(usize, Vec<String>)> {
        // 找到当前单词的起始位置
        let start = line[..pos].rfind(|c: char| c.is_whitespace()).map_or(0, |i| i + 1);
        let word = &line[start..pos];
        
        // 如果是第一个单词（命令），进行命令补全
        if start == 0 || line[..start].trim().is_empty() {
            let matches: Vec<String> = self.commands
                .iter()
                .filter(|cmd| cmd.starts_with(word))
                .map(|cmd| cmd.to_string())
                .collect();
            return Ok((start, matches));
        }
        
        // 路径补全在外部处理
        Ok((start, vec![]))
    }
}

impl Hinter for ZkFsCompleter {
    type Hint = String;
    
    fn hint(&self, _line: &str, _pos: usize, _ctx: &RlContext<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Helper for ZkFsCompleter {}
impl Highlighter for ZkFsCompleter {}
impl Validator for ZkFsCompleter {}



impl InteractiveState {
    fn new() -> Self {
        Self {
            current_path: "/".to_string(),
        }
    }

    fn resolve_path(&self, input: &str) -> String {
        if input.is_empty() {
            return self.current_path.clone();
        }

        if input == "/" {
            return "/".to_string();
        }

        let result = if input.starts_with('/') {
            input.to_string()
        } else if self.current_path == "/" {
            format!("/{input}")
        } else {
            format!("{}/{}", self.current_path, input)
        };

        // 规范化路径
        let mut parts = Vec::new();
        for part in result.split('/') {
            match part {
                "" | "." => continue,
                ".." => {
                    parts.pop();
                }
                p => parts.push(p),
            }
        }

        if parts.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", parts.join("/"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 创建 Zookeeper 客户端
    let client = Client::connector()
        .connect(&cli.server)
        .await
        .context(format!("无法连接到 Zookeeper 服务器：{}", cli.server))?;

    println!("✓ 已连接到 Zookeeper: {}", cli.server);

    if cli.interactive {
        // 交互模式
        let mut state = InteractiveState::new();
        interactive_mode(&client, &mut state).await?;
    } else {
        // 单次命令模式
        match cli.command {
            Some(cmd) => {
                let mut state = InteractiveState::new();
                execute_command(&client, &cmd, &mut state).await?;
            }
            None => {
                println!("提示：使用 -i 或 --interactive 进入交互模式");
                println!("使用 --help 查看可用命令");
            }
        }
    }

    Ok(())
}

/// 交互模式主循环（带 Tab 补全）
async fn interactive_mode(client: &Client, state: &mut InteractiveState) -> Result<()> {
    println!();
    println!("交互式 Zookeeper 文件系统 (类似 telnet)");
    println!("输入命令执行操作，输入 'quit' 或 'exit' 退出，输入 'help' 查看帮助");
    println!("支持 Tab 自动补全（命令、路径）");
    println!();

    // 设置 rustyline 编辑器
    let config = Config::builder()
        .completion_type(rustyline::CompletionType::Circular)
        .build();
    
    let mut rl = Editor::<ZkFsCompleter, _>::with_config(config)?;
    
    let completer = ZkFsCompleter::new();
    rl.set_helper(Some(completer));
    
    // 加载历史记录
    let _ = rl.load_history(".zkfs_history");
    
    loop {
        let prompt = format!("zkfs:{}> ", state.current_path);
        
        match rl.readline(&prompt) {
            Ok(input) => {
                // 添加到历史记录
                rl.add_history_entry(input.clone())?;
                
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                // 解析命令
                let parts: Vec<&str> = input.split_whitespace().collect();
                let cmd = parts[0].to_lowercase();

                // 检查退出命令
                if cmd == "quit" || cmd == "exit" || cmd == "q" {
                    // 保存历史记录
                    let _ = rl.save_history(".zkfs_history");
                    println!("再见！👋");
                    break;
                }

                // 检查帮助命令
                if cmd == "help" || cmd == "h" || cmd == "?" {
                    print_help(&state.current_path);
                    continue;
                }

                // 解析并执行命令
                let command = parse_interactive_command(input, state);
                match command {
                    Ok(cmd) => {
                        if let Err(e) = execute_command(client, &cmd, state).await {
                            eprintln!("错误：{e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("命令解析错误：{e}");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("按 Ctrl+C 退出。使用 'quit' 命令退出。");
            }
            Err(ReadlineError::Eof) => {
                println!("再见！👋");
                break;
            }
            Err(err) => {
                eprintln!("读取错误：{err}");
                break;
            }
        }
    }
    
    // 保存历史记录
    let _ = rl.save_history(".zkfs_history");

    Ok(())
}

/// 解析交互模式的命令输入
fn parse_interactive_command(input: &str, state: &InteractiveState) -> Result<Commands> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("空命令"));
    }

    let cmd = parts[0].to_lowercase();
    let args = &parts[1..];

    match cmd.as_str() {
        "ls" => Ok(Commands::Ls {
            path: args
                .first()
                .map_or(state.current_path.clone(), |s| state.resolve_path(s)),
        }),
        "dir" => Ok(Commands::Dir {
            path: args
                .first()
                .map_or(state.current_path.clone(), |s| state.resolve_path(s)),
        }),
        "cat" => {
            if args.is_empty() {
                return Err(anyhow::anyhow!("cat 命令需要指定路径"));
            }
            Ok(Commands::Cat {
                path: state.resolve_path(args[0]),
            })
        }
        "stat" => {
            if args.is_empty() {
                return Err(anyhow::anyhow!("stat 命令需要指定路径"));
            }
            Ok(Commands::Stat {
                path: state.resolve_path(args[0]),
            })
        }
        "rm" => {
            if args.is_empty() {
                return Err(anyhow::anyhow!("rm 命令需要指定路径"));
            }
            let mut recursive = false;
            let mut force = false;
            let mut path = "";

            for arg in args {
                match *arg {
                    "-r" | "--recursive" => recursive = true,
                    "-f" | "--force" => force = true,
                    _ => path = *arg,
                }
            }

            if path.is_empty() {
                return Err(anyhow::anyhow!("rm 命令需要指定路径"));
            }

            Ok(Commands::Rm {
                path: state.resolve_path(path),
                recursive,
                force,
            })
        }
        "create" | "add" => {
            if args.is_empty() {
                return Err(anyhow::anyhow!("create 命令需要指定路径"));
            }
            let mut path = "";
            let mut data = None;
            let mut file = None;
            let mut node_type = "persistent";

            let mut i = 0;
            while i < args.len() {
                match args[i] {
                    "-d" | "--data" => {
                        if i + 1 < args.len() {
                            data = Some(args[i + 1].to_string());
                            i += 2;
                        } else {
                            return Err(anyhow::anyhow!("-d 需要参数"));
                        }
                    }
                    "-f" | "--file" => {
                        if i + 1 < args.len() {
                            file = Some(args[i + 1].to_string());
                            i += 2;
                        } else {
                            return Err(anyhow::anyhow!("-f 需要参数"));
                        }
                    }
                    "-t" | "--node-type" => {
                        if i + 1 < args.len() {
                            node_type = args[i + 1];
                            i += 2;
                        } else {
                            return Err(anyhow::anyhow!("-t 需要参数"));
                        }
                    }
                    _ => {
                        if path.is_empty() {
                            path = args[i];
                        }
                        i += 1;
                    }
                }
            }

            if path.is_empty() {
                return Err(anyhow::anyhow!("create 命令需要指定路径"));
            }

            Ok(Commands::Create {
                path: state.resolve_path(path),
                data,
                file,
                node_type: node_type.to_string(),
            })
        }
        "set" => {
            if args.is_empty() {
                return Err(anyhow::anyhow!("set 命令需要指定路径"));
            }
            let mut path = "";
            let mut data = None;
            let mut file = None;

            let mut i = 0;
            while i < args.len() {
                match args[i] {
                    "-d" | "--data" => {
                        if i + 1 < args.len() {
                            data = Some(args[i + 1].to_string());
                            i += 2;
                        } else {
                            return Err(anyhow::anyhow!("-d 需要参数"));
                        }
                    }
                    "-f" | "--file" => {
                        if i + 1 < args.len() {
                            file = Some(args[i + 1].to_string());
                            i += 2;
                        } else {
                            return Err(anyhow::anyhow!("-f 需要参数"));
                        }
                    }
                    _ => {
                        if path.is_empty() {
                            path = args[i];
                        }
                        i += 1;
                    }
                }
            }

            if path.is_empty() {
                return Err(anyhow::anyhow!("set 命令需要指定路径"));
            }

            Ok(Commands::Set {
                path: state.resolve_path(path),
                data,
                file,
            })
        }
        "cd" => {
            let path = args.first().map_or("/".to_string(), |s| (*s).to_string());
            Ok(Commands::Cd { path })
        }
        "pwd" => Ok(Commands::Pwd),
        "q" | "quit" | "exit" => Ok(Commands::Quit),
        "h" | "help" | "?" => Ok(Commands::Help),
        _ => Err(anyhow::anyhow!("未知命令：{cmd}. 输入 'help' 查看可用命令")),
    }
}

/// 执行命令
async fn execute_command(
    client: &Client,
    command: &Commands,
    state: &mut InteractiveState,
) -> Result<()> {
    match command {
        Commands::Ls { path } => {
            ls_command(client, path).await?;
        }
        Commands::Dir { path } => {
            dir_command(client, path).await?;
        }
        Commands::Cat { path } => {
            cat_command(client, path).await?;
        }
        Commands::Stat { path } => {
            stat_command(client, path).await?;
        }
        Commands::Rm {
            path,
            recursive,
            force,
        } => {
            rm_command(client, path, *recursive, *force).await?;
        }
        Commands::Create {
            path,
            data,
            file,
            node_type,
        } => {
            create_command(client, path, data.as_deref(), file.as_deref(), node_type).await?;
        }
        Commands::Set { path, data, file } => {
            set_command(client, path, data.as_deref(), file.as_deref()).await?;
        }
        Commands::Cd { path } => {
            let resolved = state.resolve_path(path);
            // 验证路径是否存在
            match client.check_stat(&resolved).await {
                Ok(Some(_)) => {
                    state.current_path = resolved;
                    println!("✓ 已切换到：{}", state.current_path);
                }
                Ok(None) => {
                    return Err(anyhow::anyhow!("路径不存在：{resolved}"));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("无法访问路径：{resolved} ({e})"));
                }
            }
        }
        Commands::Pwd => {
            println!("{}", state.current_path);
        }
        Commands::Quit => {
            println!("再见！👋");
        }
        Commands::Help => {
            print_help(&state.current_path);
        }
    }

    Ok(())
}

/// 打印帮助信息
fn print_help(current_path: &str) {
    println!();
    println!("可用命令:");
    println!("  ls [路径]              列出子节点 (默认：当前路径)");
    println!("  dir [路径]             列出子节点及详细信息 (默认：当前路径)");
    println!("  cat <路径>             显示节点数据");
    println!("  stat <路径>            显示节点状态");
    println!("  rm [-r] [-f] <路径>    删除节点 (-r 递归，-f 强制)");
    println!("  create <路径> [选项]   创建节点");
    println!("    -d, --data <数据>     节点数据");
    println!("    -f, --file <文件>     从文件读取数据");
    println!("    -t, --node-type <类型>  节点类型 (persistent/ephemeral/persistent-sequential/ephemeral-sequential)");
    println!("  set <路径> [选项]      设置节点数据");
    println!("    -d, --data <数据>     节点数据");
    println!("    -f, --file <文件>     从文件读取数据");
    println!("  cd [路径]              切换当前路径 (默认：/, 支持 . 和 ..)");
    println!("  pwd                    显示当前路径");
    println!("  help, h, ?             显示此帮助");
    println!("  quit, exit, q          退出");
    println!();
    println!("当前路径：{current_path}");
    println!("提示：路径支持相对路径和 Tab 自动补全");
    println!();
}

/// ls 命令：列出子节点
async fn ls_command(session: &zookeeper_client::Client, path: &str) -> Result<()> {
    let children = session
        .list_children(path)
        .await
        .context(format!("无法列出路径：{path}"))?;

    for child in children {
        println!("{child}");
    }

    Ok(())
}

/// dir 命令：列出子节点及详细信息
async fn dir_command(session: &zookeeper_client::Client, path: &str) -> Result<()> {
    let children = session
        .list_children(path)
        .await
        .context(format!("无法列出路径：{path}"))?;

    // 获取当前路径的状态
    let stat = session
        .check_stat(path)
        .await
        .context(format!("无法获取节点状态：{path}"))?
        .context(format!("节点不存在：{path}"))?;

    println!("路径：{path}");
    println!("子节点数量：{}", stat.num_children);
    println!("数据版本：{}", stat.version);
    println!("创建时间：{}", stat.ctime);
    println!("修改时间：{}", stat.mtime);
    println!("\n子节点列表:");

    for child in children {
        let child_path = if path == "/" {
            format!("/{child}")
        } else {
            format!("{path}/{child}")
        };

        // 获取每个子节点的状态
        match session.check_stat(&child_path).await {
            Ok(Some(child_stat)) => {
                println!(
                    "  {} (版本：{}, 子节点：{}, 数据大小：{} bytes)",
                    child, child_stat.version, child_stat.num_children, child_stat.data_length
                );
            }
            Ok(None) => {
                println!("  {child} (节点不存在)");
            }
            Err(_) => {
                println!("  {child} (无法获取状态)");
            }
        }
    }

    Ok(())
}

/// cat 命令：显示节点数据
async fn cat_command(session: &zookeeper_client::Client, path: &str) -> Result<()> {
    let (data, _stat) = session
        .get_data(path)
        .await
        .context(format!("无法获取节点数据：{path}"))?;

    // 尝试将数据转换为 UTF-8 字符串
    match String::from_utf8(data.clone()) {
        Ok(text) => {
            println!("{text}");
        }
        Err(_) => {
            // 如果不是有效的 UTF-8，显示十六进制
            println!("(二进制数据，十六进制表示)");
            for chunk in data.chunks(16) {
                for byte in chunk {
                    print!("{byte:02x} ");
                }
                println!();
            }
        }
    }

    Ok(())
}

/// stat 命令：显示节点状态
async fn stat_command(session: &zookeeper_client::Client, path: &str) -> Result<()> {
    let stat = session
        .check_stat(path)
        .await
        .context(format!("无法获取节点状态：{path}"))?
        .context(format!("节点不存在：{path}"))?;

    print_stat(path, &stat);

    Ok(())
}

/// create 命令：创建节点
async fn create_command(
    session: &zookeeper_client::Client,
    path: &str,
    data: Option<&str>,
    file: Option<&str>,
    node_type: &str,
) -> Result<()> {
    // 获取数据
    let data_bytes = get_data_from_input(data, file).await?;

    // 解析节点类型
    let create_mode = parse_create_mode(node_type)?;

    // 创建节点选项（使用默认的 anyone_all ACL）
    let options = create_mode.with_acls(Acls::anyone_all());

    // 创建节点
    let (_stat, seq) = session
        .create(path, &data_bytes, &options)
        .await
        .context(format!("无法创建节点：{path}"))?;

    // 构造实际路径（如果是顺序节点，需要添加序列号）
    let seq_str = format!("{seq}");
    let actual_path = if seq_str.is_empty() || seq_str == "0" {
        path.to_string()
    } else {
        format!("{path}{seq}")
    };

    println!("✓ 成功创建节点：{actual_path}");
    if !data_bytes.is_empty() {
        println!("  写入数据：{} bytes", data_bytes.len());
    }

    Ok(())
}

/// set 命令：设置节点数据
async fn set_command(
    session: &zookeeper_client::Client,
    path: &str,
    data: Option<&str>,
    file: Option<&str>,
) -> Result<()> {
    // 获取数据
    let data_bytes = get_data_from_input(data, file).await?;

    // 设置节点数据（None 表示不检查版本）
    session
        .set_data(path, &data_bytes, None)
        .await
        .context(format!("无法设置节点数据：{path}"))?;

    println!("✓ 成功设置节点数据：{path}");
    println!("  写入数据：{} bytes", data_bytes.len());

    Ok(())
}

/// 从参数或文件获取数据
async fn get_data_from_input(data: Option<&str>, file: Option<&str>) -> Result<Vec<u8>> {
    match (data, file) {
        (Some(d), None) => {
            // 从参数获取数据
            Ok(d.as_bytes().to_vec())
        }
        (None, Some(f)) => {
            // 从文件读取数据
            tokio::fs::read(f)
                .await
                .context(format!("无法读取文件：{f}"))
        }
        (Some(_), Some(_)) => Err(anyhow::anyhow!("不能同时指定 --data 和 --file 参数")),
        (None, None) => {
            // 都不指定则返回空数据
            Ok(Vec::new())
        }
    }
}

/// 解析节点类型
fn parse_create_mode(node_type: &str) -> Result<CreateMode> {
    match node_type.to_lowercase().as_str() {
        "persistent" | "p" => Ok(CreateMode::Persistent),
        "ephemeral" | "e" => Ok(CreateMode::Ephemeral),
        "persistent-sequential" | "ps" => Ok(CreateMode::PersistentSequential),
        "ephemeral-sequential" | "es" => Ok(CreateMode::EphemeralSequential),
        _ => Err(anyhow::anyhow!(
            "无效的节点类型：{node_type}. 支持的类型：persistent(p), ephemeral(e), persistent-sequential(ps), ephemeral-sequential(es)"
        )),
    }
}

/// rm 命令：删除节点
async fn rm_command(
    session: &zookeeper_client::Client,
    path: &str,
    recursive: bool,
    force: bool,
) -> Result<()> {
    // 如果需要递归删除，先删除所有子节点
    if recursive {
        match delete_recursive(session, path, force).await {
            Ok(_) => {
                println!("✓ 成功删除：{path}");
                Ok(())
            }
            Err(e) if force => {
                println!("⚠ 警告：删除 {path} 时出错：{e}");
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        // 非递归删除
        match session.delete(path, None).await {
            Ok(_) => {
                println!("✓ 成功删除：{path}");
                Ok(())
            }
            Err(e) if force => {
                println!("⚠ 警告：删除 {path} 时出错：{e}");
                Ok(())
            }
            Err(e) => Err(anyhow::Error::from(e).context(format!("无法删除节点：{path}"))),
        }
    }
}

/// 递归删除节点及其所有子节点
fn delete_recursive<'a>(
    session: &'a zookeeper_client::Client,
    path: &'a str,
    force: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        // 先获取所有子节点
        let children = match session.list_children(path).await {
            Ok(children) => children,
            Err(e) if force => {
                println!("⚠ 警告：无法列出 {path} 的子节点：{e}");
                return Ok(());
            }
            Err(e) => {
                return Err(anyhow::Error::from(e).context(format!("无法列出路径：{path}")));
            }
        };

        // 递归删除每个子节点
        for child in children {
            let child_path = if path == "/" {
                format!("/{child}")
            } else {
                format!("{path}/{child}")
            };

            if let Err(e) = delete_recursive(session, &child_path, force).await {
                if force {
                    println!("⚠ 警告：删除 {child_path} 时出错：{e}");
                } else {
                    return Err(e);
                }
            }
        }

        // 删除当前节点
        match session.delete(path, None).await {
            Ok(_) => Ok(()),
            Err(e) if force => {
                println!("⚠ 警告：删除 {path} 时出错：{e}");
                Ok(())
            }
            Err(e) => Err(anyhow::Error::from(e).context(format!("无法删除节点：{path}"))),
        }
    })
}

/// 打印节点状态信息
fn print_stat(path: &str, stat: &Stat) {
    println!("节点路径：{path}");
    println!("创建事务 ID: {}", stat.czxid);
    println!("修改事务 ID: {}", stat.mzxid);
    println!("创建时间：{}", stat.ctime);
    println!("修改时间：{}", stat.mtime);
    println!("数据版本：{}", stat.version);
    println!("子节点版本：{}", stat.cversion);
    println!("ACL 版本：{}", stat.aversion);
    println!("临时节点所有者：{}", stat.ephemeral_owner);
    println!("数据长度：{} bytes", stat.data_length);
    println!("子节点数量：{}", stat.num_children);
    println!("子节点修改事务 ID: {}", stat.pzxid);
}
