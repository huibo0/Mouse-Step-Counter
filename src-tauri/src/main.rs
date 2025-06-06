#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 项目结构：
// - src-tauri/src/main.rs         ← Rust后端，负责监听鼠标
// - src/App.tsx                  ← React前端，显示步数
// - tauri.conf.json              ← Tauri配置文件

// Rust 后端 (src-tauri/src/main.rs)

use tauri::{Manager, State, WindowEvent};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use enigo::{Enigo, Mouse, Settings};

#[derive(Default)]
struct StepCounter {
    total_distance: f64,
    steps: u32,
    last_x: i32,
    last_y: i32,
    initialized: bool,
    permission_error: bool,
    is_minimized: bool,
}

type CounterState = Arc<Mutex<StepCounter>>;

#[tauri::command]
fn reset_counter(counter: State<CounterState>) -> Result<(), String> {
    match counter.lock() {
        Ok(mut c) => {
            c.total_distance = 0.0;
            c.steps = 0;
            c.initialized = false;
            println!("🔄 计数器已重置");
            Ok(())
        },
        Err(_) => Err("无法重置计数器".to_string())
    }
}

#[tauri::command]
fn get_current_steps(counter: State<CounterState>) -> Result<u32, String> {
    match counter.lock() {
        Ok(c) => Ok(c.steps),
        Err(_) => Err("无法获取步数".to_string())
    }
}

#[tauri::command]
fn switch_to_main_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🔄 切换到主窗口...");
    
    // 隐藏宠物窗口
    if let Some(pet_window) = app_handle.get_window("pet") {
        if let Err(e) = pet_window.hide() {
            println!("⚠️ 隐藏宠物窗口失败: {:?}", e);
        } else {
            println!("👻 宠物窗口已隐藏");
        }
    }
    
    // 显示主窗口
    match app_handle.get_window("main") {
        Some(window) => {
            println!("✅ 找到主窗口，尝试显示...");
            if let Err(e) = window.show() {
                println!("❌ 显示窗口失败: {:?}", e);
                return Err(format!("显示窗口失败: {:?}", e));
            }
            if let Err(e) = window.set_focus() {
                println!("⚠️ 设置焦点失败: {:?}", e);
            }
            if let Err(e) = window.center() {
                println!("⚠️ 居中失败: {:?}", e);
            }
            println!("🎉 主窗口已显示！");
            Ok(())
        },
        None => {
            println!("❌ 未找到主窗口");
            Err("未找到主窗口".to_string())
        }
    }
}

#[tauri::command]
fn switch_to_pet_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🔄 切换到宠物窗口...");
    
    // 隐藏主窗口
    if let Some(main_window) = app_handle.get_window("main") {
        if let Err(e) = main_window.hide() {
            println!("⚠️ 隐藏主窗口失败: {:?}", e);
        } else {
            println!("👻 主窗口已隐藏");
        }
    }
    
    // 显示宠物窗口
    match app_handle.get_window("pet") {
        Some(window) => {
            println!("✅ 找到宠物窗口，尝试显示...");
            if let Err(e) = window.show() {
                println!("❌ 显示宠物窗口失败: {:?}", e);
                return Err(format!("显示宠物窗口失败: {:?}", e));
            }
            println!("🐕 宠物窗口已显示！");
            Ok(())
        },
        None => {
            println!("❌ 未找到宠物窗口");
            Err("未找到宠物窗口".to_string())
        }
    }
}

