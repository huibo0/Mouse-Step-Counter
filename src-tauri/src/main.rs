#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// 项目结构：
// - src-tauri/src/main.rs         ← Rust后端，负责监听鼠标
// - src/App.tsx                  ← React前端，显示步数
// - tauri.conf.json              ← Tauri配置文件

// Rust 后端 (src-tauri/src/main.rs)

use tauri::{Manager, State, WindowEvent};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use enigo::{Enigo, Mouse, Settings, Key, Keyboard};
use serde::{Deserialize, Serialize};
use chrono;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct StepCounter {
    total_distance: f64,
    steps: u32,
    last_x: i32,
    last_y: i32,
    initialized: bool,
    permission_error: bool,
    is_minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WaterReminderConfig {
    daily_glasses: u32,
    reminder_interval_hours: u32,
    custom_reminder_times: Vec<String>, // HH:MM format
    enabled: bool,
}

impl Default for WaterReminderConfig {
    fn default() -> Self {
        Self {
            daily_glasses: 8,
            reminder_interval_hours: 1,
            custom_reminder_times: vec![],
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WaterReminderState {
    config: WaterReminderConfig,
    last_reminder_time: u64,
    glasses_drunk_today: u32,
    last_reset_date: String,
    current_period_water_drunk: bool,  // 当前时间段是否已喝水
    last_water_period_start: u64,     // 上次喝水的时间段开始时间
    last_custom_reminder_triggered_at: Option<String>, // 跟踪上一个触发的自定义时间点 (HH:MM)
}

impl Default for WaterReminderState {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 计算当前时间段的开始时间
        let current_period_start = if true { // 暂时用true，后面会重新计算
            // 默认按1小时计算，后面会根据实际配置调整
            let hour_seconds = 3600;
            (now / hour_seconds) * hour_seconds
        } else {
            0
        };
        
        Self {
            config: WaterReminderConfig::default(),
            last_reminder_time: now, // 设置为当前时间，避免立即提醒
            glasses_drunk_today: 0,
            last_reset_date: Self::get_current_date(),
            current_period_water_drunk: true, // 第一次启动时认为已喝水
            last_water_period_start: current_period_start,
            last_custom_reminder_triggered_at: None,
        }
    }
}

impl WaterReminderState {
    fn get_current_date() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let date = chrono::DateTime::from_timestamp(now as i64, 0)
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();
        date
    }
    
    fn get_current_period_start(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if self.config.custom_reminder_times.is_empty() {
            // 使用间隔时间，计算当前时间段的开始时间
            let interval_seconds = self.config.reminder_interval_hours as u64 * 3600;
            (now / interval_seconds) * interval_seconds
        } else {
            // 使用自定义时间，按小时计算
            let hour_seconds = 3600;
            (now / hour_seconds) * hour_seconds
        }
    }
    
    fn should_reset_daily_count(&mut self) {
        let current_date = Self::get_current_date();
        if self.last_reset_date != current_date {
            self.glasses_drunk_today = 0;
            self.last_reset_date = current_date;
            self.current_period_water_drunk = false;
            self.last_water_period_start = 0;
            println!("🔄 新的一天开始，重置喝水计数");
        }
    }
    
    fn check_period_change(&mut self) {
        if !self.config.custom_reminder_times.is_empty() {
            // --- 自定义时间模式 ---
            let current_time_str = chrono::Local::now().format("%H:%M").to_string();
            // 只有当这个时间点尚未被触发过时，才进行处理
            if self.config.custom_reminder_times.contains(&current_time_str) && self.last_custom_reminder_triggered_at.as_deref() != Some(current_time_str.as_str()) {
                // 到达了提醒时间点，如果当前状态是"已喝水"，则切换为"未喝水"以触发提醒
                println!("⏰ [Custom Time] Matched: {}. Setting flag to NEEDS DRINKING.", current_time_str);
                self.current_period_water_drunk = false;
                // "锁定"这个时间点，防止在同一分钟内重复触发
                self.last_custom_reminder_triggered_at = Some(current_time_str);
            }
        } else {
            // --- 按小时间隔模式 ---
            let current_period_start = self.get_current_period_start();
            if current_period_start > self.last_water_period_start {
                // 进入了新的时间段
                println!("🌅 [PERIOD CHANGE] New period started. Old start: {}, New start: {}. Resetting water drunk status.", self.last_water_period_start, current_period_start);
                self.current_period_water_drunk = false; // 重置喝水状态
                self.last_water_period_start = current_period_start;
                // 在小时模式下，清除自定义时间的锁定
                self.last_custom_reminder_triggered_at = None;
            }
        }
    }
    
    fn record_water_drunk(&mut self) {
        println!("💧 [WATER DRUNK] User recorded drinking water. State before: {:?}", self);
        self.should_reset_daily_count();
        self.glasses_drunk_today += 1;
        self.current_period_water_drunk = true;
        self.last_water_period_start = self.get_current_period_start(); // 将喝水时间标记为当前时间段
        println!("💧 [WATER DRUNK] State after: {:?}", self);
    }
    
    fn needs_reminder(&mut self) -> bool {
        self.should_reset_daily_count();
        self.check_period_change();
        
        if !self.config.enabled {
            return false;
        }
        
        // 如果当前时间段已经喝过水，不需要提醒
        if self.current_period_water_drunk {
            return false;
        }

        // 防刷屏：如果60秒内已经提醒过，则不再提醒
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now < self.last_reminder_time.saturating_add(60) {
            // 在UI上可能仍然显示需要喝水，但是不再发送新的提醒事件
            return false; 
        }
        
        // 如果通过了以上所有检查，说明需要发送提醒
        true
    }

    fn update_config(&mut self, new_config: WaterReminderConfig) {
        self.config = new_config;

        // --- NEW ROBUST LOGIC ---
        // Whenever the config is changed, we must reset the reminder state.
        // This prevents a reminder for an old setting (e.g., from hourly mode)
        // from firing immediately after switching to a new setting (e.g., custom time mode).
        // We reset to a "safe" state, assuming the user has drunk water for the current period.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        println!("🔄 [CONFIG UPDATE] Config changed. Resetting reminder state to be safe...");
        self.current_period_water_drunk = true;
        self.last_water_period_start = self.get_current_period_start();
        self.last_reminder_time = now; // Also update this to be safe
        self.last_custom_reminder_triggered_at = None; // 关键：重置配置时必须清除锁定
    }
}

type CounterState = Arc<Mutex<StepCounter>>;
type WaterReminderStateType = Arc<Mutex<WaterReminderState>>;

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
        println!("🗑️ 右键菜单窗口已关闭");
    }
    Ok(())
}

