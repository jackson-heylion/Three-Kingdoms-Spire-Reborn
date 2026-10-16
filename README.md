# 三国登塔录 · Three Kingdoms Spire Reborn

以 Rust 1.99.1 Stable 编写的原创三国题材文字版卡组构筑 Roguelike。参考经典爬塔卡牌游戏的通用玩法框架，采用原创角色机制、卡牌命名、敌人、奇遇与背景，不复制《杀戮尖塔 2》专有的成套卡牌文字、故事或美术。

> **当前状态：增强版单人终端 Alpha。不是《杀戮尖塔 2》全部内容的完成品。**
> 核心机制与内容已大幅扩展，但仍有地图图谱、完整状态触发、职业流派、合作模式与数值平衡等未达成项目。详见 [功能对照表](docs/FEATURE_MATRIX.md) 和 [开发路线](docs/ROADMAP.md)。

## 安装与运行

仅面向 macOS 终端提供正式操作路径（CI 同时检查 Linux）。

```bash
# 如未安装 Rust，请先访问 https://rustup.rs/
git clone https://github.com/jackson-heylion/Three-Kingdoms-Spire-Reborn.git
cd Three-Kingdoms-Spire-Reborn
cargo run --release --bin tks
```

## 命令

```text
new guanyu 20261016 0          # 关羽 / 种子 / 已解锁飞升难度
custom guanyu 20261016 10     # 自由模式，可测试任意0-10难度
variant 1 0 1                  # 出征前选择三幕的替代章节
new zhuge                    # 诸葛亮
new sun                      # 孙尚香
new cao                      # 曹操
new lubu                     # 吕布
daily 2026-10-16             # 按日期固定的原创每日挑战
choose 1                     # 选择路线、奇遇、商店、奖励、先贤赐福
play 1 1                     # 使用第 1 手牌攻击第 1 名敌军
discard 2                    # 需先获得弃换次数
end                          # 结束回合
use 1 1                      # 使用第 1 瓶药剂，对第 1 敌军（如需目标）
deck                         # 查看牌组与编号
map                          # 查看本幕七层三路图与可达节点
upgrade 3                    # 在营寨升级指定卡牌
enchant 3 迅捷               # 在营寨花35金附魔
remove 2                     # 在军市花75金移除指定卡牌
relics                       # 查看遗物
bonds                        # 当前人物羁绊和战斗加成
lore                         # 查看演义事件、伏笔与已解锁彩蛋
chapter 50                   # 120回原著回目索引与对应游戏剧情
chronicle                    # 战功簿、解锁、图鉴、最近战局
theme light                  # 亮色终端
theme dark                   # 暗色终端
save / load / status / help / quit
```

**三国叙事**：参考毛宗岗本120回目录建立章回索引，新增28则有前置条件的原创演义奇遇、25条关系羁绊、8项需完成整条选择链的隐藏结局。场景选择与战斗资源、金钱、体力、后续事件、JSON存档关联。支持 `lore`、`bonds`、`chapter N` 查询文学出处。未完成120回全文逐回审校，详见 [小说资料审定](docs/NOVEL_STUDY.md)。

**可玩内容**：五名武将各自的战斗资源与被动体系、95 张原创卡牌、31 类敌人（包含替代精英和六名关底敌军）、10 种药剂、16 种遗物、12 类分支事件、12 种先贤赐福、三幕 21 层连通三路战役地图、6名关底统帅的二阶段变化、敌人意图、单/群体伤害、护甲、虚弱、易伤、中毒、武勇、反击、亲兵扛伤与自动攻击、弃换触发、牌组抽弃/消耗、特殊任务牌探索变形、升级与附魔、军市和营寨、战功簿与每日挑战、标准难度逐级解锁和自由挑战难度。

每次成功动作保存完整快照，包括手牌、弃牌堆、RNG、职业资源和敌军意图。失败指令会回滚完整状态，不改变随机数或修订号。V1 快照可迁移到 V2。

## 存档

macOS `~/Library/Application Support/ThreeKingdomsSpireReborn/`

- `save.json`：当前游戏与回合状态（跨界面共享协议基础）。
- `chronicle.json`：长期战功、发现记录与难度解锁进度。
- `settings.json`：明暗主题偏好。

```bash
cargo run --bin tks -- --plain
cargo run --bin tks -- --save-dir ./custom-saves
```

**注意**：当前没有多进程写锁，暂时不要同时启动两个终端窗口操作同一存档。原生 App、响应式 HTML 和本机会话协调服务均属二期，尚未实现。

## 开发与测试

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --test bdd -- --nocapture
cargo build --release
cargo run --release --bin tks-sim -- 1000
```

每次更新由 GitHub Actions 在 macOS + Linux 下编译、测试、终端存档恢复、长期战功簿检查与每平台1000局种子自动模拟上进行验证。CI 状态：[GitHub Actions](https://github.com/jackson-heylion/Three-Kingdoms-Spire-Reborn/actions)。

## 技术边界

- `src/lib.rs`：唯一权威规则状态机与 `Game::apply(Action)`，含操作事务回滚、随机数和存档。
- `src/content.rs` / `src/items.rs`：原创卡牌、敌军、遗物、药剂、战役赐福。
- `src/progression.rs`：长期战功与图鉴。
- `src/main.rs`：Mac 终端呈现与输入。
- `src/bin/tks-sim.rs`：自动对局和确定性回归模拟。

二期统一用同一 Rust 核心驱动 SwiftUI 与 WASM/HTML，协调服务保证单写者与修订号冲突检测。详情见 [架构](docs/ARCHITECTURE.md)。
