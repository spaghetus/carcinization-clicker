use chrono::Duration;
use std::{collections::HashMap, sync::Arc};

use num::BigRational;

use crate::GameState;

macro_rules! gen_compiled_game {
	($($id:ident)+) => {
		$(
			mod $id;
		)*

		#[must_use]
		pub fn gen_compiled_game() -> CompiledGame {
			use std::collections::HashMap;
			let mut effects = HashMap::new();
			$(
				effects.insert(stringify!($id).to_string(), $id::gen_compiled_effect());
			)*
			CompiledGame {
				effects
			}
		}
	};
}

gen_compiled_game! {
	programmer
	pair_programming
	copilot
}

pub struct CompiledGame {
	pub effects: HashMap<String, GameEffect>,
}

#[derive(Clone)]
pub struct GameEffect {
	pub name: String,
	pub flavor: String,
	pub depends: Vec<String>,
	pub kind: EffectKind,
}

#[derive(Clone)]
pub enum EffectKind {
	Building {
		base_cost: BigRational,
		cost_fac: BigRational,
		// smh the type isn't *that* complex
		#[allow(clippy::type_complexity)]
		effect: Arc<dyn Fn(&mut GameState, Duration)>,
	},
	Upgrade {
		affects: Vec<String>,
		cost: BigRational,
		effect: Arc<dyn Fn(&mut GameEffect)>,
	},
	Achievement {
		check: Arc<dyn Fn(&GameState) -> bool>,
	},
}
