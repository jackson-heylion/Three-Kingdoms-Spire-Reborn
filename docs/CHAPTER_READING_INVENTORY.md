# 逐回审读材料交付库存（2026-10-23）

本清单区分 **文件已进入 main**、**正式审校台账已登记** 和 **P1 文学验收通过**。三者不能互相替代。这里只核对 GitHub 仓库中的证据文件及台账状态，不重新宣称完成原文逐段校勘。

## 当前可核实范围

| 证据层级 | 章回 | 数量 | 状态 |
| --- | --- | ---: | --- |
| 已纳入 `data/chapter_body_audit.tsv` 的单底本因果通读 | 001–011 | 11 | 台账标为「正文通读/未完成异本对校」 |
| 已合入 main、尚未更新正式台账的单底本审读材料 | 012–016、032–040 | 14 | 章节文件存在；TSV 仍为「未完成全文审校」 |
| 本轮补齐的单底本审读材料（已计入上方11回） | 011 | 1 | 第011回固定底本通读及台账登记已补；不重复计数 |
| 异本逐段对校及《三国志》、裴注等史料核证 | — | 0/120 | 未验收 |

合计 **25 回有单底本审读材料**，但其中 14 回尚未进入正式台账；**0/120 回完成异本与史料逐段校勘**。第033回目前只有10项因果摘要，不能冒充此前提到的更详尽审读稿；须先找回原稿并逐项核实，才可替换或扩充。

## 新近合入 main 的证据文件

第012–016回：[012](CHAPTER_012_READING.md) · [013](CHAPTER_013_READING.md) · [014](CHAPTER_014_READING.md) · [015](CHAPTER_015_READING.md) · [016](CHAPTER_016_READING.md)。

第011回：[011](CHAPTER_011_READING.md)（本轮补齐、已登记）。第032–040回：[032](CHAPTER_032_READING.md) · [033（摘要）](CHAPTER_033_READING.md) · [034](CHAPTER_034_READING.md) · [035](CHAPTER_035_READING.md) · [036](CHAPTER_036_READING.md) · [037](CHAPTER_037_READING.md) · [038](CHAPTER_038_READING.md) · [039](CHAPTER_039_READING.md) · [040](CHAPTER_040_READING.md)。

这些文件分别由 [PR #60](https://github.com/jackson-heylion/Three-Kingdoms-Spire-Reborn/pull/60)、[PR #58](https://github.com/jackson-heylion/Three-Kingdoms-Spire-Reborn/pull/58)、[PR #61](https://github.com/jackson-heylion/Three-Kingdoms-Spire-Reborn/pull/61)、[PR #62](https://github.com/jackson-heylion/Three-Kingdoms-Spire-Reborn/pull/62) 交付。第032–040回详细交付清单另见 [CHAPTERS_032_040_DELIVERY.md](CHAPTERS_032_040_DELIVERY.md)。

## 后续处理顺序与准入

1. 第011回已核对固定底本全文及第012回开头，但异本与史料核证仍须独立开展；不将单底本通读视作全校勘。
2. **再核实第033回详稿**：比较现有摘要与原始材料，不重复计数，不覆盖已有事实边界。
3. 对已入库的14回逐回复核底本修订号、全文覆盖、叙事因果和来源后，**分别**更新 `data/chapter_body_audit.tsv` 的 `body_review_status`、`body_evidence`，并同步 [正式台账](CHAPTER_BODY_AUDIT.md)。不可因为文件存在就批量晋级。
4. 跨版本异文与史料逐段对照需独立证据；游戏分支设计不算实装。涉及可玩剧情须另有 Rust/BDD 回归和最终 macOS/Linux CI。

本清单是交付库存盘点，**不是**120回文学P1验收；仅第011回经本轮独立核读更新状态，其他14回仍待复核，游戏规则未改。