// 喝水提醒相关命令
#[tauri::command]
fn get_water_reminder_config(water_state: State<WaterReminderStateType>) -> Result<WaterReminderConfig, String> {
    match water_state.lock() {
        Ok(state) => Ok(state.config.clone()),
        Err(_) => Err("无法获取喝水提醒配置".to_string())
    }
}

#[tauri::command]
fn update_water_reminder_config(
    water_state: State<WaterReminderStateType>,
    config: WaterReminderConfig
) -> Result<(), String> {
    println!("💧 [CONFIG UPDATE] Received new config: {:?}", config);
    match water_state.lock() {
        Ok(mut state) => {
            state.update_config(config);
            println!("💧 [CONFIG UPDATE] State updated. New state: {:?}", state);
            Ok(())
        },
        Err(_) => Err("无法更新喝水提醒配置".to_string())
    }
}

#[tauri::command]
fn get_water_reminder_state(water_state: State<WaterReminderStateType>) -> Result<WaterReminderState, String> {
    match water_state.lock() {
        Ok(mut state) => {
            state.should_reset_daily_count();
            Ok(state.clone())
        },
        Err(_) => Err("无法获取喝水提醒状态".to_string())
    }
}

#[tauri::command]
fn record_water_drunk(water_state: State<WaterReminderStateType>) -> Result<(), String> {
    match water_state.lock() {
        Ok(mut state) => {
            state.record_water_drunk();
            Ok(())
        },
        Err(_) => Err("无法记录喝水".to_string())
    }
}

#[tauri::command]
fn show_water_reminder(app_handle: tauri::AppHandle, message: String) -> Result<(), String> {
    println!("💧 显示喝水提醒: {}", message);
    
    let reminder_html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Arial, sans-serif;
            color: white;
            border-radius: 12px;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
            overflow: hidden;
            user-select: none;
        }}
        .reminder-container {{
            text-align: center;
            max-width: 300px;
        }}
        .water-icon {{
            font-size: 48px;
            margin-bottom: 16px;
        }}
        .message {{
            font-size: 16px;
            font-weight: 600;
            margin-bottom: 20px;
            line-height: 1.4;
        }}
        .buttons {{
            display: flex;
            gap: 12px;
            justify-content: center;
        }}
        .btn {{
            padding: 8px 16px;
            border: none;
            border-radius: 6px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }}
        .btn-primary {{
            background: rgba(255, 255, 255, 0.2);
            color: white;
            border: 1px solid rgba(255, 255, 255, 0.3);
        }}
        .btn-primary:hover {{
            background: rgba(255, 255, 255, 0.3);
        }}
        .btn-secondary {{
            background: rgba(255, 255, 255, 0.1);
            color: rgba(255, 255, 255, 0.8);
            border: 1px solid rgba(255, 255, 255, 0.2);
        }}
        .btn-secondary:hover {{
            background: rgba(255, 255, 255, 0.2);
        }}
    </style>