#[tauri::command]
fn quit_app(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("👋 用户请求退出程序");
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
fn open_devtools(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("🐛 收到打开开发者工具请求");
    
    // 尝试打开主窗口的开发者工具
    if let Some(main_window) = app_handle.get_window("main") {
        println!("🎯 找到主窗口，正在打开开发者工具...");
        main_window.open_devtools();
        println!("✅ 主窗口开发者工具已打开（独立窗口）");
        return Ok(());
    }
    
    // 如果主窗口不存在，尝试宠物窗口
    if let Some(pet_window) = app_handle.get_window("pet") {
        println!("🎯 找到宠物窗口，正在打开开发者工具...");
        pet_window.open_devtools();
        println!("✅ 宠物窗口开发者工具已打开（独立窗口）");
        return Ok(());
    }
    
    println!("❌ 找不到任何可用窗口");
    Err("找不到任何窗口".to_string())
}

#[tauri::command]
fn show_context_menu(app_handle: tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    println!("🎯 收到创建右键菜单请求，位置: ({}, {})", x, y);
    println!("🔧 开始处理右键菜单命令...");
    
    // 先关闭现有的菜单窗口（如果存在）
    if let Some(existing_menu) = app_handle.get_window("context_menu") {
        println!("🗑️ 发现已存在的右键菜单窗口，正在关闭...");
        if let Err(e) = existing_menu.close() {
            println!("⚠️ 关闭已存在窗口时出错: {}", e);
        } else {
            println!("✅ 已存在的右键菜单窗口已关闭");
        }
        // 等待一小段时间确保窗口完全关闭
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    let menu_html = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            margin: 0;
            padding: 0;
            background: rgba(255, 255, 255, 0.98);
            border: 2px solid rgba(0, 0, 0, 0.3);
            border-radius: 8px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
            overflow: hidden;
        }
        .menu-item {
            padding: 12px 16px;
            cursor: pointer;
            color: #000;
            font-size: 14px;
            font-weight: 600;
            white-space: nowrap;
            border-bottom: 1px solid rgba(0,0,0,0.1);
            transition: background-color 0.2s;
        }
        .menu-item:last-child {
            border-bottom: none;
        }
        .menu-item:hover {
            background-color: rgba(0, 120, 255, 0.1);
            color: #007AFF;
        }
        .menu-item.danger:hover {
            background-color: rgba(255, 68, 68, 0.1);
            color: #ff4444;
        }
    </style>
</head>
<body>
    <div class="menu-item" onclick="openDevTools()">🐛 调试 (Cmd+D)</div>
    <div class="menu-item danger" onclick="quitApp()">❌ 退出</div>
    
    <script>
        async function openDevTools() {
            try {
                const currentWindow = window.__TAURI__.window.getCurrent();
                await currentWindow.close();
                
                // 调用后端的开发者工具命令
                await window.__TAURI__.invoke('open_devtools');
            } catch (error) {
                console.error('打开开发者工具失败:', error);
            }
        }
        
        async function quitApp() {
            try {
                await window.__TAURI__.invoke('quit_app');
            } catch (error) {
                console.error('退出应用失败:', error);
            }
        }
        
        // 点击窗口外部时关闭菜单
        document.addEventListener('click', (e) => {
            if (e.target === document.body) {
                window.__TAURI__.window.getCurrent().close();
            }
        });
        
        // 监听窗口失去焦点事件，自动关闭菜单
        window.addEventListener('blur', () => {
            console.log('🔄 右键菜单失去焦点，自动关闭');
            setTimeout(() => {
                window.__TAURI__.window.getCurrent().close();
            }, 200);
        });
        
        // 监听键盘事件，ESC键关闭菜单
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                window.__TAURI__.window.getCurrent().close();
            }
        });
    </script>
</body>
</html>
    "#;
    
    use tauri::WindowBuilder;
    
    match WindowBuilder::new(
        &app_handle,
        "context_menu",
        tauri::WindowUrl::App("".into())
    )
    .title("Context Menu")
    .inner_size(130.0, 80.0)
    .position(x as f64, y as f64)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(true)
    .transparent(true)
    .build() {
        Ok(window) => {
            // 设置HTML内容
            let escaped_html = menu_html
                .replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace('\n', "\\n")
                .replace('\r', "");
                
            let script = format!("document.documentElement.innerHTML = `{}`;", escaped_html);
            
            match window.eval(&script) {
                Ok(_) => {
                    println!("✅ 右键菜单窗口创建成功，HTML内容已设置");
                },
                Err(e) => {
                    println!("⚠️ 设置HTML内容失败: {:?}", e);
                }
            }
            
            println!("✅ 右键菜单窗口创建成功");
            Ok(())
        },
        Err(e) => {
            println!("❌ 创建右键菜单窗口失败: {:?}", e);
            Err(format!("创建菜单窗口失败: {:?}", e))
        }
    }
}

