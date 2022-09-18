use std::sync::Arc;

use num::{BigInt, BigRational, FromPrimitive};

use super::GameEffect;

pub fn gen_compiled_effect() -> GameEffect {
	GameEffect {
		name: "Programmer".to_string(),
		flavor: "A fellow programmer to join your cause.".to_string(),
		depends: vec![],
		kind: super::EffectKind::Building {
			base_cost: BigRational::from_u8(10).unwrap(),
			cost_fac: BigRational::from_f64(1.15).unwrap(),
			effect: Arc::new(|game, delta_t| {
				game.carcinized += BigRational::from_f64(0.1).unwrap()
					* BigRational::new_raw(
						BigInt::from_i64(delta_t.num_milliseconds()).unwrap(),
						BigInt::from_i64(1000).unwrap(),
					)
			}),
		},
	}
}
