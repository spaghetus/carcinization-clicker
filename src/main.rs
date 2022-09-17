use carcinization_clicker::{compiled::game::CompiledGame, GameState};
use chrono::{Duration, Utc};
use eframe::{egui, App};
use include_dir::{include_dir, Dir};
use num::{rational::Ratio, BigInt, BigRational, FromPrimitive};

const GAME_DEF: Dir = include_dir!("game_def");

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
}

impl App for GuiState {
	fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
		let prev_state = self.state.clone();
		self.state.tick(&self.compiled, Utc::now());
		let delta_t = self.state.last_tick - prev_state.last_tick;
		let delta_c = self.state.carcinized.clone() - prev_state.carcinized;
		let rate = if !delta_t.is_zero() {
			BigRational::from_integer(BigInt::from(1000)) * delta_c
				/ BigRational::from_integer(BigInt::from(delta_t.num_milliseconds()))
		} else {
			BigRational::from_i16(0).unwrap()
		};
		egui::TopBottomPanel::bottom("bottom_info").show(ctx, |ui| {
			ui.label(format!("{} carcinizations per second", rate.to_integer()));
		});
		egui::SidePanel::right("shop").show(ctx, |ui| {
			ui.heading("Shop");
			ui.separator();
			egui::CollapsingHeader::new("Buildings").show(ui, |ui| {
				let mut buildings: Vec<(_, _)> =
					self.compiled.buildings.clone().into_iter().collect();
				buildings.sort_by(|(_, a), (_, b)| a.spec.base_cost.cmp(&b.spec.base_cost));
				for (id, building) in buildings {
					if self.state.buildings.get(&id).unwrap_or(&0) == &0
						&& Ratio::from_u128(building.spec.base_cost).unwrap()
							> self.state.carcinized
					{
						continue;
					}
					let cost = (0..*self.state.buildings.get(&id).unwrap_or(&0))
						.fold(building.spec.base_cost, |acc, _| acc * 115 / 100);
					let cost = Ratio::from_u128(cost).unwrap();
					ui.heading(format!(
						"{} - {} - ${}",
						self.state.buildings.get(&id).unwrap_or(&0),
						building.spec.name,
						cost
					));
					ui.label(building.spec.flavor);
					if cost <= self.state.carcinized {
						if ui.button("Buy").clicked() {
							self.state.carcinized -= cost;
							*self.state.buildings.entry(id).or_insert(0) += 1;
						}
					} else if rate.clone() > Ratio::from_u8(0).unwrap() {
						ui.label(format!(
							"{}s to afford",
							((cost - self.state.carcinized.clone()) / rate.clone()).to_integer()
						));
					} else {
						ui.label("Can't afford");
					}
					ui.separator();
				}
			})
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading(format!(
				"{} programs carcinized",
				self.state.carcinized.to_integer()
			));
			if ui.button("Carcinize!!!").clicked() {
				self.state.click(&self.compiled);
			}
		});
		ctx.request_repaint_after(std::time::Duration::from_secs(1))
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
