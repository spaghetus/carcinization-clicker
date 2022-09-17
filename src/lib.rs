use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use compiled::game::CompiledGame;
use num::{rational::Ratio, BigInt, BigRational, FromPrimitive};
use rhai::{Engine, Scope};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
	pub last_tick: DateTime<Utc>,
	pub carcinized: BigRational,
	pub upgrades: HashSet<String>,
	pub buildings: HashMap<String, usize>,
	pub achievements: HashSet<String>,
	#[serde(skip)]
	#[serde(default = "GameState::mk_engine")]
	engine: rhai::Shared<rhai::Engine>,
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
		// Check achievements
		for (id, achievement) in game.achievements.iter() {
			if achievement
				.spec
				.depends
				.iter()
				.all(|v| self.achievements.contains(v))
			{
				let script_result: bool = if let Some(script) = achievement.script.clone() {
					let self_ser = json5::to_string(self).expect("failed to ser");
					let self_dynamic: rhai::Dynamic =
						json5::from_str(&self_ser).expect("failed to deser");
					let res = self.engine.call_fn::<bool>(
						&mut Scope::new(),
						&script,
						"check",
						(self_dynamic,),
					);
					match res {
						Ok(v) => v,
						Err(e) => {
							eprintln!("Script failed with {}", e);
							false
						}
					}
				} else {
					false
				};
				if script_result {
					self.achievements.insert(id.to_string());
				}
			}
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
	pub fn mk_engine() -> rhai::Shared<rhai::Engine> {
		let ng = Engine::new();
		rhai::Shared::new(ng)
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
			engine: GameState::mk_engine(),
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
