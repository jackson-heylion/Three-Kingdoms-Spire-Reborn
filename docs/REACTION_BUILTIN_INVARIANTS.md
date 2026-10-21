# P1 内建反制监听器不可伪造契约

四张防御牌的保留监听器占用独立槽位，不能因其 ID 被认作“内建”就绕过普通监听器配额并获得任意效果。旧版 `Stack::valid` 仅识别 ID，本次补充来源、目标、事件种类、优先级、次数及效果的完整绑定。

| 保留 ID | 事件 | 来源 | 目标 | 效果 |
| --- | --- | --- | --- | --- |
| `hold_line:weak` | StatusApplied | weak | hero | Cancel |
| `kongcheng:vulnerable` | StatusApplied | vulnerable | hero | Reduce(1) |
| `qiaoying:weak` | StatusApplied | weak | hero | Reduce(1) |
| `zhongyi:weak` | StatusApplied | weak | hero | Replace(1) |

四种监听均要求优先级 0、次数 1。合法旧 V2 快照保持兼容；伪造监听器在游戏存档验证时拒绝加载。同名非法更新不覆盖原监听。配套集成测试验证 16 个自定义监听与四种内建反制并存、非法字段拒绝和坏档拒绝。

**验收边界**：只封闭保留监听器契约，不代表完整通用事件栈、跨战斗监听、完整事件源重建、全部卡牌/敌军交互、15 套成熟构筑或 120 回异本审校完成。须以 PR 最终 SHA 的 macOS/Linux CI 为准；P1 仍未验收。
