# nmon-rs

[![rust](https://img.shields.io/badge/rust-%23e24d2c.svg?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![ratatui](https://img.shields.io/badge/ratatui-%23e63946.svg?style=flat-square&logo=ratatui&logoColor=white)](https://crates.io/crates/ratatui)
[![crossterm](https://img.shields.io/badge/crossterm-%232a9d8f.svg?style=flat-square&logo=terminal&logoColor=white)](https://crates.io/crates/crossterm)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/bugsbunny8/nmon-rs?style=flat-square&logo=github)](https://github.com/bugsbunny8/nmon-rs/releases/latest)
[![github actions](https://img.shields.io/github/actions/workflow/status/bugsbunny8/nmon-rs/release.yml?style=flat-square&logo=github)](https://github.com/bugsbunny8/nmon-rs/actions)

`nmon-rs` 是一个基于 **Rust** 编写的终端系统性能监视器，重新实现了经典的 **nmon** (Nigel's Monitor) 工具。它支持交互式的终端用户界面 (TUI) 模式以及与标准 nmon 分析工具兼容的非交互式 CSV 日志记录模式。

---

## 功能特性

- **交互式 TUI 仪表盘**：基于 `ratatui` 和 `crossterm` 构建，可实时展示丰富、直观的系统指标面板。
- **CSV 日志记录模式**：支持将系统快照保存为 `.nmon` 格式的文件。该格式严格遵循 nmon 标准结构（包括配置 AAA 区、时间戳 ZZZZ 区和各项指标数据行），非常适合直接导入电子表格或 nmon 专用的分析工具。
- **异步告警引擎**：采用简单的阈值告警机制，在后台线程中异步评估指标快照，一旦触发（例如 CPU 使用率 > 85%，内存使用率 > 90%）即会自动将警报记录至 `alerts.log`。
- **跨平台指标收集支持**：
  - 收集详尽的 CPU 核心数据（使用率、核心数、主频、品牌、厂商等）。
  - 追踪物理内存（RAM）与交换空间（Swap）的占用。
  - 监控各磁盘分区的存储容量、读写吞吐量以及 nmon 风格的磁盘繁忙度映射（%Busy Map）。
  - 监控活动网络接口的带宽速率（每秒接收/发送的字节数）。
  - 展示当前系统活动进程列表（可动态按 CPU 占用率或内存占用大小进行排序）。
  - 读取操作系统元数据（系统名称、版本、内核版本、运行时间、系统平均负载）。

---

## 交互式快捷键控制 (TUI 模式)

在 TUI 模式下运行时，您可以使用以下单键快捷键来开启或隐藏特定的监控面板：

| 按键 | 说明 |
|---|---|
| **`c`** | 切换 CPU 利用率面板 (SMP 多核视图) |
| **`l`** | 切换 CPU 长期历史趋势图面板 |
| **`m`** | 切换内存 & 交换空间 (Swap) 利用率面板 |
| **`V`** | 切换虚拟内存 (Virtual Memory) 指标面板 (包含 Swap 换页统计) |
| **`d`** | 切换磁盘汇总 I/O 趋势图 |
| **`D`** | 切换磁盘详细状态表格 (展示各磁盘的吞吐速率与分区容量) |
| **`o`** | 切换磁盘繁忙度映射图 (%Busy Map) |
| **`j`** | 切换文件系统 (JFS) 空间占用与使用率条形图面板 |
| **`n`** | 切换网络接口流量面板 (RX/TX 吞吐速率) |
| **`t`** | 切换活动进程排行列表面板 (Top 进程) |
| **`r`** | 切换系统资源元数据面板 (包含硬件规格与操作系统信息) |
| **`k`** | 切换内核统计、平均负载及运行时间面板 |
| **`h`** | 切换快捷键迷你帮助信息 |
| **`+`** | 双倍延长快照刷新间隔（降低更新频率） |
| **`-`** | 减半快照刷新间隔（提高更新频率） |
| **`Space`** | 强制立即刷新系统指标 |
| **`q`** | 退出程序 |

*当 **Top 进程列表 (`t`)** 面板处于活动状态时：*
- 按 **`4`** 键：按**内存消耗量**对进程进行降序排列。
- 按 **`5`** 键：按**CPU 占用率**对进程进行降序排列。

---

## 安装与构建

请确保您的系统上已安装 Rust 工具链。克隆仓库并执行以下命令构建：

```bash
# 开发/调试模式构建
cargo build

# 优化版本构建 (生成 Release 生产级二进制文件)
cargo build --release
```

编译出的二进制文件会保存在 `target/debug/nmon-rs` 或 `target/release/nmon-rs` 下。

---

## 命令行界面 (CLI) 选项

```text
nmon-rs: Curses based Performance Monitor (written in Rust)
Usage: monitor-rs [options]

Options:
  -f          启动 CSV 日志模式，并保存至以默认格式命名的文件。
  -F <file>   启动 CSV 日志模式，并保存至指定的路径。
  -s <secs>   设置指标收集的刷新间隔（默认：2 秒）。
  -c <count>  设置快照记录次数（默认：288 次，仅在日志模式下生效）。
  -h, -?      显示此帮助菜单并退出。
  -V          打印版本信息并退出。
```

### 命令示例：

1. **以交互式 TUI 运行**，刷新间隔设为 3 秒：
   ```bash
   cargo run -- -s 3
   ```

2. **启动 CSV 日志记录模式**，将快照写入 `my_server.nmon`，每 5 秒采集一次，共采集 60 次（累计执行 5 分钟）：
   ```bash
   cargo run -- -F my_server.nmon -s 5 -c 60
   ```

---

## 系统架构与实现细节

`nmon-rs` 的源码由以下三个核心模块组成：

- **指标收集器 (`src/metrics/`)**：
  - `cpu.rs`：收集 CPU 使用率及核心参数。
  - `memory.rs`：读取内存与 Swap 空间统计数据。
  - `disk.rs`：处理磁盘 I/O 和文件系统挂载信息。在 Linux 系统下解析 `/proc/diskstats`；在 Windows 系统下启动后台线程执行 `typeperf` 来抓取 `LogicalDisk` 性能计数器。
  - `network.rs`：记录各活动网口的 RX/TX 吞吐量。
  - `snapshot.rs`：定义序列化结构体 `MetricSnapshot`，用于统一封装所有收集到的指标。
  - `csv_logger.rs`：提供 CSV 格式化输出实现，向流中输出符合 nmon 规范的 `AAA` 元数据区及包含 `ZZZZ` 时间戳在内的各种性能数据行。

- **告警模块 (`src/alerting/`)**：
  - `rules.rs`：管理警报检查规则 `AlertRule`，设置 CPU、内存等指标的阈值条件。
  - `handler.rs`：在后台线程中异步检查指标快照，并在条件触发时将日志追加写入到 `alerts.log` 中。

- **用户界面 (`src/ui/`)**：
  - `dashboard.rs`：根据 `UiState` 中激活的面板选项动态计算布局高度与区域分配，并控制子模块的调用。
  - `*_widget.rs`：对应每个具体显示面板的小部件，利用 `ratatui` 完成界面样式的最终绘制。
  - `theme.rs`：管理基本主题配色方案。
