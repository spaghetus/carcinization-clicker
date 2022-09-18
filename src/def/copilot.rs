use super::{EffectKind, GameEffect};
use chrono::Duration;
use num::{rational::Ratio, BigInt, BigRational, FromPrimitive};
use std::sync::Arc;

pub fn gen_compiled_effect() -> GameEffect {
	GameEffect {
		name: "Copilot".to_string(),
		flavor: "A questionably ethical automated buddy for your programmers".to_string(),
		depends: vec![],
		kind: super::EffectKind::Upgrade {
			affects: vec!["programmer".to_string()],
			cost: Ratio::from_u8(100).unwrap(),
			effect: Arc::new(|game| {
				if let EffectKind::Building {
					base_cost: _,
					cost_fac: _,
					effect,
				} = &mut game.kind
				{
					let orig_effect = effect.clone();
					*effect = Arc::new(move |game, delta_t| {
						let mut test_game = game.clone();
						orig_effect(&mut test_game, Duration::seconds(1));
						let delta = test_game.carcinized - game.carcinized.clone();
						game.carcinized += delta
							* Ratio::from_f64(1.1).unwrap()
							* Ratio::new_raw(
								BigInt::from_i64(delta_t.num_milliseconds()).unwrap(),
								BigInt::from_i64(1000).unwrap(),
							)
					})
				}
			}),
		},
	}
}
