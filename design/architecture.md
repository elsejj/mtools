# 系统架构与核心时序设计 (System Architecture & Core Lifecycle)

## 1. 系统概述与技术选型

`mtools` 是一款基于 **Tauri v2 + Vue 3** 架构的桌面效率与工具分发软件。其核心形态为一个常驻或受快捷键激活的单例应用：当用户在操作系统任意软件中选中内容（文本、富文本、图片等）并按下快捷键后，软件被拉起，通过操作系统级模拟按键执行“复制”，获取剪切板载荷，利用智能路由引擎将其分发至最合适的目标工具，完成即时处理。

### 1.1 技术栈选型

- **Native 桌面容器**: Tauri v2
  - **跨平台核心 (Rust)**: 负责单例管理、模拟按键发送（`sendkey`）、系统剪贴板监听与读取、外部系统命令受控执行、原生窗口尺寸/位置状态持久化保存、本地 SQLite 数据库与文件系统操作。
  - **单例插件**: `tauri-plugin-single-instance`。
  - **剪贴板插件**: `tauri-plugin-clipboard-manager` + 原生增强。
- **前端表现层 (UI)**:
  - **框架**: Vue 3 (Composition API, `<script setup>`) + TypeScript。
  - **样式**: Tailwind CSS。
  - **运行时扩展引擎**: 纯 JS/TS 脚本执行器（支持内置及用户编写的自定义工具匹配函数和纯代码处理函数）。
- **数据持久化**:
  - 本地 SQLite (通过 Rust 端 `rusqlite` 或 Tauri SQL 插件) 存储历史记录、工具配置与全局设置。
  - 媒体文件缓存: 图片等大文件落盘于应用数据目录 `cache/images/`，数据库仅存储索引指针。

---

## 2. 软件分层架构

```mermaid
graph TD
    subgraph OS_Environment["操作系统环境 (OS: Windows / Linux / macOS)"]
        UserAction["用户全局选中内容 + 按下系统快捷键"]
        TargetApp["第三方前台窗口 (浏览器 / 编辑器 / 查看器等)"]
        SysClipboard["系统剪贴板 (Text / Image)"]
    end

    subgraph Native_Layer["Tauri 原生层 (Rust Backend - 重数据核心)"]
        AppLifecycle["单例与启动守卫 (Single Instance Manager)"]
        SendKeyEngine["按键模拟引擎 (sendkey)"]
        ClipboardExtractor["app.clipboard() 提取剪贴板 (Text / Image)"]
        RustDecoder["前置解码器链 (Decoder: Base64 / Hex / URL-Percent)"]
        RustSniffer["插件化内容嗅探器 (Sniffers: JSON/URL/JWT/时间戳/图片等)"]
        RustPreprocessor["预处理与格式化引擎 (Preprocessors: Pretty JSON / 图片归档)"]
        CommandExecutor["外部命令执行器 (CLI Process Runner)"]
        WindowStateManager["窗口位置与尺寸记忆管理 (Window State)"]
        DatabaseEngine["本地存储引擎 (SQLite / File Cache)"]
    end

    subgraph Bridge_Layer["IPC 通信通道 (Tauri IPC / Events)"]
        IPCEvents["事件发射器: 'payload-ready' (携带解码溯源、预处理与推荐结果)"]
        IPCCommands["命令调用: 执行CLI / 读写配置 / 历史查询 / 重新处理"]
    end

    subgraph Frontend_Layer["前端表现层 (Vue3 + TS - 瘦客户端专职呈现)"]
        UI_Display["主工作台渲染: 代码高亮 / Markdown / 图像预览 / 解码提示"]
        UI_Toolbar["工具栏快捷切换与状态推荐"]
        UI_History["历史记录与版本回溯抽屉"]
        UI_Settings["系统与工具配置中心"]
    end

    UserAction --> AppLifecycle
    AppLifecycle -->|激活已存在实例| SendKeyEngine
    SendKeyEngine -->|模拟 Ctrl+C / Cmd+C| TargetApp
    TargetApp -->|内容写入| SysClipboard
    SendKeyEngine -.->|延时等待 50~100ms| ClipboardExtractor
    ClipboardExtractor --> SysClipboard
    ClipboardExtractor --> RustDecoder
    RustDecoder -->|还原真实实体 如解出JSON或升格图片| RustSniffer
    RustSniffer --> RustPreprocessor
    RustPreprocessor --> IPCEvents
    IPCEvents --> UI_Display
    UI_Toolbar -.->|切换工具或重新执行| IPCCommands
    IPCCommands --> CommandExecutor
    IPCCommands --> DatabaseEngine
    WindowStateManager <--> UI_Display
```