</head>
<body>
    <div class="reminder-container">
        <div class="water-icon">💧</div>
        <div class="message">{}</div>
        <div class="buttons">
            <button class="btn btn-primary" onclick="drinkWater()">我喝了水</button>
            <button class="btn btn-secondary" onclick="closeReminder()">稍后提醒</button>
        </div>
    </div>
    
    <script>
        async function drinkWater() {{
            try {{
                await window.__TAURI__.invoke('record_water_drunk');
                window.__TAURI__.window.getCurrent().close();
            }} catch (error) {{
                console.error('记录喝水失败:', error);
            }}
        }}
        
        async function closeReminder() {{
            window.__TAURI__.window.getCurrent().close();
        }}
        
        // 5秒后自动关闭
        setTimeout(() => {{
            window.__TAURI__.window.getCurrent().close();
        }}, 5000);
        
        // ESC键关闭
        document.addEventListener('keydown', (e) => {{
            if (e.key === 'Escape') {{
                window.__TAURI__.window.getCurrent().close();
            }}
        }});
    </script>
</body>
</html>
    "#, message);
    
    use tauri::WindowBuilder;
    
    match WindowBuilder::new(
        &app_handle,
        "water_reminder",
        tauri::WindowUrl::App("".into())
    )
    .title("喝水提醒")
    .inner_size(340.0, 200.0)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(true)
    .transparent(true)
    .center()
    .build() {
        Ok(window) => {
            let escaped_html = reminder_html
                .replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace('\n', "\\n")
                .replace('\r', "");
                
            let script = format!("document.documentElement.innerHTML = `{}`;", escaped_html);
            
            match window.eval(&script) {
                Ok(_) => {
                    println!("✅ 喝水提醒窗口创建成功");
                },
                Err(e) => {
                    println!("⚠️ 设置提醒HTML内容失败: {:?}", e);
                }
            }
            
            Ok(())
        },
        Err(e) => {
            println!("❌ 创建喝水提醒窗口失败: {:?}", e);
            Err(format!("创建提醒窗口失败: {:?}", e))
        }
    }
}

fn main() {
    let counter = Arc::new(Mutex::new(StepCounter::default()));
    let water_reminder_state = Arc::new(Mutex::new(WaterReminderState::default()));
    
    tauri::Builder::default()
        .manage(counter.clone())
        .manage(water_reminder_state.clone())
        .invoke_handler(tauri::generate_handler![reset_counter, get_current_steps, switch_to_main_window, switch_to_pet_window, quit_app, open_devtools, show_context_menu, hide_context_menu, get_water_reminder_config, update_water_reminder_config, get_water_reminder_state, record_water_drunk, show_water_reminder])
        .setup(move |app| {
            let app_handle = app.handle();
            let window = app.get_window("main").unwrap();

            // 监听窗口事件
            let counter_for_window = counter.clone();
            let app_handle_for_window = app_handle.clone();
            window.on_window_event(move |event| {
                match event {
                    WindowEvent::Focused(focused) => {
                        if let Ok(c) = counter_for_window.lock() {
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
            let app_handle_for_steps = app_handle.clone();
            thread::spawn(move || {
                println!("🖱️ 开始监听鼠标移动...");
                match Enigo::new(&Settings::default()) {
                    Ok(enigo) => {
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
            let app_handle_for_emit = app_handle.clone();
            thread::spawn(move || {
                loop {
                    let steps = {
                        let c = counter.lock().unwrap();
                        c.steps
                    };
                    let _ = app_handle_for_emit.emit_all("step_update", steps);
                    thread::sleep(Duration::from_secs(1));
                }
            });

            // 喝水提醒检查线程
            let water_state_clone = water_reminder_state.clone();
            let app_handle_for_water = app_handle.clone();
            thread::spawn(move || {
                println!("💧 开始喝水提醒检查...");
                
                // 等待一段时间再开始检查，避免应用刚启动就立即提醒
                thread::sleep(Duration::from_secs(10));
                
                loop {
                    // 每30秒检查一次，提高精确度
                    thread::sleep(Duration::from_secs(30));
                    
                    if let Ok(mut state) = water_state_clone.lock() {
                        println!("🔍 [CHECK] Running reminder check. Current state: {:?}", state);
                        if state.needs_reminder() {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            state.last_reminder_time = now;
                            
                            // 发送事件给宠物狗提醒
                            if let Err(e) = app_handle_for_water.emit_all("water_reminder", "time_to_drink") {
                                println!("❌ 发送喝水提醒事件失败: {}", e);
                            }
                            
                            println!("💧 发送喝水提醒事件给宠物狗");
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
