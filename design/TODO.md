# mtools 项目实施 TODO 任务清单 (Implementation Roadmap)

本文档是 `mtools` 项目的实施路线图与阶段性任务清单。整个开发流程分为 6 个阶段，按底层基础设施到前端交互呈现循序渐进推进。

---

## 阶段一：Rust 原生后端核心基建 (Phase 1: Rust Core & Pipeline)

### 1.1 依赖与 Cargo 模块配置
- [x] 在 `src-tauri/Cargo.toml` 补全核心依赖项：
  - `rusqlite` (本地数据库)
  - `base64`, `hex`, `url` (前置解码与协议解析)
  - `image` (图片格式与魔数识别)
  - `chrono` (日期时间与文件命名)
  - `uuid` (唯一标识生成)
  - `tokio` (异步子进程与超时)
- [x] 规划 Rust 端模块目录结构 (`sniffer/`, `decoder/`, `storage/`, `cli/`, `window/`, `post_action/`)

### 1.2 模拟按键与剪贴板增强
- [x] 完善 `sendkey` 模拟复制后自适应延迟（80ms~100ms）防竞争机制
- [x] 基于 `app.clipboard()` 封装稳定的文本/图片读取函数，自动适配空剪贴板与格式异常

### 1.3 前置解码与解包流水线 (`decoder/`)
- [x] 实现 `DecoderChain` 解码器责任链
- [x] 实现 Base64 探测与解码，结合图片魔数（PNG/JPEG/WebP）自动升格为图片载荷
- [x] 实现 Hex 十六进制探测与 UTF-8 文本安全解码
- [x] 实现 URL Percent-encoding 探测与解码
- [x] 记录解码链路追踪信息 (`decodingTrace`)

### 1.4 灵活可扩展的嗅探器体系 (`sniffer/`)
- [x] 定义 `ContentSniffer` 标准 Trait 与 `SniffOutput` 结构体
- [x] 实现单例 `SnifferRegistry`，支持按优先级调度和动态注册
- [x] 实现内置核心嗅探器：
  - [x] `JsonSniffer`: 尝试解析，并直接生成格式化后的 Pretty JSON 预处理文本
  - [x] `UrlSniffer`: 提取协议、域名、路径与 Query 参数
  - [x] `JwtSniffer`: 三段式 Token 探测与 Header/Payload 解包
  - [x] `TimestampSniffer`: 秒/毫秒时间戳探测与双向可读时间转换
  - [x] `ImageSniffer`: 提取图片尺寸、MIME 类型、生成 Base64 及年月分级落盘
  - [x] `DynamicRegexSniffer`: 支持根据工具配置动态构造正则嗅探器

### 1.5 智能路由与载荷封装 (`router/`)
- [x] 实现基于嗅探标签、正则与工具声明的打分仲裁算法
- [x] 组装 `EnrichedPayload` 并通过 `app.emit("payload-ready", ...)` 推送前端

### 1.6 后置处理与文件操作 (`post_action/`)
- [x] 实现 `save_content_to_file`：支持工具专属目录递归创建与时间戳自动命名
- [x] 实现 `show_in_folder`：在原生文件管理器中定位文件
- [x] 实现 `pick_directory_dialog`：原生目录选择对话框

### 1.7 本地 SQLite 存储与缓存管理 (`storage/`)
- [x] 历史记录表 (`history_records`) 初始化与 CRUD (增、查、删、清空)
- [x] 图片缓存落盘规范（`cache/images/YYYY-MM/YYYY-MM-DD_HH-mm-ss.png`）
- [x] 图片配额淘汰 (FIFO) 与 TTL 清理逻辑，联动历史记录删除
- [x] 窗口矩形状态 (`WindowGeometry`) 保存与恢复

### 1.8 外部 CLI 执行引擎 (`cli/`)
- [x] 实现 `execute_cli_command`：支持 stdin 管道、参数占位符替换、工作目录与超时

### 1.9 Tauri Commands 统一导出
- [x] 在 `lib.rs` 与 `commands.rs` 注册所有设计文档中声明的 `#[tauri::command]`
- [x] 运行 `bun run check:m` 确保 Rust 端无报错编译通过


---

## 阶段二：前端基础工程与状态架构 (Phase 2: Frontend Infrastructure)

### 2.1 UI 体系配置
- [x] 引入并配置 Tailwind CSS
- [x] 检查并初始化 `shadcn-vue` 基础组件环境
- [x] 引入 Tabler / Lucide 图标库与语法高亮支持 (Prism)

### 2.2 Pinia 状态管理层搭建
- [x] `usePayloadStore`: 监听 `payload-ready` 事件，管理当前载荷、解码溯源标签
- [x] `useToolStore`: 管理内置与自定义工具池、当前激活工具、智能推荐工具列表
- [x] `useHistoryStore`: 历史记录拉取、搜索筛选、删除与一键恢复
- [x] `useSettingsStore`: 系统全局配置、AI 服务商列表、主题切换

