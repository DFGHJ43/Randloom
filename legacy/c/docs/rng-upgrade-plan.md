# RNG 引擎升级 + 种子复现功能 — 实施指导书

> 版本：v2（合并两阶段改动，含完整技术细节）
> 状态：**已实施**（2026-08-14）。本文档作为实施记录保留。
> 实施说明：因一次性实施，`src/random.c` 直接采用最终版（`rng_init(uint64_t)`），
> 与 `src/random.h` 同步修改；"三个 commit"的切分由提交者按内容自行决定
> （建议：代码一个 commit + README 一个 commit）。第四部分验证方案已全部实测通过。

---

## 目录

- [第一部分 背景与决策](#第一部分-背景与决策)
- [第二部分 阶段一：xoshiro256\*\* 引擎替换](#第二部分-阶段一xoshiro256-引擎替换)
- [第三部分 阶段二：Seed 手动种子字段（可复现功能）](#第三部分-阶段二seed-手动种子字段可复现功能)
- [第四部分 验证方案](#第四部分-验证方案)
- [第五部分 风险与回滚](#第五部分-风险与回滚)
- [第六部分 文件改动清单与实施顺序](#第六部分-文件改动清单与实施顺序)
- [第七部分 验收清单](#第七部分-验收清单)

---

# 第一部分 背景与决策

## 1.1 现状问题

`src/random.c` 基于 C 标准库 `rand()`。`rand()` 质量是实现定义的：

| 平台 | 问题 |
| --- | --- |
| Windows / MSVC | `RAND_MAX = 32767`（15 位）；底层 LCG `state*214013+2531011` 的低 15 位有周期相关模式 |
| Linux / glibc | 同样是质量较差的 LCG |

对本项目的具体影响：

1. **正态分布量化**：`rng_normal()` 中 `rand()/RAND_MAX` 只有 32768 个离散值，Box-Muller 输入被量化，结果有可见颗粒感。
2. **伯努利精度不足**：p 的有效分辨率仅 1/32768。
3. **64 位池拼接相关性**：`rng_raw64()` 把 5 个 15 位块拼成 64 位，反复拼入 LCG 低位周期模式。
4. 泊松 PTRS 分支同样受 `RAND_MAX` 精度限制。

## 1.2 候选对比与决策

| 方案 | 质量 | 代码量 | 跨平台/零依赖 | 结论 |
| --- | --- | --- | --- | --- |
| `rand()`（现状） | 差（实现定义） | — | ✓ | 淘汰 |
| **xoshiro256\*\*** | 通过 TestU01 BigCrush | ~20 行 | ✓ 公有领域，纯 C99 | ⭐ 采用 |
| PCG32/64 | 很好 | ~25 行 | ✓ | 备选 |
| MT19937 | 良好 | ~50 行 + 2.5KB 状态 | ✓ | 偏重、播种慢 |
| 系统 CSPRNG | 密码学级 | 每平台一份 `#ifdef` | ✗ | 过度设计 |

**决策**：xoshiro256\*\*（Blackman & Vigna，2018，公有领域，官网 prng.di.unimi.it）。周期 2²⁵⁶−1、通过 BigCrush、比 `rand()` 更快、实现极小，与本项目"零依赖 + C99 + 跨平台 + 学习型"定位完全一致。

## 1.3 本次实施范围

两个正交的功能，分两个 commit：

- **阶段一**：引擎替换（只动 `src/random.c`）。
- **阶段二**：Seed 手动种子字段，兑现 README"结果可复现"承诺（动 `tui.h/c`、`controls.c`、`random.h`、README）。

---

# 第二部分 阶段一：xoshiro256\*\* 引擎替换

## 2.1 算法家族与变体选择

- 名字拆解：`xoshiro` = **xo**r + **shi**ft + **ro**tate；`256` = 状态 4×64 位；`**` = 输出端带乘法扰频。
- 同族还有 xoshiro256+：它的低 1~3 位是线性的，会把弱点泄露给"取低位"的调用者。`**` 在输出前做乘法扰频打散线性规律，是官方推荐的全能变体。本项目 `rng_uniform` 取模恰好用低位，故**必须选 `**`**。
- 原理一句话：状态在 GF(2) 上做线性变换（异或 + 移位 + 旋转，周期 2²⁵⁶−1），输出前经非线性扰频（两次乘法 + 一次旋转），外部看不出线性结构。

## 2.2 核心代码（逐行解读）

```c
#include <stdint.h>          /* uint64_t（新增） */

static uint64_t s[4];                              /* 4×64 位内部状态 */

static inline uint64_t rotl64(uint64_t x, int k) {
    return (x << k) | (x >> (64 - k));             /* 循环左移 k 位 */
}

static uint64_t next_u64(void) {
    uint64_t result = rotl64(s[1] * 5, 7) * 9;     /* ① 输出扰频：s[1] 乘5、转7位、乘9 */
    uint64_t t = s[1] << 17;                       /* ② 状态更新（5 步线性变换） */
    s[2] ^= s[0];
    s[3] ^= s[1];
    s[1] ^= s[2];
    s[0] ^= s[3];
    s[2] ^= t;
    s[3] = rotl64(s[3], 45);
    return result;
}
```

要点：

- **输出与状态分离**：先取 `result` 再更新状态，调用者拿到的值与下一状态"看起来"无关。
- **两次乘法的意义**：乘法让低位参与高位、高位参与低位（进位传播），这是通过 BigCrush 的关键。
- **代价**：每次约 6 次异或 + 2 次旋转 + 2 次乘法，x86-64 上几纳秒，**比 MSVC `rand()` 更快**，性能只升不降。

## 2.3 播种：SplitMix64（为什么不能随便填）

- xoshiro 的**全零状态是吸收态**（永远输出 0），`s[0..3]` 不能全 0；直接填 4 个相同值也会让状态退化。
- 官方推荐 SplitMix64 把任意 64 位种子搅拌成 4 个高质量状态字：

```c
static uint64_t splitmix64(uint64_t *x) {
    uint64_t z = (*x += 0x9e3779b97f4a7c15ULL);    /* 黄金比例常数 */
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}

void rng_init(unsigned int seed) {                 /* 阶段一保持旧签名 */
    if (seed == 0) seed = (unsigned int)time(NULL);
    uint64_t x = (uint64_t)seed;
    for (int i = 0; i < 4; i++) s[i] = splitmix64(&x);
}
```

- `rng_init(0)` 语义不变：用当前时间随机播种；非 0 种子两次运行序列完全一致。
- **种子空间说明**：64 位种子最多对应 2⁶⁴ 个初始状态，小于 2²⁵⁶ 总空间，但 2⁶⁴ 对非密码用途绰绰有余，**不是缺陷**。
- 可选增强：`x` 再异或 `clock()` 与栈变量地址，降低"同一秒两次启动同种子"概率。默认不做，保持简单。

## 2.4 满 53 位精度浮点

```c
/* [0,1) 闭开区间：高 53 位 = 双精度最大有效位数，无损失 */
static double next_double(void) {
    return (double)(next_u64() >> 11) * 0x1.0p-53;
}

/* (0,1) 开区间：PTRS 分支专用（避免 log(0)） */
static double next_double_open(void) {
    return ((double)(next_u64() >> 11) + 0.5) * 0x1.0p-53;
}
```

- `0x1.0p-53` 是 C99 十六进制浮点字面量，MSVC 与 GCC 均支持。
- 效果：正态/伯努利的取值空间从 32768 提升到 2⁵³，量化问题彻底消失。

## 2.5 逐函数改造映射

| 现状（random.c） | 改造后 | 说明 |
| --- | --- | --- |
| `rng_raw64()`：5×15bit 拼接 | **整个删除** | 引擎自带 64 位 |
| `rng_uniform()`：64 位池 + 拒绝采样 | 池来源换 `next_u64()`，拒绝采样保留 | 现逻辑已被 bugcheck 验证无偏，只换"水源" |
| `rng_normal()`：Box-Muller | 算法不动；`u1 = next_double()`（循环到 >0），`u2 = next_double()` | 输入满 53 位 |
| `rng_bernoulli()` | `next_double() < p`，clamp 保留 | 一行 |
| `rng_poisson()` Knuth 分支 | `p *= next_double()` | 算法不动；u=0 概率从 1/32768 降到 2⁻⁵³ |
| `rng_poisson()` PTRS 分支 | `u`、`v` 用 `next_double_open()` | 常数与判据原样 |

**头文件变化**：新增 `<stdint.h>`；`RAND_MAX` 引用全部消失；`<limits.h>`（`INT_MAX` clamp）、`<math.h>`、`<time.h>` 保留；`#define M_PI` 保留。

**已知小瑕疵（本次不改）**：Box-Muller 一次算两个正态数 z0、z1，代码只用 z0 丢弃 z1，浪费一半计算。缓存 z1 会改变调用语义，学习项目不值得，保持现状即可。

**质量边界**：

- ✅ 统计质量：BigCrush 全套通过、周期 2²⁵⁶−1、等分布优秀——科学计算、模拟、抽样完全够。
- ❌ **非密码学安全**：知道连续输出可反推状态、可预测未来。未来若生成密码/密钥需换系统 CSPRNG，另立项。
- 单线程专用：全局 4 变量状态，本项目无多线程需求，不引入锁。

---

# 第三部分 阶段二：Seed 手动种子字段（可复现功能）

## 3.1 语义设计

| 种子值 | 行为 |
| --- | --- |
| `0`（默认） | 自动模式：每次按 `G` 用当前时间播种，与现状一致 |
| 非 0 | 固定模式：每次按 `G` 产生**完全相同**的序列（可复现） |

关键细节：**自动模式下把实际使用的种子显示在状态栏**，用户想复现某次结果时，把状态栏里的数字输回 Seed 字段即可。

## 3.2 UI 设计：新增 `FIELD_SEED`

`src/tui.h`：

```c
#define FIELD_COUNT    8          /* 原 7 */

typedef enum {
    FIELD_COUNT_NUM = 0,
    FIELD_SEED,                   /* 新增，紧跟在 Count 后 */
    FIELD_MIN,
    FIELD_MAX,
    FIELD_MEAN,
    FIELD_STDDEV,
    FIELD_PROB,
    FIELD_LAMBDA
} FieldId;

/* TuiState 新增成员 */
unsigned long long seed;          /* 0 = 自动 */
```

- 枚举插入 `FIELD_COUNT_NUM` 之后：屏幕顺序与 Tab 焦点顺序都是 **Count → Seed → 分布参数**，Seed 紧跟 Count（两者都是全局参数，语义正确）。
- `tui_init()`：`state->seed = 0;`，`snprintf(field_buf[FIELD_SEED], ..., "0");`，cursor 初始化循环自动覆盖（`FIELD_COUNT` 已变 8）。

## 3.3 输入过滤（`tui.c` default 分支）

Seed 字段**只接受数字 `0-9`**，拒绝 `-` 与 `.`——避免 `strtoull("-5")` 卷成 ULLONG_MAX 的反直觉行为：

```c
default:
    if (state.focus_field == FIELD_SEED) {
        if (key >= '0' && key <= '9')
            field_insert(FIELD_SEED, &state, (char)key);
    } else if ((key >= '0' && key <= '9') || key == '-' || key == '.') {
        field_insert((FieldId)state.focus_field, &state, (char)key);
        parse_fields(&state);
        if (field_is_visible(state.focus_field, state.dist))
            state.graph_dirty = 1;
    }
    break;
```

## 3.4 解析（`controls.c` parse_fields）

```c
state->seed = strtoull(state->field_buf[FIELD_SEED], NULL, 10);
```

- 空串 / 非法字符 → 返回 0 → 自动模式，无需额外钳制。
- 超过 20 位数字 → `strtoull` 饱和到 `ULLONG_MAX`，行为安全。
- 需要 `#include <stdlib.h>`（已有）。

`field_is_visible()` 改为：

```c
if (fid == FIELD_COUNT_NUM || fid == FIELD_SEED) return 1;   /* 恒可见 */
/* 其余按 dist 判断，原逻辑不变 */
```

`labels[]` 增加一项 `"Seed:   "`（与其余标签 8 字符宽对齐）。

## 3.5 生成逻辑（`tui.c` do_generate）

```c
unsigned long long seed = state->seed;
if (seed == 0) seed = (unsigned long long)time(NULL);
rng_init(seed);                 /* 阶段二签名已拓宽，见 3.6 */

/* ……四个分布的生成循环不变…… */

snprintf(state->status, sizeof(state->status),
         "Gen %d %s numbers. Seed %llu%s.", n, dist_name, seed,
         state->seed == 0 ? " (auto)" : "");
```

`tui_init()` 末尾的 `rng_init((unsigned int)time(NULL))` 改为 `rng_init(0)`（语义一致，自动播种）。

## 3.6 `rng_init` 签名拓宽为 64 位

`src/random.h`（加 `#include <stdint.h>`）：

```c
void rng_init(uint64_t seed);   /* 原 unsigned int */
```

`src/random.c` 中 `seed == 0` 时取 `time(NULL)` 的逻辑不变，`uint64_t x = seed;` 直接进 SplitMix64。调用点仅 `tui.c` 两处（`tui_init`、`do_generate`），一并更新。

## 3.7 布局碰撞分析与修复（最容易翻车的点）

当前 `draw_controls()` 几何（`y = 3`）：

- 行 3：`Distribution:` 标题；行 4–7：四个分布单选。
- `ry = y + 7`（行 10）起画 7 个字段 → 行 10–16。
- 按钮固定 `by = y + 14`（行 17–19）：`[ Generate ]` / `[ Export ]` / `[ Quit ]`。

**碰撞推导**：字段加到 8 个后，字段块为行 10–17，第 8 个字段（Seed）画在行 17；随后按钮绘制会**覆盖同一行**——Seed 字段被 `[ Generate ]` 盖掉。若把按钮下移，22 行最小窗口下会撞上状态栏（`status_y = rows - 2 = 20`），同样不行。

**修复**：字段起点 `y + 7` → **`y + 6`**（行 9 起画 8 行，结束于行 16），按钮保持行 17–19 不动。数学核对：`TERM_MIN_ROWS = 22` 时状态栏行 20，按钮清理循环 `row < term_rows - 1` 到行 20 为止，无越界；`tui.h` 头部注释"rows 3-19"仍成立。这是阶段二**唯一**的布局改动。

## 3.8 可复现性保证范围

- 同一 seed + 同一程序 → **任何平台、任何运行次数，序列逐位一致**（xoshiro 全整数运算，无浮点、无平台依赖）。
- 复现前提：**相同分布 + 相同参数 + 相同 count**——状态栏与 README 中说明即可。
- Seed 输入 `007` = 7、`000` = 0（自动），strtoull 语义自然成立，无需特判。

## 3.9 README 同步

- `README.md` / `README_zh.md`：键位表不变（Seed 是字段不是键）；在字段说明处补 "Seed（0 = 自动，非 0 = 固定种子，结果可复现）"；使"可指定随机种子，结果可复现"承诺变为真实。

---

# 第四部分 验证方案

## 4.1 回归（build/bugcheck.c，本地工具，gitignored）

现有三模式（`crash` / `poisson` / `layout`）重新编译跑通——API 语义未变，应全部通过。**新增 `seq` 模式**：

```
用法: bugcheck seq [seed]
功能: rng_init(seed) 后打印前 10 个 rng_uniform(0,100)（整数），供跨工具链对比
```

同一 seed 在 **MSVC 与 MinGW GCC 两套工具链**下输出必须逐行相同（验证跨平台复现性，见 4.3）。

## 4.2 统计抽查（一次性小工具，不必入库）

| 检查 | 方法 | 通过阈值 |
| --- | --- | --- |
| 均匀分布 | 200k 样本，10 桶卡方检验 | χ² < 25（df=9，宽松） |
| 正态分布 | n=200k | 样本均值 ±0.02 内、样本标准差 ±0.02 内（≈10×SE） |
| 伯努利 | n=200k | \|频率 − p\| < 0.005 |
| 泊松 | λ=1/10/1000 各 200k | 样本均值与 λ 偏差 < 1%（覆盖 Knuth 与 PTRS 两分支） |

## 4.3 复现性验证

1. seed=42 → 生成 → 记录 → 再按 G → 结果完全一致；换 seed=7 结果变化；换回 42 复原。
2. seed=0 连按两次 G → 序列不同，且状态栏显示的 Seed 不同（自动模式证据）。
3. 边界输入：空字段（=0 自动）、`0`、`18446744073709551615`、尝试输入 `-`/`.`（应被过滤）。
4. 跨工具链：`bugcheck seq 42` 在 MSVC / MinGW 下输出一致。

## 4.4 编译矩阵

- MSVC（CLion/Visual Studio toolchain）与 MinGW GCC 各 `-Wall -Wextra -std=c99 -pedantic` **零警告**。
- 命令行 gcc 构建命令不变（文件列表未增减）。

## 4.5 TUI 人工冒烟

1. 四种分布各生成一轮，PDF 图正常。
2. Tab 焦点顺序：Count → Seed → 分布参数，Seed 在任何分布下都可见。
3. 最小窗口 60×22 下按钮 `[ Generate ]/ [ Export ]/ [ Quit ]` 不被字段压住。
4. 导出 CSV 正常。

---

# 第五部分 风险与回滚

| 风险 | 等级 | 说明 |
| --- | --- | --- |
| 布局碰撞（阶段二） | 中 | 已用 3.7 的推导修复；冒烟测试 4.5.3 兜底 |
| 状态全零吸收态 | 低 | SplitMix64 播种天然避免 |
| `uint64_t` 可移植性 | 低 | `<stdint.h>` 是 C99 标准，MSVC/GCC 均支持 |
| 种子溢出 | 低 | `strtoull` 饱和 + 输入过滤，无 UB |
| 状态栏长度 | 低 | Seed 最多 20 位数字 + 前缀，128 字节 buffer 充裕 |

**回滚**：阶段一 `git checkout -- src/random.c`；阶段二按 commit 粒度 `git revert`。动手前先 `git commit` 留检查点（工作树须干净）。

---

# 第六部分 文件改动清单与实施顺序

| 文件 | 阶段一 | 阶段二 |
| --- | --- | --- |
| `src/random.c` | ✅ 引擎替换 | 仅 `rng_init` 签名 |
| `src/random.h` | — | ✅ `uint64_t` 签名 + `<stdint.h>` |
| `src/tui.h` | — | ✅ `FIELD_SEED`、`FIELD_COUNT=8`、`TuiState.seed` |
| `src/tui.c` | — | ✅ 初始化、输入过滤、`do_generate`、`tui_init` 播种 |
| `src/controls.c` | — | ✅ 可见性、labels、parse_fields、布局起点 `y+6` |
| `README.md` / `README_zh.md` | — | ✅ Seed 字段说明 |
| `build/bugcheck.c` | — | ✅ 新增 `seq` 模式 |

**实施顺序**（每个阶段单独 commit，各自可独立回滚）：

1. **Commit 1**：阶段一——引擎替换（只动 `src/random.c`，API 不变）。
2. **Commit 2**：阶段二——Seed 字段（上述其余文件）+ bugcheck `seq` 模式。
3. **Commit 3**：README 双语文档同步。

---

# 第七部分 验收清单

- [ ] bugcheck 三旧模式 + 新 `seq` 模式全部通过
- [ ] 4.2 四项统计抽查全部达标
- [ ] seed=42 两次生成结果逐位一致；seed=0 每次不同且状态栏可见种子
- [ ] MSVC 与 MinGW 双工具链零警告构建；`seq 42` 输出一致
- [ ] 60×22 最小窗口布局无碰撞，四分布冒烟正常
- [ ] 工作树干净，3 个 commit 边界清晰，随时可回滚
