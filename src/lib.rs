use std::{
	any::Any,
	collections::{HashMap, HashSet},
	sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, Utc};
use def::{CompiledGame, EffectKind, GameEffect};
use num::{rational::Ratio, BigInt, BigRational, FromPrimitive};
use serde::{Deserialize, Serialize};

pub mod def;

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
	pub last_tick: DateTime<Utc>,
	pub carcinized: BigRational,
	pub upgrades: HashSet<String>,
	pub buildings: HashMap<String, usize>,
	pub achievements: HashSet<String>,
}

impl GameState {
	pub fn tick(&mut self, game: &CompiledGame, new_time: DateTime<Utc>) {
		let time_delta = new_time - self.last_tick;
		self.last_tick = new_time;
		let mut effects = vec![];
		// Get building effects
		for (id, count) in self.buildings.iter() {
			if !self.buildings.contains_key(id) {
				continue;
			}
			let base_building = game.effects.get(id).expect("nonexistent building owned");
			let mut building = base_building.clone();
			for (up_id, upgrade) in game.effects.iter() {
				if let EffectKind::Upgrade {
					affects,
					cost: _,
					effect,
				} = &upgrade.kind
				{
					if self.upgrades.contains(up_id) && affects.contains(id) {
						effect(&mut building)
					}
				}
			}
			for _ in 0..*self.buildings.get(id).unwrap_or(&0) {
				if let EffectKind::Building {
					base_cost: _,
					cost_fac: _,
					effect,
				} = &building.kind
				{
					effects.push(effect.clone())
				}
			}
		}
		// Apply building effects
		for effect in effects {
			effect(self, time_delta)
		}
		// Check achievements
		for (id, achievement) in game.effects.iter() {
			if self.achievements.contains(id)
				|| achievement
					.depends
					.iter()
					.any(|v| !self.achievements.contains(v))
			{
				continue;
			}
			if let EffectKind::Achievement { check } = &achievement.kind {
				if check(self) {
					self.achievements.insert(id.clone());
				}
			}
		}
	}

	pub fn click(&mut self, game: &CompiledGame) {
		let mut base_effect = GameEffect {
			name: "Click".to_string(),
			flavor: "This shouldn't appear".to_string(),
			depends: vec![],
			kind: EffectKind::Building {
				base_cost: Default::default(),
				cost_fac: Default::default(),
				effect: Arc::new(|game, _| game.carcinized += BigRational::from_u8(1).unwrap()),
			},
		};
		for (id, upgrade) in game.effects.iter() {
			if let EffectKind::Upgrade {
				affects,
				cost: _,
				effect,
			} = &upgrade.kind
			{
				if self.achievements.contains(id) && affects.contains(id) {
					effect(&mut base_effect)
				}
			}
		}
		if let EffectKind::Building {
			base_cost: _,
			cost_fac: _,
			effect,
		} = base_effect.kind
		{
			effect(self, Duration::seconds(0))
		}
	}
}

impl Default for GameState {
	fn default() -> Self {
		GameState {
			last_tick: Utc::now(),
			carcinized: Ratio::from_integer(BigInt::from(0)),
			upgrades: Default::default(),
			buildings: Default::default(),
			achievements: Default::default(),
		}
	}
}

pub fn human_number(n: &BigRational, decimals: u32) -> String {
	let mut n = n.clone();
	let cmp = Ratio::from_u16(1000).unwrap();
	let mut thousands: usize = 0;
	let suffixes = ["", "k", "M", "B", "G", "T", "P", "E", "Z", "Y"];
	while n > cmp && thousands < (suffixes.len() - 1) {
		thousands += 1;
		n /= cmp.clone();
	}
	let whole = n.to_integer();
	let decimal = ((n % Ratio::from_u8(1).unwrap())
		* Ratio::from_usize(10usize.pow(decimals)).unwrap())
	.to_integer();
	format!("{}.{:02}{}", whole, decimal, suffixes[thousands])
}
