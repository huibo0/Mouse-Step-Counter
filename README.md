# 🖱️ Mouse Step Counter

一个现代化的鼠标计步器应用，实时监听鼠标移动并转换为步数统计。

## ✨ 功能特性

- 🖱️ **实时监听** - 50ms间隔检测鼠标移动
- 📊 **步数统计** - 100像素移动距离 = 1步
- 📈 **进度显示** - 可视化进度条和统计信息
- 🔄 **重置功能** - 一键重置计数器
- 💫 **现代UI** - Material-UI设计，渐变背景
- 🪟 **后台运行** - 最小化窗口时也能继续计数
- 🌐 **跨平台** - 支持 Windows、macOS、Linux

## 🏗️ 技术栈

- **前端**: React + TypeScript + Material-UI
- **后端**: Rust + Tauri
- **构建**: Vite + Tauri CLI
- **CI/CD**: GitHub Actions

## 🚀 本地开发

### 环境要求

- Node.js 18+
- Rust 1.60+
- 系统依赖:
  - **macOS**: 需要授予辅助功能权限
  - **Linux**: 需要安装相关GUI库
  - **Windows**: 需要Visual Studio Build Tools

### 安装依赖

```bash
# 克隆项目
git clone <项目地址>
cd mouse-step-counter

# 安装前端依赖
npm install

# 安装Rust (如果没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 开发运行

```bash
# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 📦 GitHub Actions 自动构建

我们提供了两种构建方案：

### 方案1: 自动发布 (推荐)

当你推送版本标签时自动构建并发布：

```bash
# 1. 更新版本号
npm version patch  # 或 minor/major

# 2. 推送标签
git push origin v1.0.0

# 3. GitHub Actions 自动构建三平台版本并发布Release
```

### 方案2: 手动构建

在GitHub仓库页面手动触发：

1. 进入 **Actions** 页面
2. 选择 **Manual Build** workflow
3. 点击 **Run workflow**
4. 输入版本号
5. 等待构建完成
6. 在 **Artifacts** 中下载构建产物

## 📋 构建产物

每个平台的构建产物：

| 平台 | 文件类型 | 说明 |
|------|----------|------|
| **Windows** | `.msi`, `.exe` | 安装包和便携版 |
| **macOS** | `.app`, `.dmg` | 应用包和磁盘镜像 |
| **Linux** | `.deb`, `.AppImage` | Debian包和便携版 |

## 🔧 权限设置

### macOS
1. 打开 **系统设置** > **隐私与安全性** > **辅助功能**
2. 添加 **Terminal** 或应用本身
3. 确保开关打开 ✅

### Windows
- 应用会自动请求必要权限

### Linux
- 确保安装了必要的GUI库依赖

## 📖 使用说明

1. **启动应用** - 打开后会显示现代化界面
2. **移动鼠标** - 系统开始自动记录移动距离
3. **查看步数** - 界面实时显示当前步数和统计
4. **重置计数** - 点击重置按钮清零计数器
5. **最小化** - 窗口最小化后依然继续计数

## 🎯 计数逻辑

- **检查频率**: 每50毫秒检测一次鼠标位置
- **距离计算**: 使用欧几里得距离公式 `√(dx² + dy²)`
- **步数转换**: 累积移动距离 ÷ 100像素 = 步数
- **实时更新**: 每秒向界面推送一次步数更新

## 🛠️ 自定义配置

可以修改的参数：

```rust
// src-tauri/src/main.rs
let new_steps = (c.total_distance / 100.0) as u32; // 修改步数比例
thread::sleep(Duration::from_millis(50));          // 修改检测频率
```

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**享受你的鼠标计步之旅！** 🖱️✨ 