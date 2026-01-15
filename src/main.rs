use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use zookeeper_client::{Acls, Client, CreateMode, Stat};

/// Zookeeper文件系统命令行工具
#[derive(Parser, Debug)]
#[command(name = "zkfs")]
#[command(about = "Zookeeper命令行工具", long_about = None)]
struct Cli {
    /// Zookeeper服务器地址 (例如: localhost:2181)
    #[arg(short, long, env = "ZK_SERVER", default_value = "localhost:2181")]
    server: String,

    /// 连接超时时间（秒）
    #[arg(short, long, env = "ZK_TIMEOUT", default_value = "10")]
    timeout: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 列出指定路径下的子节点（类似ls命令）
    Ls {
        /// Zookeeper路径
        #[arg(default_value = "/")]
        path: String,
    },
    /// 列出指定路径下的子节点及详细信息（类似dir命令）
    Dir {
        /// Zookeeper路径
        #[arg(default_value = "/")]
        path: String,
    },
    /// 显示节点数据内容（类似cat命令）
    Cat {
        /// Zookeeper路径
        path: String,
    },
    /// 显示节点状态信息（类似stat命令）
    Stat {
        /// Zookeeper路径
        path: String,
    },
    /// 删除节点（类似rm命令）
    Rm {
        /// Zookeeper路径
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
        /// Zookeeper路径
        path: String,
        /// 节点数据（如果不指定则从--file读取，都不指定则为空数据）
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
        /// Zookeeper路径
        path: String,
        /// 节点数据（如果不指定则从--file读取）
        #[arg(short, long)]
        data: Option<String>,
        /// 从文件读取数据
        #[arg(short, long)]
        file: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 创建Zookeeper客户端
    let client = Client::connector()
        .connect(&cli.server)
        .await
        .context("无法连接到Zookeeper服务器")?;

    // 执行命令
    match cli.command {
        Commands::Ls { path } => {
            ls_command(&client, &path).await?;
        }
        Commands::Dir { path } => {
            dir_command(&client, &path).await?;
        }
        Commands::Cat { path } => {
            cat_command(&client, &path).await?;
        }
        Commands::Stat { path } => {
            stat_command(&client, &path).await?;
        }
        Commands::Rm { path, recursive, force } => {
            rm_command(&client, &path, recursive, force).await?;
        }
        Commands::Create { path, data, file, node_type } => {
            create_command(&client, &path, data.as_deref(), file.as_deref(), &node_type).await?;
        }
        Commands::Set { path, data, file } => {
            set_command(&client, &path, data.as_deref(), file.as_deref()).await?;
        }
    }

    Ok(())
}

/// ls命令：列出子节点
async fn ls_command(
    session: &zookeeper_client::Client,
    path: &str,
) -> Result<()> {
    let children = session
        .list_children(path)
        .await
        .context(format!("无法列出路径: {}", path))?;

    for child in children {
        println!("{}", child);
    }

    Ok(())
}

/// dir命令：列出子节点及详细信息
async fn dir_command(
    session: &zookeeper_client::Client,
    path: &str,
) -> Result<()> {
    let children = session
        .list_children(path)
        .await
        .context(format!("无法列出路径: {}", path))?;

    // 获取当前路径的状态
    let stat = session
        .check_stat(path)
        .await
        .context(format!("无法获取节点状态: {}", path))?
        .context(format!("节点不存在: {}", path))?;

    println!("路径: {}", path);
    println!("子节点数量: {}", stat.num_children);
    println!("数据版本: {}", stat.version);
    println!("创建时间: {}", stat.ctime);
    println!("修改时间: {}", stat.mtime);
    println!("\n子节点列表:");

    for child in children {
        let child_path = if path == "/" {
            format!("/{}", child)
        } else {
            format!("{}/{}", path, child)
        };

        // 获取每个子节点的状态
        match session.check_stat(&child_path).await {
            Ok(Some(child_stat)) => {
                println!(
                    "  {} (版本: {}, 子节点: {}, 数据大小: {} bytes)",
                    child,
                    child_stat.version,
                    child_stat.num_children,
                    child_stat.data_length
                );
            }
            Ok(None) => {
                println!("  {} (节点不存在)", child);
            }
            Err(_) => {
                println!("  {} (无法获取状态)", child);
            }
        }
    }

    Ok(())
}

/// cat命令：显示节点数据
async fn cat_command(
    session: &zookeeper_client::Client,
    path: &str,
) -> Result<()> {
    let (data, _stat) = session
        .get_data(path)
        .await
        .context(format!("无法获取节点数据: {}", path))?;

    // 尝试将数据转换为UTF-8字符串
    match String::from_utf8(data.clone()) {
        Ok(text) => {
            println!("{}", text);
        }
        Err(_) => {
            // 如果不是有效的UTF-8，显示十六进制
            println!("(二进制数据，十六进制表示)");
            for chunk in data.chunks(16) {
                for byte in chunk {
                    print!("{:02x} ", byte);
                }
                println!();
            }
        }
    }

    Ok(())
}

/// stat命令：显示节点状态
async fn stat_command(
    session: &zookeeper_client::Client,
    path: &str,
) -> Result<()> {
    let stat = session
        .check_stat(path)
        .await
        .context(format!("无法获取节点状态: {}", path))?
        .context(format!("节点不存在: {}", path))?;

    print_stat(path, &stat);

    Ok(())
}

/// create命令：创建节点
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