#[tauri::command]
fn hide_context_menu(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(menu_window) = app_handle.get_window("context_menu") {
        let _ = menu_window.close();
        println!("🚫 右键菜单窗口已关闭");
    }
    Ok(())
}

fn main() {
    let counter = Arc::new(Mutex::new(StepCounter::default()));
    
    tauri::Builder::default()
        .manage(counter.clone())
        .invoke_handler(tauri::generate_handler![reset_counter, get_current_steps, switch_to_main_window, switch_to_pet_window, quit_app, open_devtools, show_context_menu, hide_context_menu])
        .setup(|app| {
            let app_handle = app.handle();
            let window = app.get_window("main").unwrap();

            // 监听窗口事件
            let counter_for_window = counter.clone();
            let app_handle_for_window = app_handle.clone();
            window.on_window_event(move |event| {
                match event {
                    WindowEvent::Focused(focused) => {
                        if let Ok(mut c) = counter_for_window.lock() {
                            println!("🪟 窗口焦点状态: {}", if *focused { "获得焦点" } else { "失去焦点" });
                        }
                    },
                    WindowEvent::Resized(_) => {
                        if let Ok(mut c) = counter_for_window.lock() {
                            c.is_minimized = false;
                            println!("🪟 窗口已调整大小");
                        }
                    },
                    WindowEvent::CloseRequested { api, .. } => {
                        println!("🚪 用户点击关闭按钮，切换到宠物窗口");
                        api.prevent_close();
                        if let Err(e) = switch_to_pet_window(app_handle_for_window.clone()) {
                            println!("❌ 切换到宠物窗口失败: {}", e);
                        }
                    },
                    _ => {}
                }
            });

            let counter_clone = counter.clone();
            thread::spawn(move || {
                println!("🖱️ 开始监听鼠标移动...");
                match Enigo::new(&Settings::default()) {
                    Ok(mut enigo) => {
                        loop {
                            match enigo.location() {
                                Ok((x, y)) => {
                                    let mut c = counter_clone.lock().unwrap();
                                    if c.permission_error {
                                        c.permission_error = false;
                                        println!("✅ 成功获取鼠标位置权限！");
                                    }
                                    
                                    if !c.initialized {
                                        c.last_x = x;
                                        c.last_y = y;
                                        c.initialized = true;
                                        println!("🎯 鼠标监听已初始化");
                                    } else {
                                        let dx = (x - c.last_x) as f64;
                                        let dy = (y - c.last_y) as f64;
                                        let distance = (dx.powi(2) + dy.powi(2)).sqrt();
                                        
                                        if distance > 0.0 {
                                            c.total_distance += distance;
                                            let new_steps = (c.total_distance / 100.0) as u32;
                                            if new_steps != c.steps {
                                                c.steps = new_steps;
                                                // 即使窗口最小化也打印日志
                                                if new_steps % 10 == 0 {
                                                    println!("📈 步数更新: {} (距离: {:.1}px)", c.steps, c.total_distance);
                                                }
                                            }
                                            c.last_x = x;
                                            c.last_y = y;
                                        }
                                    }
                                },
                                Err(e) => {
                                    let mut c = counter_clone.lock().unwrap();
                                    if !c.permission_error {
                                        c.permission_error = true;
                                        println!("❌ 需要辅助功能权限才能读取鼠标位置: {:?}", e);
                                        println!("📋 解决方法：");
                                        println!("   1. 打开 系统设置 > 隐私与安全性 > 辅助功能");
                                        println!("   2. 添加并启用这个应用或终端");
                                        
                                        // 等待更长时间再重试
                                        thread::sleep(Duration::from_secs(5));
                                        continue;
                                    }
                                }
                            }
                            thread::sleep(Duration::from_millis(50)); // 更频繁的检查
                        }
                    },
                    Err(e) => {
                        println!("❌ 无法初始化鼠标监听: {:?}", e);
                    }
                }
            });

            // 每秒发送一次当前步数给前端
            thread::spawn(move || {
                loop {
                    let steps = {
                        let c = counter.lock().unwrap();
                        c.steps
                    };
                    let _ = app_handle.emit_all("step_update", steps);
                    thread::sleep(Duration::from_secs(1));
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