---

## 3. 完整启动与触发核心时序 (End-to-End Execution Flow)

当用户在任意外部软件中选中内容后，触发软件的核心时序如下：

```mermaid
sequenceDiagram
    autonumber
    actor User as 用户
    participant ExtApp as 外部软件(选中文本/图片)
    participant Hotkey as 系统全局快捷键
    participant Native as Tauri后端 (Rust)
    participant Clipboard as 系统剪贴板
    participant Frontend as 前端应用 (Vue 3 瘦客户端)

    User->>ExtApp: 选中待处理的文本或图片
    User->>Hotkey: 按下设定的启动快捷键
    Hotkey->>Native: 启动命令行 `mtools copy`
    Native->>Native: single-instance 捕获次级启动，提取参数 ["copy"]

    rect rgb(240, 248, 255)
    note over Native, ExtApp: 模拟复制与剪贴板捕获阶段
    Native->>ExtApp: 调用 sendkey 模拟释放快捷键并发送 Ctrl+C (Cmd+C)
    ExtApp->>Clipboard: 将选中内容写入剪贴板
    Native->>Native: 异步延时等待 (Debounce ~80ms) 确保剪贴板写入完毕
    Native->>Clipboard: app.clipboard().read_text() / read_image()
    end

    rect rgb(230, 245, 255)
    note over Native: Rust 前置解码、嗅探与格式化预处理
    Native->>Native: 前置解码: 识别 Base64/Hex，解出真实文本或升格为图片载荷
    Native->>Native: 嗅探特征: 插件化责任链匹配 (JSON/URL/JWT/时间戳/图片) 并打标
    Native->>Native: 执行内置预处理 (如 JSON 自动格式化为 Pretty JSON、图片存盘)
    Native->>Native: 计算各工具匹配置信度，确定推荐目标工具
    Native->>Native: 恢复窗口上一次记忆的位置与尺寸并展示、设置焦点聚焦
    end

    rect rgb(245, 255, 245)
    note over Native, Frontend: 结果直达与前端纯展示
    Native->>Frontend: 发射 `payload-ready` 事件 (包含解码溯源、预处理好文本、推荐工具)
    Frontend->>Frontend: 自动定位到目标工具，直接渲染已格式化/处理好的数据 (前端零耗时计算)
    Frontend-->>User: 界面高亮展示最终结果 + 解码溯源提示 / 触发后置处理 (如自动复制或存盘)
    Native->>Native: 异步持久化该条历史记录至本地 SQLite
    end
```

---

## 4. 关键技术细节与工程考量

### 4.1 模拟按键时序与防竞争 (Race Condition Prevention)

1. **键状态复位**: 用户触发快捷键通常需要按下物理组合键（例如 `Super+Alt+C` 或 `Ctrl+Shift+C`）。在模拟系统 `Ctrl+C` 之前，必须确保物理辅助键已被释放或在模拟代码中做 KeyUp 处理，否则第三方应用可能接收到错误组合键。
2. **写剪贴板缓冲等待**: 外部软件将数据序列化至剪贴板不是即时完成的（特别是大文本或图片），因此后端在模拟复制后设置可配置的自适应延时（默认 `80ms`），并读取剪贴板版本号/Hash，防止读出旧的剪贴板遗留数据。

### 4.2 窗口行为与状态持久化 (Window State Persistence)

- 用户的明确决策：**普通桌面窗口，记住上一次位置与大小**。
- **实现机制**：
  - 利用 Tauri 的窗口事件监听 (`tauri::WindowEvent::Resized`, `tauri::WindowEvent::Moved`)。
  - 在应用退出或窗口失焦时，异步更新配置表存储的 `{ x, y, width, height, is_maximized }`。
  - 实例被 `copy` 唤起时：
    1. 若窗口处于最小化，调用 `window.unminimize()`；
    2. 若窗口处于隐藏，调用 `window.show()`；
    3. 调用 `window.set_focus()` 确保置于前台，同时恢复上次关闭时的矩形边界。

### 4.3 多进程与单例设计

- `tauri-plugin-single-instance` 在主进程启动时建立本地 IPC 套接字（Unix Domain Socket 或 Windows Named Pipe）。
- 当新进程带参数 `mtools copy` 启动时，将命令行参数发送给主进程，新进程立即退出，主进程回调接收到 `args` 并进入复制与激活流程。
