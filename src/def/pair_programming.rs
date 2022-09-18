use std::sync::Arc;

use super::GameEffect;

pub fn gen_compiled_effect() -> GameEffect {
	GameEffect {
		name: "Pair Programming".to_string(),
		flavor: "Everything is better with a friend".to_string(),
		depends: vec![],
		kind: super::EffectKind::Achievement {
			check: Arc::new(|game| game.buildings.get("programmer").unwrap_or(&0) >= &2),
		},
	}
}
