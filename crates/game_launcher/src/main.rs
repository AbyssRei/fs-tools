use std::io::{self, Write};
use std::process::Command;

fn main() {
    println!("=== 启动器配置 ===");

    // 第一次询问：是否使用局域网联机
    if get_user_confirmation("是否使用局域网联机？") {
        println!("正在启动 steamclient_loader.exe ...");
        run_program("steamclient_loader.exe", &[]);
    } else {
        // 第二次询问：是否加载 OnlineFix 补丁
        if get_user_confirmation("是否加载OnlineFix补丁？") {
            println!("正在启动 me3_launcher.exe --with-onlinefix ...");
            run_program("me3_launcher.exe", &["--with-onlinefix"]);
        } else {
            println!("正在启动 me3_launcher.exe ...");
            run_program("me3_launcher.exe", &[]);
        }
    }
}

/// 获取用户确认（y/n）
/// 循环询问直到用户输入有效的 "y" 或 "n"
fn get_user_confirmation(prompt: &str) -> bool {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("{} (y/n): ", prompt);
        // 强制刷新输出缓冲区，确保提示语立即显示
        io::stdout().flush().unwrap();

        input.clear();
        // 读取用户输入
        stdin.read_line(&mut input).expect("无法读取输入");

        match input.trim() {
            "y" | "Y" => return true,
            "n" | "N" => return false,
            _ => println!("输入无效，请输入 'y' 或 'n'。"),
        }
    }
}

/// 运行指定的程序
fn run_program(program: &str, args: &[&str]) {
    match Command::new(program).args(args).spawn() {
        Ok(_child) => {
            println!("程序已成功启动。");
        }
        Err(e) => {
            eprintln!("启动 {} 失败: {}", program, e);
            eprintln!("请确保 {} 文件位于当前目录或系统 PATH 中。", program);
            // 防止窗口一闪而过，等待用户按键后退出（仅在出错时）
            println!("按回车键退出...");
            let _ = io::stdin().read_line(&mut String::new());
        }
    }
}
