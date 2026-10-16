# 三国登塔录 · Three Kingdoms Spire Reborn

使用 Rust 构建的原创三国题材卡组构筑 Roguelike。参考《杀戮尖塔》系列的通用玩法循环，但不复制其专有美术、剧情、文本或原作卡牌数据库。

**当前版本：0.1.0 / 可运行文字版核心样机，并非《杀戮尖塔 2》全内容复刻。**

## 环境

- macOS (Apple Silicon / Intel)，终端支持 UTF-8
- Rust **1.99.1 stable**；根目录 rust-toolchain.toml 自动固定版本
- 首次编译需要网络下载 serde 和 serde_json

安装 Rust：https://rustup.rs/

## 快速开始

运行以下命令：

    git clone https://github.com/jackson-heylion/Three-Kingdoms-Spire-Reborn.git
    cd Three-Kingdoms-Spire-Reborn
    rustup update stable
    cargo run --release

选将：new guanyu、new zhuge、new sun、new cao、new lubu。也可输入 new guanyu 20261016 指定可复现的随机种子。

- choose 1：选择路线、奖励或营寨选项
- play 1 1：打出第 1 张手牌，目标是第 1 名敌军
- end：结束回合；use：使用回血药剂
- theme dark / theme light：暗色／亮色，记住偏好
- save / load：手动保存／读取（成功操作也自动保存）
- status / help / quit：状态、帮助、退出
- cargo run -- --plain：关闭 ANSI 控制符
- cargo run -- --save-dir ./my-save：使用自定义存档目录

存档路径：~/Library/Application Support/ThreeKingdomsSpireReborn/save.json；主题路径：同目录 settings.json。退出后再次运行会自动恢复，包括战斗中间状态。

## 当前可玩范围

五名三国武将、15 种原创卡牌、随机牌堆和抽弃牌循环、能量、敌方意图、护甲、叠加武勇、中毒、群体攻击、升级牌、消耗牌、回合结算、三幕各七层路线、普通/精英/首领战、三名关底敌人、营寨、军市、奇遇、战利品、遗物基础奖励、药剂、胜负结算、存档校验与自动保存。平衡与内容仍属首版样机。

核心引擎位于 src/lib.rs，内容位于 src/content.rs，终端仅位于 src/main.rs。所有界面通过 Game::apply(Action) 驱动一致的规则状态，序列化包含 RNG 状态，便于后续复现和跨前端接力。

## 二期接口约定

macOS 原生 GUI → SwiftUI + UniFFI/FFI → Rust 核心；响应式 HTML → Rust WASM → Rust 核心。两端传输带版本号和 revision 的同一 JSON Game 快照。跨进程同步由本地协调服务负责，不能让浏览器直接读任意本机文件。**当前尚未实现原生 GUI、HTML、实时跨端同步、多进程写入锁。** 这些模块的 API 和安全边界见 docs/ARCHITECTURE.md。

## 开发

    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test
    cargo build --release

项目里程碑和未实现功能见 docs/ROADMAP.md。
