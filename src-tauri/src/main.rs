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

fn main() {
    let counter = Arc::new(Mutex::new(StepCounter::default()));
    
    tauri::Builder::default()
        .manage(counter.clone())
        .invoke_handler(tauri::generate_handler![reset_counter, get_current_steps])
        .setup(|app| {
            let app_handle = app.handle();
            let window = app.get_window("main").unwrap();

            // 监听窗口事件
            let counter_for_window = counter.clone();
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
