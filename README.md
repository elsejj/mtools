<div align="center">

# 🛠️ mtools

**下一代跨平台桌面级全能效率工具箱与智能内容分发工作台**

基于 **Tauri v2 + Vue 3 + Rust** 打造，集「选中文本/图片一键直达、前置智能解码、特征嗅探自动路由、多模态 AI 与本地代码/CLI 扩展」于一体的现代化生产力利器。

[![Tauri v2](https://img.shields.io/badge/Tauri-v2-24C8D8?logo=tauri&logoColor=white)](https://tauri.app/)
[![Vue 3](https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vuedotjs&logoColor=white)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tailwind CSS](https://img.shields.io/badge/TailwindCSS-v4-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

</div>

---

## 📖 简介 (Introduction)

日常开发与办公中，我们经常遇到琐碎的数据处理需求：解析一段 JSON、解密 JWT Token、换算时间戳或中英文单位表达式、识别图片里的文字（OCR）、翻译润色一段句子，或是用系统 CLI 管道处理特定文本。传统的做法是打开多个网页工具或不同软件，频繁复制粘贴，上下文极易割裂。

**mtools** 的核心设计哲学是 **「唤起即得，智能直达」**：
你只需在操作系统的任何第三方软件（浏览器、IDE、聊天软件、文档查看器等）中**划词选中或复制内容**，按下全局快捷键，mtools 会自动捕获内容、运行前置解码与特征嗅探，以最匹配的工具呈现实时处理结果，并支持自动复制或落盘归档。

---

## ✨ 核心特性 (Key Features)

### 🚀 1. 极速唤起与端到端无感捕获 (Instant Trigger)

- **全局快捷键联动**：支持绑定系统全局快捷键直接执行 `mtools copy` 呼出窗口。
- **模拟按键与原子捕获**：Rust 原生向当前激活窗口发送模拟按键获取最新剪贴板内容，消除上下文切换成本。
- **防竞态与窗口状态记忆**：采用单例进程通道，自动记录并恢复上次关闭时的窗口尺寸和屏幕坐标（非简陋悬浮窗，具备大篇幅阅读操作体验）。

### 🔍 2. 深度前置解码与溯源管道 (Decoder Pipeline)

在将数据送入工具前，Rust 后端构建了前置深度解码流水线：

- **文件路径自动展开**：自动探测剪贴板中的多行/单行本地文件路径，若指向图片或文本，自动就地提取解析。
- **引号与转义剥除**：自动去除外层包裹的单/双引号并还原 JSON 转义字符。
- **多重编码逆向解包**：自动探测并解码 **Base64**、**DataURL**、**Hex 十六进制**、**URL 百分号编码**。若 Base64 解码后为图片，自动升格为图片实体载荷；同时完整记录**解码溯源链（Trace）**，原汁原味追踪变换过程。

### 🎯 3. 插件化特征嗅探与智能路由 (Sniffer & Smart Routing)

- **Rust 原生高速嗅探链**：内置 JSON、JWT、URL、时间戳/日期、中英文单位表达式、图片魔数、动态正则与纯文本嗅探器。
- **智能置信度打分算法**：根据输入特征实时为每个工具动态打分。例如输入标准 JSON 自动置顶「JSON 格式化」，输入图片自动聚焦「OCR 识图提取」。
- **毫秒级自适应纠偏**：即使当前停留在不匹配工具，也能无缝纠偏至最匹配项；同时提供快捷候选栏，通过 `Alt + 1~9` 一键切换候选工具。

### ⚙️ 4. 三大核心处理引擎 (Three Powerful Engines)

| 引擎类型                           | 描述与能力                                                                                                                                          | 适用场景与示例                                                                                                              |
| :--------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------- |
| **代码处理引擎<br>(Code Tool)**    | 基于前端沙箱与纯 JavaScript/TypeScript 运行，支持自定义函数脚本与 JSON/Markdown/纯文本格式化输出。                                                  | JSON 语法校验与美化、JWT 结构反解与过期判断、URL Query 参数结构化提取、时间戳多维度互转、带单位计算器。                     |
| **大模型多模态引擎<br>(LLM Tool)** | 原生接入 OpenAI 兼容规范（支持 **OpenAI、DeepSeek、Anthropic 兼容协议、Ollama 本地大模型** 等），图文原生多模态组装，SSE 流式打字机响应与中断控制。 | 多模态 OCR 视觉识图、长短文翻译与润色、代码解释、自然语言重写等。支持全局 Provider 与单个工具独立覆盖系统级 Prompt 与参数。 |
| **外部系统命令引擎<br>(CLI Tool)** | Rust 异步子进程驱动，支持将输入载荷通过管道 (`stdin/pipe`) 传入，具备超时控制与安全可控的输出捕获。                                                 | 调用系统已有工具链，如 `jq` 高级筛选、`cat`、`sed`、本地 Python/Shell 脚本转换等。                                          |

### 📦 5. 灵活的后置动作执行管道 (Post-Action Pipeline)

工具执行完毕后，可自由配置自动化后续动作：

- **默认无动作 (None)**：仅在界面查看结果。
- **自动复制到剪贴板 (Copy to Clipboard)**：处理完成即刻写回系统剪贴板（如 OCR 识别完毕后可直接在编辑器中粘贴）。
- **自动保存为文件 (Save to File)**：按工具独立目录归档，支持时间戳自动命名、文件名冲突消解，并支持在系统文件管理器中一键定位展示（Reveal in Folder）。

### 💾 6. 数据持久化与现代桌面 UX

- **本地 SQLite 历史记录**：所有处理记录本地存储，支持按工具/时间搜索、一键重放复现、历史记录清除。
- **媒体文件分级缓存**：剪贴板图像等大文件落盘于应用数据目录缓存，数据库只维护元数据索引，并提供缓存容量统计与一键清理功能。
- **优雅交互与主题切换**：支持跟随系统、亮色与暗色主题；全套快捷键驱动键盘流操作；输入框支持手动键入并重新嗅探分发。

---

## 🧰 内置工具矩阵 (Built-in Tools)

| 图标 | 工具名称          | 引擎 | 触发特征 / 接受类型             | 功能说明                                                                                     |
| :--: | :---------------- | :--: | :------------------------------ | :------------------------------------------------------------------------------------------- |
|  🔣  | **JSON 格式化**   | Code | `text` (JSON 语法)              | 格式化并高亮 JSON 字符串，语法纠错，支持折叠与最小化                                         |
|  🔑  | **JWT 解析**      | Code | `text` (JWT Token)              | 解析 Header 与 Payload 声明，友好显示到期时间（过期提示）与生效时间                          |
|  ⏰  | **时间戳转换**    | Code | `text` (秒/毫秒/微秒/浮点/日期) | 支持 Unix 秒、毫秒、Python 浮点与常见日期文本双向转换，附带相对时间与中英文计数表达          |
|  🧮  | **智能计算器**    | Code | `text` (算式/单位)              | 智能支持中英文数量单位（如 `1.5万 * 2`、`3K + 500`、`2.5亿 / 5000`、`M/G/T/P/万亿`）直接运算 |
|  🔗  | **URL 编解码**    | Code | `text` (URL 链接)               | URL Encode / Decode，自动解析提取 Query 参数为键值对表格/JSON                                |
|  🖼️  | **OCR 识图提取**  | LLM  | `image` (剪贴板图片/文件)       | 利用多模态 AI 识别并提取图片中的全部排版文字，默认自动复制结果                               |
|  🌐  | **AI 翻译与润色** | LLM  | `text` (多语言文本)             | 中英双语即时翻译与专业文案表达润色，流式打字机输出                                           |
|  💻  | **外部 CLI**      | CLI  | `text` (任意文本)               | 通过 stdin 管道将数据传递给本地系统命令行程序（如 `cat`、`jq` 等）                           |

> 💡 你还可以在「工具管理中心」自由新增专属的自定义 Code、LLM 或 CLI 工具！

---

## ⌨️ 快捷键速查 (Shortcuts Cheat Sheet)

在 mtools 窗口中，你可以使用以下预设快捷键进行高效率操作：

| 快捷键                         | 功能描述                         |
| :----------------------------- | :------------------------------- |
| `Ctrl + K` / `Cmd + K`         | 聚焦顶部手动搜索/输入框          |
| `Alt + 1` ~ `Alt + 9`          | 快速切换至对应推荐排名的候选工具 |
| `Ctrl + Enter` / `Cmd + Enter` | 立即重新执行当前工具             |
| `Ctrl + H` / `Cmd + H`         | 打开/关闭历史记录抽屉            |
| `Ctrl + ,` / `Cmd + ,`         | 打开系统与全局设置面板           |
| `Escape`                       | 关闭弹窗、抽屉或清除焦点         |

---

## 🛠️ 安装与使用指南 (Getting Started)

### 前置环境要求 (Prerequisites)

- **Rust**: 建议使用最新稳定版（Rust 1.77+ / 2021 Edition）
- **Bun**: 前端包管理器与脚本运行时（推荐 Bun 1.1+）或 Node.js (v18+)
- **系统底层依赖 (Linux)**:
  - 若在 Linux 环境（X11 / Wayland）下使用按键模拟与托盘，需要安装开发库：
    ```bash
    # Ubuntu / Debian
    sudo apt-get update
    sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
    ```

---

### 开发与本地运行 (Local Development)

1. **克隆仓库并安装前端依赖**：

   ```bash
   git clone https://github.com/your-username/mtools.git
   cd mtools
   bun install
   ```

2. **以开发模式运行 Tauri**：

   ```bash
   bun run tauri dev
   ```

   > 该命令会自动拉起 Vite 开发服务器，并在原生 Rust 容器中渲染前端应用，支持热重载（HMR）。

3. **常用项目指令**：
   ```bash
   bun run check:f    # 校验前端 TypeScript 与 Vue 模板语法
   bun run check:m    # 校验后端 Rust 编译与依赖正确性
   bun run lint       # 使用 oxlint 执行代码质量检查
   bun run shadcn     # shadcn-vue 组件管理命令行
   ```

---

### 打包构建 (Production Build)

构建可直接发布的跨平台独立客户端安装包：

```bash
bun run tauri build
```

打包生成的文件将位于 `src-tauri/target/release/bundle/` 目录中：

- **Linux**: `.deb`, `.AppImage`
- **macOS**: `.dmg`, `.app`
- **Windows**: `.msi`, `.exe` 安装程序

---

### 🌟 推荐姿势：配置操作系统全局快捷键 (OS Global Shortcut)

为了实现「在任何软件中选中文本/图片后一键呼出」，建议在你的操作系统键盘快捷键设置中新增自定义快捷键：

- **快捷键示例**：`Ctrl + Alt + C` 或 `Super + C`
- **执行命令**：`mtools copy`

**工作机制**：

1. 当你按下快捷键时，系统执行 `mtools copy`；
2. mtools 单例插件接收到二级参数 `copy`；
3. 后端自动模拟释放当前按键并触发 `Ctrl+C` 写入剪贴板；
4. 等待剪贴板写入完成后自动捕获、解码并唤醒置顶 mtools 窗口展示结果！

---

## 🏗️ 架构全景 (System Architecture)

```
mtools 总体架构
├── src/ (前端呈现层 - Vue 3 + TypeScript + Tailwind)
│   ├── components/
│   │   ├── history/      # 历史记录抽屉与详情查看
│   │   ├── layout/       # 顶栏导航、工具侧边栏与布局底座
│   │   ├── settings/     # 系统设置与 LLM Provider 模态框
│   │   ├── tools/        # 工具管理中心与自定义工具编辑器
│   │   ├── ui/           # 基于 shadcn-vue 的基础原子组件库
│   │   └── workspace/    # 主工作台（源内容查看器、结果呈现、底部快捷栏）
│   ├── composables/      # 窗口状态记忆、键盘快捷键监听
│   ├── lib/engines/      # 代码执行沙箱与大模型 SSE 流式引擎
│   └── stores/           # Pinia 状态管理 (Payload, Tools, History, Settings)
│
└── src-tauri/ (原生后端核心 - Rust)
    ├── src/
    │   ├── cli/          # 外部子进程管道执行引擎 (stdin/stdout)
    │   ├── decoder/      # 前置解码链 (Base64/Hex/URL/文件路径反解)
    │   ├── sniffer/      # 插件化特征嗅探器 (JSON, JWT, URL, Time, Calc, Image 等)
    │   ├── storage/      # SQLite 数据库引擎、图片文件缓存、窗口几何记忆
    │   ├── post_action/  # 后置动作执行器 (存盘与文件管理器呼出)
    │   ├── sendkey/      # 模拟按键与系统剪贴板协同
    │   ├── commands.rs   # Tauri IPC 接口导出层
    │   ├── models.rs     # 数据模型定义 (Payload, Config, History)
    │   └── tray.rs       # 系统托盘菜单与生命周期接管
    └── Cargo.toml
```

---

## 🤖 大模型接入指南 (LLM Setup)

在 mtools 的「设置 (`Ctrl+,`)」页面中，你可以配置 OpenAI 兼容的 API 供应商：

- **DeepSeek**：
  - Base URL: `https://api.deepseek.com/v1`
  - 默认模型: `deepseek-chat`
- **OpenAI**：
  - Base URL: `https://api.openai.com/v1`
  - 默认模型: `gpt-4o`
- **本地 Ollama**：
  - Base URL: `http://localhost:11434/v1`
  - 默认模型: `llama3.2-vision` / `qwen2.5`
- **其他兼容服务**：Moonshot (Kimi)、零一万物、阿里百炼、SiliconFlow 等只要兼容 OpenAI 标准的接口均可无缝填入。

---

## 📄 开源许可证 (License)

本项目采用 [MIT License](LICENSE) 开源许可证。欢迎提交 Issue 与 Pull Request 共同建设！
