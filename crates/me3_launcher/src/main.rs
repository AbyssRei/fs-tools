use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// ME3 模组加载器启动器
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 加载 OnlineFix 补丁 (生成临时配置文件启动)
    #[arg(long)]
    with_onlinefix: bool,
}

/// 配置文件结构
#[derive(Deserialize)]
struct Config {
    me3_path: String,
    mod_path: String,
    game_exe: String,
    game: String,
    extra_args: Option<Vec<String>>,
}

fn main() -> Result<()> {
    // 1. 设置 Windows 控制台代码页为 UTF-8
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
        let _ = SetConsoleCP(65001);
        let _ = SetConsoleOutputCP(65001);
    }

    // 2. 解析命令行参数
    let args = Args::parse();

    // 3. 读取配置文件
    let exe_path = env::current_exe().context("无法获取当前程序路径")?;
    let exe_dir = exe_path.parent().context("无法获取程序所在目录")?;
    let config_path = exe_dir.join("config.toml");

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("读取配置文件失败: {:?}", config_path))?;
    let config: Config = toml::from_str(&config_content).context("解析配置文件失败")?;

    // 4. 根据参数决定是否生成临时补丁文件
    let original_mod_path = PathBuf::from(&config.mod_path);
    let mut final_mod_path = original_mod_path.clone();
    let mut temp_file_to_clean: Option<PathBuf> = None;

    if args.with_onlinefix {
        println!("正在生成临时补丁配置 (OnlineFix)...");

        // 构造临时文件路径
        let temp_path = original_mod_path.with_extension("temp.me3");

        // 定义要追加的 TOML 内容
        let patch_content = r#"
[[natives]]
id = "OnlineFix"
path = "OnlineFix/OnlineFix64.dll"
load_early = true
"#;

        // 读取原始内容并合并
        let original_content = fs::read_to_string(&original_mod_path)
            .with_context(|| format!("无法读取原始模组文件: {:?}", original_mod_path))?;
        let new_content = format!("{}\n{}", original_content, patch_content);

        // 写入临时文件
        fs::write(&temp_path, new_content)
            .with_context(|| format!("无法写入临时文件: {:?}", temp_path))?;

        println!("临时文件已生成: {:?}", temp_path);

        // 更新路径并标记清理
        final_mod_path = temp_path.clone();
        temp_file_to_clean = Some(temp_path);
    }

    // 5. 构建启动参数
    let mut args_list = vec![
        "launch".to_string(),
        "-p".to_string(),
        final_mod_path.to_string_lossy().to_string(),
        "--game".to_string(),
        config.game,
    ];

    if let Some(extra) = &config.extra_args {
        args_list.extend(extra.clone());
    }

    args_list.push("--exe".to_string());
    args_list.push(config.game_exe);

    println!("--------------------------------");
    println!("启动程序: {}", config.me3_path);
    if args.with_onlinefix {
        println!("模式: 加载 OnlineFix");
    } else {
        println!("模式: 标准启动");
    }
    println!("--------------------------------");

    // 6. 启动进程
    let status = Command::new(&config.me3_path)
        .args(&args_list)
        .current_dir(exe_dir)
        .status()
        .with_context(|| format!("无法启动程序: {}", config.me3_path))?;

    // 7. 清理临时文件
    if let Some(temp_path) = temp_file_to_clean {
        print!("正在清理临时文件... ");
        if let Err(e) = fs::remove_file(&temp_path) {
            eprintln!("失败: {}", e);
        } else {
            println!("完成");
        }
    }

    if !status.success() {
        eprintln!("程序异常退出，退出码: {:?}", status.code());
    }

    Ok(())
}
