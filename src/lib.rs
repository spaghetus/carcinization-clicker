use std::{
	any::Any,
	collections::{HashMap, HashSet},
	sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use compiled::game::CompiledGame;
use dyon::{
	dyon_macro_items, dyon_obj, embed::PushVariable, Call, Module, Runtime, RustObject, Variable,
};
use num::{rational::Ratio, BigInt, BigRational, FromPrimitive};
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
			let time_delta: BigRational =
				Ratio::from_i64(time_delta).unwrap() / Ratio::from_i64(1000).unwrap();

			let new_carcinized = base_cps * time_delta.clone() * Ratio::from_usize(*count).unwrap();
			self.carcinized += new_carcinized;
		}
		// Check achievements
		for (id, achievement) in game.achievements.iter() {
			if achievement
				.spec
				.depends
				.iter()
				.all(|v| self.achievements.contains(v))
			{}
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
	pub fn mk_engine() -> Arc<Runtime> {
		let mut ng = Runtime::new();
		Arc::new(ng)
	}

	pub fn get_building(&self, k: String) -> usize {
		*self.buildings.get(&k).unwrap_or(&0)
	}
	pub fn set_building(&mut self, k: String, v: usize) {
		self.buildings.insert(k, v);
	}

	pub fn get_upgrade(&self, k: String) -> bool {
		self.upgrades.contains(&k)
	}
	pub fn set_upgrade(&mut self, k: String, v: bool) {
		if v {
			self.upgrades.insert(k);
		} else {
			self.upgrades.remove(&k);
		}
	}

	pub fn get_achievement(&self, k: String) -> bool {
		self.achievements.contains(&k)
	}
	pub fn set_achievement(&mut self, k: String, v: bool) {
		if v {
			self.achievements.insert(k);
		} else {
			self.achievements.remove(&k);
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

pub mod compiled;

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
