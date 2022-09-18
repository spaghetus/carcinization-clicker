use carcinization_clicker::{compiled::game::CompiledGame, GameState};
use chrono::Utc;
use eframe::{egui, App};
use egui_notify::Toasts;
use include_dir::{include_dir, Dir};
use num::{rational::Ratio, BigInt, BigRational, FromPrimitive, ToPrimitive};

const GAME_DEF: Dir = include_dir!("game_def");

#[macro_use]
extern crate dyon;

fn main() {
	let game_def = CompiledGame::compile(&GAME_DEF);
	let game_state: GameState = std::fs::read_to_string(
		dirs::data_dir()
			.expect("Couldn't get data dir")
			.join("carcinize.json5"),
	)
	.map(|stri| json5::from_str(&stri).expect("Bad save file"))
	.unwrap_or_default();
	let gui = GuiState {
		compiled: game_def,
		state: game_state,
		toasts: Default::default(),
	};
	eframe::run_native(
		"Carcinization Clicker",
		Default::default(),
		Box::new(|_| Box::new(gui)),
	);
}

struct GuiState {
	pub compiled: CompiledGame,
	pub state: GameState,
	pub toasts: Toasts,
}

impl App for GuiState {
	fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
		let prev_state = self.state.clone();
		self.state.tick(&self.compiled, Utc::now());
		// Check if there are new achievements
		self.state
			.achievements
			.iter()
			.filter(|v| !prev_state.achievements.contains(&v.to_string()))
			.map(|v| {
				self.compiled
					.achievements
					.get(v)
					.expect("got missing achievement")
			})
			.for_each(|v| {
				self.toasts
					.info(format!("{} - {}", v.spec.name, v.spec.flavor));
			});
		let delta_t = self.state.last_tick - prev_state.last_tick;
		let delta_c = self.state.carcinized.clone() - prev_state.carcinized;
		let rate = if !delta_t.is_zero() {
			BigRational::from_integer(BigInt::from(1000)) * delta_c
				/ BigRational::from_integer(BigInt::from(delta_t.num_milliseconds()))
		} else {
			BigRational::from_i16(0).unwrap()
		};
		egui::TopBottomPanel::bottom("bottom_info").show(ctx, |ui| {
			Self::draw_cps(ui, &rate);
		});
		egui::SidePanel::right("shop").show(ctx, |ui| self.draw_shop(ui, rate));
		egui::CentralPanel::default().show(ctx, |ui| {
			self.draw_main(ui);
		});
		ctx.request_repaint_after(std::time::Duration::from_secs(1));
		self.toasts.show(ctx);
	}

	fn on_close_event(&mut self) -> bool {
		let dir = dirs::data_dir()
			.expect("Couldn't get data dir")
			.join("carcinize.json5");
		std::fs::write(
			dir,
			json5::to_string(&self.state).expect("serialize failed"),
		)
		.expect("write failed");
		true
	}
}

impl GuiState {
	fn draw_cps(ui: &mut egui::Ui, rate: &Ratio<BigInt>) {
		ui.label(format!(
			"{} carcinizations per second",
			carcinization_clicker::human_number(rate, 2)
		));
	}

	fn draw_main(&mut self, ui: &mut egui::Ui) {
		ui.heading(format!(
			"{} programs carcinized",
			carcinization_clicker::human_number(&self.state.carcinized, 2)
		));
		if ui.button("Carcinize!!!").clicked() {
			self.state.click(&self.compiled);
		}
	}

	fn draw_shop(&mut self, ui: &mut egui::Ui, rate: Ratio<BigInt>) {
		ui.heading("Shop");
		ui.separator();
		egui::CollapsingHeader::new("Buildings").show(ui, |ui| {
			self.draw_building_shop(ui, &rate);
		});
		egui::CollapsingHeader::new("Upgrades").show(ui, |ui| {
			self.draw_upgrade_shop(ui, &rate);
		});
	}

	fn draw_building_shop(&mut self, ui: &mut egui::Ui, rate: &Ratio<BigInt>) {
		let mut buildings: Vec<(_, _)> = self.compiled.buildings.clone().into_iter().collect();
		buildings.sort_by(|(_, a), (_, b)| a.spec.base_cost.cmp(&b.spec.base_cost));
		for (id, building) in buildings {
			if self.state.buildings.get(&id).unwrap_or(&0) == &0
				&& building.spec.base_cost > self.state.carcinized
			{
				continue;
			}
			let cost = (0..*self.state.buildings.get(&id).unwrap_or(&0))
				.fold(building.spec.base_cost, |acc, _| {
					acc * BigRational::from_f64(1.15).unwrap()
				});
			ui.heading(format!(
				"{} - {} - ${}",
				self.state.buildings.get(&id).unwrap_or(&0),
				building.spec.name,
				carcinization_clicker::human_number(&cost, 2)
			));
			ui.label(building.spec.flavor);
			if cost <= self.state.carcinized {
				if ui.button("Buy").clicked() {
					self.state.carcinized -= cost;
					*self.state.buildings.entry(id).or_insert(0) += 1;
				}
			} else if rate.clone() > Ratio::from_u8(0).unwrap() {
				let seconds = ((cost - self.state.carcinized.clone()) / rate.clone()).to_integer();
				match seconds.to_u64() {
					Some(n) => {
						let dur = std::time::Duration::from_secs(n);
						let dur = humantime::format_duration(dur);
						ui.label(format!("{} to afford", dur))
					}
					None => ui.label("Will never afford..."),
				};
			} else {
				ui.label("Can't afford");
			}
			ui.separator();
		}
	}

	fn draw_upgrade_shop(&mut self, ui: &mut egui::Ui, rate: &Ratio<BigInt>) {
		let mut upgrades: Vec<_> = self
			.compiled
			.upgrades
			.iter()
			.filter(|v| !self.state.upgrades.contains(v.0))
			.collect();
		upgrades.sort_by(|(_, a), (_, b)| a.spec.cost.cmp(&b.spec.cost));
		for (id, upgrade) in upgrades.iter() {
			if upgrade.spec.cost > self.state.carcinized.clone() * Ratio::from_f32(3.0).unwrap()
				|| !upgrade
					.spec
					.depends
					.iter()
					.all(|v| self.state.achievements.contains(v))
			{
				continue;
			}
			let cost = upgrade.spec.cost.clone();
			ui.heading(format!(
				"{} - ${}",
				upgrade.spec.name,
				carcinization_clicker::human_number(&cost, 2)
			));
			ui.label(upgrade.spec.flavor.clone());
			if cost <= self.state.carcinized {
				if ui.button("Buy").clicked() {
					self.state.carcinized -= cost;
					self.state.upgrades.insert(id.to_string());
				}
			} else if rate.clone() > Ratio::from_u8(0).unwrap() {
				let seconds = ((cost - self.state.carcinized.clone()) / rate.clone()).to_integer();
				match seconds.to_u64() {
					Some(n) => {
						let dur = std::time::Duration::from_secs(n);
						let dur = humantime::format_duration(dur);
						ui.label(format!("{} to afford", dur))
					}
					None => ui.label("Will never afford..."),
				};
			} else {
				ui.label("Can't afford");
			}
		}
	}
}
