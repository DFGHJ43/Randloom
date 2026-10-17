# Randloom

Randloom 是一个用 Rust 编写的交互式终端随机数生成器。它保留了 C 版的三栏 TUI、
分布图、种子复现与 CSV 导出，同时把平台层替换为
[crossterm](https://crates.io/crates/crossterm)，时间戳处理使用
[chrono](https://crates.io/crates/chrono)。随机数引擎是自带实现的
xoshiro256**，项目 `src/` 中不含 `unsafe` 代码。

## 截图

![Randloom TUI 截图](screenshot.png)

## 功能

- 三栏 TUI 布局：控制面板 | 分布图 | 结果列表
- 均匀分布：在 [min, max] 范围内生成随机整数
- 正态分布：Box-Muller 变换，可配置均值和标准差
- 伯努利分布：按概率 p 生成 0/1 结果
- 泊松分布：λ <= 10 使用 Knuth 算法，λ > 10 使用 PTRS 拒绝采样，可配置速率参数 λ
- 实时绘制概率密度/质量函数图形，使用 Unicode 块元素（█、▀、▄）与制表符坐标轴，
  曲线平滑、填充实心不呈阶梯状
- CSV 导出，文件名带时间戳
- 可指定随机种子（Seed 字段，0 = 自动），结果可复现

### 图形渲染

图形使用“半块”画布：每个字符格包含上下两个半格，用 `█`（全块）、`▀`（上半块）、
`▄`（下半块）渲染。相当于在字符网格上获得 1×2 的纵向分辨率——比纯 `#` 字符画
纵向清晰一倍，同时保持实心填充。坐标轴使用制表符 `│`、`─`、`└`、`→`，边框使用
`┌ ┬ ┐ ├ ┤ └ ┴ ┘` 系列；刻度标签与数值提示仍为 ASCII。

如果终端无法显示 Unicode，可设置 `RANDLOOM_ASCII=1`，边框退化为 ASCII 的
`+ - |`，图形格子退化为 `#`。

## 编译

需要 Rust 1.85 或更新版本（edition 2024）。

```bash
cargo build --release
```

运行：

```bash
target/release/randloom        # Linux / macOS
target\release\randloom.exe    # Windows
```

## 开发

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

UI 快照使用 [insta](https://crates.io/crates/insta)。修改渲染后：

```bash
cargo install cargo-insta   # 只需安装一次
cargo insta review
```

`tests/fixtures/` 中的 golden 数据由 C 参考实现生成，具体命令见
`tests/fixtures/README.md`。

Windows 上可使用 MSVC 构建工具或 GNU 工具链；CI 在 `windows-latest` 上使用
stable MSVC 工具链。若机器只有 MinGW、没有 MSVC linker：

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnu
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-gnu"
cargo run --release
```

请在真实交互式终端中运行 TUI；重定向 stdin/stdout 会被拒绝。

## 使用

```bash
./randloom
```

### 按键说明

| 按键          | 功能        |
| ----------- | --------- |
| `G`         | 生成随机数     |
| `E`         | 导出结果为 CSV |
| `U`         | 切换到均匀分布   |
| `N`         | 切换到正态分布   |
| `B`         | 切换到伯努利分布  |
| `P`         | 切换到泊松分布   |
| `Tab`       | 切换输入框焦点   |
| `↑` `↓`     | 滚动结果列表    |
| `←` `→`     | 在输入框中移动光标 |
| `Backspace` | 删除输入框中的字符 |
| `Q` / `Esc` | 退出        |
| `Ctrl+C`    | 退出并恢复终端   |

### 参数字段

| 字段          | 含义                                   |
| ----------- | ------------------------------------ |
| Count       | 生成数量（1–1000）                        |
| Seed        | 0 = 自动（按时间播种）；非 0 = 固定种子，序列可复现      |
| Min / Max   | 均匀分布范围 `[min, max]`                  |
| Mean / StdDev | 正态分布的均值与标准差                          |
| Prob        | 伯努利分布概率 p（[0, 1]）                    |
| Lambda      | 泊松分布速率 λ                             |

> 可复现性：相同的 Seed，配合相同的分布、参数与 Count，整数序列一致。
> 正态分布依赖平台 `f64` 数学库，跨平台可能有末位差异；均匀、伯努利、泊松的整数
> 结果跨平台一致。自动模式下状态栏会显示实际使用的种子。

## 项目结构

```text
src/
├── main.rs                     二进制入口
├── lib.rs                      公开模块与 #![forbid(unsafe_code)]
├── run.rs                      终端初始化、事件循环、resize 处理
├── app.rs                      应用状态机与按键动作
├── config.rs                   参数、字段、解析与校验
├── input.rs                    与终端无关的 Key / InputEvent / Flow
├── rng.rs                      xoshiro256** + SplitMix64 引擎
├── stats.rs                    最小值 / 最大值 / 均值 / 总体标准差
├── fmt.rs                      兼容 C printf "%g" 的格式化
├── export.rs                   带时间戳的单列 CSV 导出
├── dist/                       均匀 / 正态 / 伯努利 / 泊松采样器
├── term/                       crossterm 终端层、按键映射、信号处理
└── ui/                         无终端依赖的 Frame、布局、面板与图形
    ├── frame.rs, glyphs.rs, layout.rs
    ├── chrome.rs, controls.rs, results.rs
    └── graph/                  半块画布、坐标轴、各分布图形
tests/
├── fixtures/                   来自 C 参考实现的 golden 数据
├── snapshots/                  insta UI 快照
└── *.rs                        对照、统计、配置、应用与导出测试
docs/rust-rewrite/              架构、对照清单、迁移说明
```

原 C 实现保留在 `master` 分支；Rust 重写树中不再包含 C 源码。

## 平台说明

- **Windows 10 / 11**：crossterm 为控制台启用 VT 处理，Windows Terminal 与经典
  控制台均可运行。Rust 1.78 起官方基线为 Windows 10，因此 Rust 版不支持
  Windows 7/8；如需 Win7/8，请使用 `master` 上的 C 版。
- **Linux / macOS**：照常使用 ANSI 终端。
- TUI 需要交互式 stdin/stdout；重定向输入会给出明确错误并退出。

## 依赖

| crate | 用途 | 许可证 |
| --- | --- | --- |
| `crossterm` | 跨平台终端 I/O 与事件 | MIT |
| `chrono` | 导出文件名的本地时间 | MIT OR Apache-2.0 |
| `signal-hook`（仅 Unix） | SIGINT/SIGTERM 优雅退出 | MIT OR Apache-2.0 |
| `insta`（仅开发） | UI 快照测试 | Apache-2.0 |

随机数引擎与全部分布均在本仓库实现，不依赖随机数 crate；`src/` 中无 `unsafe`
（`unsafe_code = "forbid"`）。

## 协议

MIT