### 2.3 窗口事件与快捷键监听
- [x] 监听窗口 `resize` / `move`，防抖调用 `save_window_geometry`
- [x] 挂载全局键盘快捷键监听 (`Ctrl+K`、`Alt+1~9`、`Ctrl+H`)

---

## 阶段三：主工作台界面与展现交互 (Phase 3: Main Workspace UI)

### 3.1 页面宏观布局
- [x] 顶部导航栏：Logo、候选推荐工具条（带 `Alt+数字` 快捷键徽标）、设置与历史入口
- [x] 左侧边栏：工具分类列表、搜索过滤框、快捷添加自定义工具按钮
- [x] 主工作区：左右分栏（输入原稿 vs 处理结果）

### 3.2 高效结果渲染 (零数据计算，纯展示)
- [x] 渲染 Rust 端已格式化好的 Pretty JSON（代码语法高亮与折叠）
- [x] 渲染多模态图片预览组件
- [x] 顶部“解码溯源提示条”（如：“💡 已自动通过 Base64 解码，点击可查看原始 Base64 内容”）

### 3.3 底部交互栏与后置反馈
- [x] 一键复制结果到剪贴板
- [x] “回贴源软件”按键（触发 `simulate_paste`）
- [x] 执行耗时与状态指示器
- [x] 后置处理结果反馈（如保存文件后的物理路径链接，点击调用 `show_in_folder`）

---

## 阶段四：工具生态与三大引擎打通 (Phase 4: Tool Engines Integration)

### 4.1 内置代码处理型工具
- [ ] 验证 JSON 格式化工具全链路跑通
- [ ] 验证 URL 编解码 / 参数解析工具
- [ ] 验证 JWT 结构展示工具
- [ ] 验证时间戳互转工具

### 4.2 大模型客户端与流式输出
- [ ] 封装统一的 OpenAI 兼容客户端
- [ ] 支持纯文本与图文多模态动态拼装 (Content Parts)
- [ ] 支持 SSE 流式打字机渲染与一键中断生成 (`AbortController`)
- [ ] 验证 OCR 文字提取工具
- [ ] 验证多语言翻译与文本润色工具

### 4.3 外部 CLI 命令工具接入
- [ ] 联调调用系统 CLI（如 `jq`, `python` 等）输出实时回显

### 4.4 后置处理流水线集成
- [ ] 验证流式/非流式执行完成后，自动触发后置动作（自动复制 / 自动按时间戳存盘）

---

## 阶段五：工具管理、历史回溯与系统配置 (Phase 5: Management & Settings)

### 5.1 自定义工具向导
- [ ] 新增/编辑工具弹窗：支持代码型、AI 型、CLI 型向导式表单
- [ ] 规则匹配实时验证面板（输入样例即可实时查看匹配分数）
- [ ] 配置专属后置处理目录与行为
- [ ] 工具的停用、排序、删除与 JSON 导出/导入

### 5.2 历史记录侧边抽屉
- [ ] 历史记录时间线倒序展示与分页
- [ ] 工具类型快捷筛选与全文搜索
- [ ] 历史卡片徽标（`📋 已复制` / `📁 已存盘`）
- [ ] 一键将历史记录重播回填至主工作台

### 5.3 系统设置中心
- [ ] AI 服务商池管理（添加多个 Provider、配置 BaseURL、API Key、测试连接）
- [ ] 设定全局默认模型
- [ ] 主题切换（Light / Dark / Follow System）
- [ ] 图片缓存统计展示（大小/张数）与一键清理按钮

---

## 阶段六：全链路闭环联调与验证 (Phase 6: Verification & Polish)

### 6.1 端到端典型场景测试
- [ ] **场景 1（普通文本与纠偏）**：在外部选中乱序 JSON，快捷键启动，验证是否自动由非相关工具平滑纠偏跳转至 JSON 格式化工具，并以 Pretty 格式呈现。
- [ ] **场景 2（Base64 还原与升格）**：在外部选中 Base64 图片，快捷键启动，验证前置解码器是否检测到 PNG 魔数并自动升格为图片，直接送入 OCR / 图片工具。
- [ ] **场景 3（后置处理）**：配置 OCR 工具后置处理为存盘，测试识别完成后是否在专属目录生成 `YYYY-MM-DD_HH-mm-ss.txt` 文件并支持一键打开。
- [ ] **场景 4（窗口位置记忆）**：移动与缩放窗口后关闭，外部重新快捷键唤起，验证窗口是否精确在上次位置与大小还原。

### 6.2 编译与代码检查
- [ ] 运行 `bun run check:f` 确保前端 TypeScript 类型检查零错误
- [ ] 运行 `bun run check:m` 确保 Rust 后端编译通过、无 warning

