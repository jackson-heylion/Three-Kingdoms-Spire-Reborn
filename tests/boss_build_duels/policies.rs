//! Shared deterministic duel policies, separate from Boss fixture setup and reports.
use three_kingdoms_spire_reborn::{
    Action, Game, Intent,
    content::{Effect, card_spec},
};

/// Reuse the first-playable policy semantics of tks-sim, with a hard cap on
/// zero-cost loops. This is a deterministic bot, not a human strategy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DuelPolicy { FirstPlayable, DiscardAware, GuardFirst }

pub(super) fn next_action(game: &Game) -> Action {
    next_action_with_policy(game, DuelPolicy::FirstPlayable)
}

pub(super) fn next_action_with_policy(game: &Game, policy: DuelPolicy) -> Action {
    let battle = game.battle.as_ref().expect("combat");
    if battle.played_this_turn >= 36 {
        return Action::EndTurn;
    }
    if policy == DuelPolicy::DiscardAware && battle.discard_tokens > 0 {
        // Discard the card whose effect is explicitly stronger on discard;
        // remove unplayable curses before spending energy on normal cards.
        if let Some(index) = battle.hand.iter().position(|card|
            card.id == "feishi" || card_spec(&card.id)
                .is_some_and(|spec| matches!(spec.effect, Effect::Unplayable)))
        {
            return Action::Discard { index: index + 1 };
        }
    }
    let target = battle.foes.iter().position(|foe| foe.hp > 0)
        .map_or(1, |index| index + 1);
    // A genuinely different, threat-aware ordering for every hero: play a
    // usable guard before attacking when a live foe has an attack intent.
    if policy == DuelPolicy::GuardFirst
        && battle.foes.iter().any(|foe| foe.hp > 0
            && matches!(&foe.intent, Intent::Attack(_)
                | Intent::MultiAttack(_, _) | Intent::Boss(_)))
    {
        for (index, card) in battle.hand.iter().enumerate() {
            let Some(spec) = card_spec(&card.id) else { continue };
            if !matches!(spec.effect, Effect::Guard(_)
                | Effect::GuardEnergy(_) | Effect::XGuard(_)) {
                continue;
            }
            let cost = if spec.cost == -1 { battle.energy }
                else if card.enchantment.as_deref() == Some("迅捷") {
                    (spec.cost - 1).max(0)
                } else { spec.cost };
            if cost <= battle.energy && (spec.cost != -1 || battle.energy > 0) {
                return Action::Play { index: index + 1, target };
            }
        }
    }
    for (index, card) in battle.hand.iter().enumerate() {
        let Some(spec) = card_spec(&card.id) else { continue };
        if matches!(spec.effect, Effect::Unplayable) { continue; }
        let cost = if spec.cost == -1 { battle.energy }
            else if card.enchantment.as_deref() == Some("迅捷") {
                (spec.cost - 1).max(0)
            } else { spec.cost };
        if cost <= battle.energy {
            return Action::Play { index: index + 1, target };
        }
    }
    if policy == DuelPolicy::DiscardAware
        && battle.discard_tokens > 0 && !battle.hand.is_empty()
    {
        return Action::Discard { index: battle.hand.len() };
    }
    Action::EndTurn
}


#[cfg(test)]
mod tests {
    use super::*;
    use three_kingdoms_spire_reborn::{Card, builds::PLANS};

    #[test]
    fn guard_first_really_changes_a_live_attack_decision() {
        let mut game = super::super::encounter(&PLANS[0], "zhangjiao", 0, 26_400_000, false);
        let battle = game.battle.as_mut().expect("live duel");
        battle.hand = vec![Card::new("strike"), Card::new("defend")];
        battle.energy = 2;
        battle.foes[0].intent = Intent::Attack(10);
        assert!(matches!(
            next_action_with_policy(&game, DuelPolicy::FirstPlayable),
            Action::Play { index: 1, .. }
        ));
        assert!(matches!(
            next_action_with_policy(&game, DuelPolicy::GuardFirst),
            Action::Play { index: 2, .. }
        ));
    }
}
