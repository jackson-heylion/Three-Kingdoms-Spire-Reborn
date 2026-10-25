//! Independent committed-journal audit, not a replacement for tks-attest
//! or an authenticated, full-Game event-sourced restore.
use std::{env, fs};
use three_kingdoms_spire_reborn::Game;

fn run() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("usage: tks-authority <save.json>".into());
    }
    let raw = fs::read_to_string(&args[1])
        .map_err(|error| format!("cannot read save: {error}"))?;
    let game = Game::from_json(&raw)?;
    let view = game.audit_composite_authority()?;
    let report = serde_json::json!({
        "revision": view.revision,
        "rng_state": view.rng_state,
        "active_combat": view.combat.is_some(),
        "floor": view.campaign.floor,
        "stage": view.campaign.stage,
        "turn": view.combat.map(|combat| combat.turn),
        "deck_cards": view.cards.deck.len(),
        "foe_count": view.foes.as_ref().map_or(0, |foes| foes.foes.len()),
        "journal_domains": [
            "resources", "rng", "combat_resources",
            "foe_board", "card_zones", "combat_effects", "campaign", "telemetry"
        ],
        "whole_game_reconstructed": false,
        "cryptographically_authenticated": false
    });
    println!("{}", serde_json::to_string_pretty(&report)
        .map_err(|error| error.to_string())?);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("authority audit failed: {error}");
        std::process::exit(1);
    }
}
