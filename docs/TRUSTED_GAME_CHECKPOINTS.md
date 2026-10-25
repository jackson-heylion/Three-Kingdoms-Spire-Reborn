# P1：可外部锚定的完整 Game 修订检查点

## 解决的问题

既有 `Game::reconstruct_game_at` 和动作→事件因果重放能够发现单独篡改的存档，但若攻击者同时改写动作日志、种子和最终快照，并重新生成全部日志，本地一致性检查仍可能通过。新增 **SHA-256 版本检查点**：从完整动作重放产生指定修订号的 Game，使用域隔离 `tks/game-checkpoint/v1\\0` 对完整确定性 JSON 计算摘要，交给**独立可信存储**保存。以后从不可信存档重建同一修订号，与外部可信摘要比较；仅改写本地存档不能伪造原先已外部锚定的历史。

## 终端使用

先构建 `cargo build --bin tks-attest`。以下假设 `save.json` 是真实战役存档。

```sh
# 输出当前修订号的 JSON 检查点。必须保存在独立可信位置。
target/debug/tks-attest /path/to/save.json > /trusted-storage/run-checkpoint.json

# 也可指定历史修订号，0 表示最初完整 Game。
target/debug/tks-attest /path/to/save.json 0 > /trusted-storage/run-start.json

# 重新载入存档，验证完整重放与外部锚点，失败返回非零退出码。
target/debug/tks-attest /path/to/save.json --verify /trusted-storage/run-checkpoint.json
```

`RevisionCheckpoint` 包含 `schema_version=1`、`revision`、`hero`、`seed`、`run_id` 和 `digest_sha256`。API：`Game::checkpoint_at(revision)`、`Game::verify_checkpoint(&checkpoint)`，只读，不改变 RNG、存档或动作日志。历史修订号必须真实存在；旧档缺完整动作日志、动作日志截断、当前快照与重放不一致、检查点版本错误或摘要不匹配均拒绝。

## 验证和信任边界

- `tests/revision_checkpoint.rs`：五职业×飞升0/10×四个修订号；跨序列化往返、有效但重写的路线、改种子、篡改快照、截断日志、旧档缺日志、错误版本/摘要。
- `tests/checkpoint_cli_e2e.py`：真实 `tks` 与 `tks-attest` 进程；外部文件验证、历史锚点、修改快照和合法替换战役拒绝；在 macOS 14、Ubuntu 独立 CI 中执行。
- **前提**：外部锚点须先在可信环境生成，且其完整性独立保护；把锚点和存档放在同一可改目录、让攻击者重算锚点，不提供真实性保证。首次导出仅建立信任起点（trust on first use），不能证明此前未被篡改。
- **局限**：此增量提供有外部信任前提的完整 Game 修订认证，不是以事件独立归约出 Game 的事件源架构；没有逐次 RNG 抽样调用点、事件内部父子边、跨版本规则迁移、外部签名/时间戳服务、密钥管理或自动远端锚定。当前 SHA-256 输入是版本1规则序列化，未来改变 JSON/规则可能需要版本化迁移。历史检查点验证会先验证当前整局动作重放，成本随历史增长，不应用于 TUI 每帧。
- P1 仍欠成熟构筑、全效果交互、120回正文与异本校勘、真人试玩、完整国际化，不能发布1.0。
