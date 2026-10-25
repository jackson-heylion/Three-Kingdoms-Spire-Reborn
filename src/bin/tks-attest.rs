//! Read-only, independently anchored Game revision audit.
//! Save and trusted checkpoint must be held in different trust domains.
use std::{env, fs, process::ExitCode};
use three_kingdoms_spire_reborn::{Game, event_stack::RevisionCheckpoint};

const USAGE: &str = "usage: tks-attest <save.json> [revision]\n       tks-attest <save.json> --verify <trusted-checkpoint.json>\n       tks-attest <save.json> --rng-trace";

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let save_path = args.first().ok_or(USAGE)?;
    let raw = fs::read_to_string(save_path)
        .map_err(|error| format!("cannot read save {save_path}: {error}"))?;
    let game = Game::from_json(&raw)?;
    match args.as_slice() {
        [_, flag] if flag == "--rng-trace" => {
            let trace = game.reconstruct_rng_draw_attribution()?;
            println!("{}", serde_json::to_string_pretty(&trace)
                .map_err(|error| error.to_string())?);
            Ok(())
        }
        [_] | [_, _] if args.get(1).is_none_or(|arg| arg != "--verify") => {
            let revision = match args.get(1) {
                Some(raw) => raw.parse::<u64>().map_err(|_| "invalid checkpoint revision")?,
                None => game.revision,
            };
            let checkpoint = game.checkpoint_at(revision)?;
            println!("{}", serde_json::to_string_pretty(&checkpoint)
                .map_err(|error| error.to_string())?);
            Ok(())
        }
        [_, flag, trusted_path] if flag == "--verify" => {
            let trusted = fs::read_to_string(trusted_path)
                .map_err(|error| format!("cannot read trusted checkpoint {trusted_path}: {error}"))?;
            let checkpoint: RevisionCheckpoint = serde_json::from_str(&trusted)
                .map_err(|error| format!("invalid checkpoint document: {error}"))?;
            game.verify_checkpoint(&checkpoint)?;
            println!("verified revision={} sha256={}", checkpoint.revision, checkpoint.digest_sha256);
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tks-attest: {error}");
            ExitCode::FAILURE
        }
    }
}
