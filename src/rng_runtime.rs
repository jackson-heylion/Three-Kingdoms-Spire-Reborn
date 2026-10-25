//! Opt-in, non-persistent RNG call-site tracing.
//!
//! The normal game path keeps rng_trace=None. Offline action replay enables
//! tracing to explain every draw, including upper<=1 calls that consume no RNG.
//! Keeping this in its own module reduces churn in the large rules engine.
use crate::{Card, Game, rng_events::{RngCall, MAX_ATTRIBUTED_DRAWS_PER_ACTION}};

impl Game {
    #[track_caller]
    pub(crate) fn random(&mut self, upper: usize) -> usize {
        let before = self.rng_state;
        let value = if upper <= 1 {
            0
        } else {
            let mut x = self.rng_state;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.rng_state = x;
            (x.wrapping_mul(0x2545F4914F6CDD1D) % upper as u64) as usize
        };
        if let Some(calls) = self.rng_trace.as_mut() {
            // Preserve the diagnostic failure signal without unbounded memory.
            // The normal gameplay RNG is never capped or rejected.
            if calls.len() <= MAX_ATTRIBUTED_DRAWS_PER_ACTION as usize {
                let site = std::panic::Location::caller();
                calls.push(RngCall {
                    call_site: format!("{}:{}", site.file(), site.line()),
                    upper,
                    value,
                    state_before: before,
                    state_after: self.rng_state,
                });
            }
        }
        value
    }

    pub(crate) fn shuffle(&mut self, cards: &mut [Card]) {
        for i in (1..cards.len()).rev() {
            let j = self.random(i + 1);
            cards.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Hero;

    #[test]
    fn zero_bound_calls_are_visible_without_consuming_state_or_touching_save() {
        let mut game = Game::new(Hero::GuanYu, 42);
        let original = game.to_json().unwrap();
        let state = game.rng_state;
        game.rng_trace = Some(Vec::new());
        assert_eq!(game.random(0), 0);
        assert_eq!(game.random(1), 0);
        assert_eq!(game.rng_state, state);
        let sampled = game.random(7);
        assert!(sampled < 7);
        let calls = game.rng_trace.take().unwrap();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls.iter().map(|call| call.upper).collect::<Vec<_>>(), vec![0, 1, 7]);
        assert!(calls[..2].iter().all(|call| call.state_before == state
            && call.state_after == state && call.value == 0));
        assert!(calls.iter().all(|call| call.call_site.contains(".rs:")));
        assert_ne!(calls[2].state_after, state);
        assert_eq!(game.to_json().unwrap(), original, "diagnostics never persist");
    }
}
