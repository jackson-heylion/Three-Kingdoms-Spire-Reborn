//! Cryptographic Game checkpoints anchored OUTSIDE the mutable save directory.
//! SHA-256 protects an exact versioned serialization, not the trustworthiness
//! of an unanchored local file or future changes to the game rules.
use crate::{Game, content::Hero};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DOMAIN: &[u8] = b"tks/game-checkpoint/v1\0";
const CHECKPOINT_VERSION: u32 = 1;

/// Export this record to independent, trusted storage. The save file alone
/// cannot authenticate a checkpoint kept beside it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionCheckpoint {
    pub schema_version: u32,
    pub revision: u64,
    pub hero: Hero,
    pub seed: u64,
    pub run_id: String,
    pub digest_sha256: String,
}

fn digest(game: &Game) -> Result<String, String> {
    let json = game.to_json()?;
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    hash.update(json.as_bytes());
    Ok(format!("{:x}", hash.finalize()))
}

impl Game {
    /// Reconstruct an exact historical Game revision and compute its SHA-256.
    /// First verify the entire current snapshot against committed actions, so
    /// no modified live snapshot is silently endorsed by the export operation.
    /// This is an offline audit; it replays game rules and is not a hot path.
    pub fn checkpoint_at(&self, revision: u64) -> Result<RevisionCheckpoint, String> {
        if revision > self.revision {
            return Err("checkpoint revision exceeds current game revision".into());
        }
        let current = self.reconstruct_game_at(self.revision)?;
        if current.to_json()? != self.to_json()? {
            return Err("checkpoint export refused: saved Game differs from action replay".into());
        }
        let historical = if revision == self.revision {
            current
        } else {
            self.reconstruct_game_at(revision)?
        };
        Ok(RevisionCheckpoint {
            schema_version: CHECKPOINT_VERSION,
            revision,
            hero: self.hero,
            seed: self.seed,
            run_id: self.run_id.clone(),
            digest_sha256: digest(&historical)?,
        })
    }

    /// Compare reconstructed state to an externally trusted checkpoint.
    /// A successful result authenticates this exact revision only if the
    /// caller independently trusts the supplied checkpoint bytes.
    pub fn verify_checkpoint(&self, checkpoint: &RevisionCheckpoint) -> Result<(), String> {
        if checkpoint.schema_version != CHECKPOINT_VERSION {
            return Err("unsupported Game checkpoint schema".into());
        }
        if checkpoint.digest_sha256.len() != 64
            || !checkpoint.digest_sha256.bytes().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
            return Err("invalid SHA-256 checkpoint digest".into());
        }
        if checkpoint.hero != self.hero || checkpoint.seed != self.seed || checkpoint.run_id != self.run_id {
            return Err("checkpoint belongs to a different campaign".into());
        }
        let rebuilt = self.checkpoint_at(checkpoint.revision)?;
        if rebuilt.digest_sha256 != checkpoint.digest_sha256 {
            return Err("trusted checkpoint digest mismatch: historical Game or action log differs".into());
        }
        Ok(())
    }
}
