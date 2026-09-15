# 跨平台模拟按键（send_keys）前置权限与系统配置指南

## 1. 概述与背景

在 `mtools` 的端到端交互设计中，用户通过操作系统全局快捷键执行 `mtools copy` 唤起应用时，后端会向当前激活的第三方软件窗口发送**模拟复制按键**，以实现「选中文本/图片后无感捕获内容并自动唤出处理」的顺滑体验。此外，在部分回写或后置动作中，也会用到模拟粘贴功能。

现代桌面操作系统出于系统安全、反键盘记录（Keylogger）与跨进程权限隔离的考虑，对**合成全局输入事件（Synthetic Input Events）**均设有严格的权限控制与安全限制。

为了保证 `send_keys` 能够在您的操作系统中正常模拟按键，请参考以下针对不同操作系统的配置指南。

---

## 2. Linux 平台

### 2.1 底层机制与权限要求

`mtools` 在 Linux 下优先使用内核 **`uinput`（用户空间输入子系统，`/dev/uinput`）** 驱动虚拟键盘设备。
默认情况下，绝大多数 Linux 发行版将 `/dev/uinput` 设备节点的权限设置为仅允许 `root` 用户访问（或归属于 `root:input` / `root:uinput` 组且权限为 `0660`），普通用户无读写权限。若未授权，调用将报错 `can't open virtual keyboard`。

### 2.2 配置步骤

#### 步骤一：将当前用户加入 `input` 组

执行以下命令将当前用户添加到系统的 `input` 用户组中：

```bash
sudo usermod -a -G input $USER
```

_(注：部分发行版如 Arch Linux / Ubuntu 亦可检查是否存在 `uinput` 组并同时添加：`sudo usermod -a -G uinput $USER`)_。

#### 步骤二：配置持久化 udev 规则（推荐）

为了确保设备在系统重启或热插拔时始终具有组可读写权限，建议配置 udev 规则：

```bash
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-uinput.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

#### 步骤三：重启或重新登录生效

用户组权限的变更在 Linux 中不会即时对已登录的会话生效，**需要重启系统（推荐）或注销当前桌面会话重新登录**：

```bash
sudo reboot
```

> 💡 **临时测试提示**：在当前终端会话中，可执行 `newgrp input` 临时刷新组凭据进行即时验证。

#### 备选模式：ydotool

若环境限制无法直接访问 `/dev/uinput`，可在后台运行 `ydotoold` 守护进程，并确保其 Socket 具备访问权限：

```bash
# 安装 ydotool 后启动服务
systemctl --user enable --now ydotool
```

---

## 3. macOS 平台

### 3.1 底层机制与权限要求

`mtools` 在 macOS 下基于 Apple CoreGraphics 框架的 `CGEvent` 机制（`CGEventCreateKeyboardEvent` 与 `CGEventPost`），将按键事件直接注入到系统的 HID 全局事件流中。

macOS 自 macOS 10.14 (Mojave) 及更高版本（Catalina、Big Sur、Monterey、Ventura、Sonoma、Sequoia 等）起，通过 **TCC (Transparency, Consent, and Control)** 安全框架强制要求：**向其他应用程序注入合成事件的进程必须被显式授予「辅助功能（Accessibility）」权限**。

> ⚠️ **未授权的表现**：macOS 不会直接抛出致命崩溃，而是**静默吞掉（Drop）所有合成按键**，表现为划词唤起后未能抓取到最新选中的剪贴板内容。

### 3.2 配置步骤

#### 正式打包应用 (`mtools.app`)

1. 打开 macOS **「系统设置 (System Settings)」**；
2. 导航至 **「隐私与安全性 (Privacy & Security)」** -> **「辅助功能 (Accessibility)」**；
3. 在应用列表中找到 **`mtools`**，将其右侧开关切换为**允许（打开）**状态。

#### 本地开发与调试环境 (`tauri dev` / `cargo run`)

在开发阶段，由于二进制可执行文件是由终端或 IDE 拉起的，macOS 会校验**父进程（发起者）**的辅助功能权限：

1. 同样进入 **「系统设置」->「隐私与安全性」->「辅助功能」**；
2. 确保勾选了你运行命令所使用的**终端工具**（如 `Terminal.app`、`iTerm2`、`Ghostty`、`Alacritty` 等）或 **代码编辑器**（如 `Visual Studio Code`、`Cursor`）；
3. 如果修改权限后按键仍未生效，建议完全退出终端/IDE 后重新打开。

---

## 4. Windows 平台

### 4.1 底层机制与权限说明

`mtools` 在 Windows 下调用 Win32 原生 `SendInput` API 进行键盘按键注入。

Windows 平台对普通桌面应用之间的输入模拟无需特殊的系统级开关，**普通用户权限下开箱即用**。

### 4.2 特殊边界：UIPI (用户界面特权隔离)

Windows 引入了 UIPI (User Interface Privilege Isolation) 机制：

- 低完整性等级（普通用户权限）的应用程序**无法**向高完整性等级（管理员权限 / Elevated）的窗口发送输入消息。
- **典型场景**：若你当前正在使用以「管理员身份运行」的终端（如 Administrator 命令提示符、Elevated PowerShell、任务管理器或特权安装向导），普通权限启动的 `mtools` 将无法向该窗口发送 `Ctrl+C`。
- **解决方案**：若日常高频需要在管理员窗口中划词取词，可右键 `mtools.exe` 选择 **「以管理员身份运行」**。

---

## 5. 各平台模拟按键差异速查

为了适应各操作系统的操作习惯与终端兼容性，`mtools` 针对不同操作系统适配了不同的底层按键序列：

| 平台        | 复制按键 (`COPY_KEY`) | 粘贴按键 (`PASTE_KEY`) | 原因与考量                                                                                               |
| :---------- | :-------------------- | :--------------------- | :------------------------------------------------------------------------------------------------------- |
| **Windows** | `Ctrl + C`            | `Ctrl + V`             | Windows 标准全局快捷键。                                                                                 |
| **Linux**   | `Ctrl + Insert`       | `Shift + Insert`       | 避免在部分 Linux 终端中 `Ctrl+C` 触发 `SIGINT` 中断正在运行的进程，使用 X11/Linux 标准的备选剪贴板按键。 |
| **macOS**   | `Cmd + C`             | `Cmd + V`              | macOS 原生键盘规范，使用 Command 键而非 Control 键。                                                     |