    // 创建节点选项（使用默认的anyone_all ACL）
    let options = create_mode.with_acls(Acls::anyone_all());

    // 创建节点
    let (_stat, seq) = session
        .create(path, &data_bytes, &options)
        .await
        .context(format!("无法创建节点: {}", path))?;

    // 构造实际路径（如果是顺序节点，需要添加序列号）
    let seq_str = format!("{}", seq);
    let actual_path = if seq_str.is_empty() || seq_str == "0" {
        path.to_string()
    } else {
        format!("{}{}", path, seq)
    };

    println!("成功创建节点: {}", actual_path);
    if data_bytes.len() > 0 {
        println!("写入数据: {} bytes", data_bytes.len());
    }

    Ok(())
}

/// set命令：设置节点数据
async fn set_command(
    session: &zookeeper_client::Client,
    path: &str,
    data: Option<&str>,
    file: Option<&str>,
) -> Result<()> {
    // 获取数据
    let data_bytes = get_data_from_input(data, file).await?;

    // 设置节点数据（None表示不检查版本）
    session
        .set_data(path, &data_bytes, None)
        .await
        .context(format!("无法设置节点数据: {}", path))?;

    println!("成功设置节点数据: {}", path);
    println!("写入数据: {} bytes", data_bytes.len());

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
                .context(format!("无法读取文件: {}", f))
        }
        (Some(_), Some(_)) => {
            Err(anyhow::anyhow!("不能同时指定 --data 和 --file 参数"))
        }
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
            "无效的节点类型: {}. 支持的类型: persistent(p), ephemeral(e), persistent-sequential(ps), ephemeral-sequential(es)",
            node_type
        )),
    }
}

/// rm命令：删除节点
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
                println!("成功删除: {}", path);
                Ok(())
            }
            Err(e) if force => {
                println!("警告: 删除 {} 时出错: {}", path, e);
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        // 非递归删除
        match session.delete(path, None).await {
            Ok(_) => {
                println!("成功删除: {}", path);
                Ok(())
            }
            Err(e) if force => {
                println!("警告: 删除 {} 时出错: {}", path, e);
                Ok(())
            }
            Err(e) => Err(anyhow::Error::from(e)
                .context(format!("无法删除节点: {}", path))),
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
                println!("警告: 无法列出 {} 的子节点: {}", path, e);
                return Ok(());
            }
            Err(e) => {
                return Err(anyhow::Error::from(e)
                    .context(format!("无法列出路径: {}", path)));
            }
        };

        // 递归删除每个子节点
        for child in children {
            let child_path = if path == "/" {
                format!("/{}", child)
            } else {
                format!("{}/{}", path, child)
            };

            if let Err(e) = delete_recursive(session, &child_path, force).await {
                if force {
                    println!("警告: 删除 {} 时出错: {}", child_path, e);
                } else {
                    return Err(e);
                }
            }
        }

        // 删除当前节点
        match session.delete(path, None).await {
            Ok(_) => Ok(()),
            Err(e) if force => {
                println!("警告: 删除 {} 时出错: {}", path, e);
                Ok(())
            }
            Err(e) => Err(anyhow::Error::from(e)
                .context(format!("无法删除节点: {}", path))),
        }
    })
}

/// 打印节点状态信息
fn print_stat(path: &str, stat: &Stat) {
    println!("节点路径: {}", path);
    println!("创建事务ID: {}", stat.czxid);
    println!("修改事务ID: {}", stat.mzxid);
    println!("创建时间: {}", stat.ctime);
    println!("修改时间: {}", stat.mtime);
    println!("数据版本: {}", stat.version);
    println!("子节点版本: {}", stat.cversion);
    println!("ACL版本: {}", stat.aversion);
    println!("临时节点所有者: {}", stat.ephemeral_owner);
    println!("数据长度: {} bytes", stat.data_length);
    println!("子节点数量: {}", stat.num_children);
    println!("子节点修改事务ID: {}", stat.pzxid);
}
