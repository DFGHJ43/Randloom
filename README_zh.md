# 随机数生成器 (RNG)

一个基于终端界面的 C 语言随机数生成器。
零外部依赖，使用 ANSI 转义序列和 C 标准库；Windows 上统一通过 Win32 控制台 API 渲染，不涉及 ANSI/VT（Windows 10/11 开箱即用；Windows 7/8 建议在 ConEmu 等终端模拟器中运行）。

## 截图

![RNG TUI 截图](screenshot.png)

## 功能

- 三栏 TUI 布局：控制面板 | 分布图 | 结果列表
- 均匀分布：在 [min, max] 范围内生成随机整数
- 正态分布：Box-Muller 变换，可配置均值和标准差
- 伯努利分布：按概率 p 生成 0/1 结果
- 泊松分布：Knuth 算法，可配置速率参数 λ
- 实时 ASCII 概率密度/质量函数图形
- CSV 导出，文件名带时间戳
- 可指定随机种子（Seed 字段，0 = 自动），结果可复现

## 编译

```bash
gcc -Wall -Wextra -std=c99 -pedantic -Isrc -o rng-tui \
    src/tui.c src/term.c src/controls.c src/random.c \
    src/stats.c src/output.c \
    src/graph/graph.c src/graph/graph_uniform.c \
    src/graph/graph_normal.c src/graph/graph_bernoulli.c \
    src/graph/graph_poisson.c -lm
```

> 安装了 CMake 的话也可以使用项目自带的 CMakeLists.txt 构建。

## 使用

```bash
./rng-tui
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

### 参数字段

| 字段          | 含义                                   |
| ----------- | ------------------------------------ |
| Count       | 生成数量（1–1000）                        |
| Seed        | 0 = 自动（按时间播种）；非 0 = 固定种子，序列可复现      |
| Min / Max   | 均匀分布范围 `[min, max]`                  |
| Mean / StdDev | 正态分布的均值与标准差                          |
| Prob        | 伯努利分布概率 p（[0, 1]）                    |
| Lambda      | 泊松分布速率 λ                             |

> 可复现性：相同的 Seed，配合相同的分布、参数与 Count，在任何平台、任何运行次数下都会得到完全相同的序列。

## 项目结构

```
src/
├── tui.c/h           主事件循环、画面渲染、结果面板
├── term.c/h          终端控制 (Windows 用 Win32 控制台 API，其他平台用 ANSI) + 键盘输入
├── controls.c/h      左栏：参数字段、字段编辑
├── random.c/h        随机数生成 (xoshiro256** + SplitMix64)
├── stats.c/h         统计计算 (最小值、最大值、均值、标准差)
├── output.c/h        输出到控制台 / CSV 文件
└── graph/
    ├── graph.c/h             图形区域分发 (清屏 + 路由)
    ├── graph_uniform.c/h     均匀分布 PDF 图形
    ├── graph_normal.c/h      正态分布 PDF 钟形曲线
    ├── graph_bernoulli.c/h   伯努利分布 PMF 双柱图
    └── graph_poisson.c/h     泊松分布 PMF 柱状图
```

## 平台说明

- **Windows 10 / 11**（cmd.exe、Windows Terminal、ConEmu 等其他控制台主机）：统一通过 Win32 控制台 API 渲染——单一代码路径，不涉及 ANSI/VT，无需终端模拟器。
- **Windows 7 / 8**：原生控制台窗口（cmd.exe 的 conhost）对全屏 TUI 存在渲染问题（视口滚动、残影），建议在终端模拟器中运行。推荐 **ConEmu** 或 **Cmder**——两者均免费且支持 Windows 7（Windows Terminal 需要 Windows 10，7/8 上不可用）。任何能正确处理 Win32 控制台 API 的终端主机均可。
- **非 Windows**（Linux/macOS）：照常使用 ANSI 转义序列。

## 协议

MIT
