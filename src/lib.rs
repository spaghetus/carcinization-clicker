use std::{
	collections::{HashMap, HashSet},
	sync::atomic::AtomicU64,
};

use chrono::{DateTime, Utc};
use compiled::{game::CompiledGame, upgrade::UpgradeEffect};
use include_dir::{include_dir, Dir};
use num::{rational::Ratio, traits::Pow, BigInt, BigRational, FromPrimitive};
use rhai::{Instant, AST};
use serde::{Deserialize, Serialize};

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
		// Apply buildings
		for (id, count) in self.buildings.iter() {
			let base_building = game.buildings.get(id).expect("nonexistent building owned");
			let mut base_cps = base_building.spec.base_cps.clone();
			let effects: Vec<_> = self
				.upgrades
				.iter()
				.flat_map(|f| game.upgrades.get(f))
				.flat_map(|f| f.spec.effects.clone())
				.filter(|v| &v.affects == id || &v.affects == "*")
				.collect();
			for effect in effects.iter() {
				base_cps += effect
					.cps_add
					.clone()
					.unwrap_or_else(|| Ratio::from_i16(0).unwrap());
			}
			for effect in effects.iter() {
				base_cps *= effect
					.cps_fac
					.clone()
					.unwrap_or_else(|| Ratio::from_i16(1).unwrap());
			}
			let time_delta: i64 = time_delta.num_milliseconds();
			let time_delta = Ratio::from_i64(time_delta).unwrap() / Ratio::from_i64(1000).unwrap();
			let new_carcinized = base_cps * time_delta * Ratio::from_usize(*count).unwrap();
			self.carcinized += new_carcinized;
		}
	}
	pub fn click(&mut self, game: &CompiledGame) {
		let mut base_click = BigRational::from_u8(1).unwrap();
		let effects: Vec<_> = self
			.upgrades
			.iter()
			.flat_map(|f| game.upgrades.get(f))
			.flat_map(|f| f.spec.effects.clone())
			.filter(|v| &v.affects == "click" || &v.affects == "*")
			.collect();
		for effect in effects.iter() {
			base_click += effect
				.cps_add
				.clone()
				.unwrap_or_else(|| Ratio::from_i16(0).unwrap());
		}
		for effect in effects.iter() {
			base_click *= effect
				.cps_fac
				.clone()
				.unwrap_or_else(|| Ratio::from_i16(1).unwrap());
		}
		self.carcinized += base_click
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

pub mod compiled;
