use include_dir::Dir;

use super::achievement::CompiledAchievement;

use super::upgrade::CompiledUpgrade;

use super::building::CompiledBuilding;

use std::collections::HashMap;

#[derive(Clone)]
pub struct CompiledGame {
	pub buildings: HashMap<String, CompiledBuilding>,
	pub upgrades: HashMap<String, CompiledUpgrade>,
	pub achievements: HashMap<String, CompiledAchievement>,
}

impl CompiledGame {
	pub fn compile(dir: &Dir) -> Self {
		CompiledGame {
			buildings: {
				dir.get_dir("buildings")
					.expect("No buildings dir")
					.dirs()
					.map(|dir| {
						(
							dir.path()
								.file_name()
								.expect("bad building file name")
								.to_string_lossy()
								.to_string(),
							CompiledBuilding::compile(dir),
						)
					})
					.collect()
			},
			upgrades: {
				dir.get_dir("upgrades")
					.expect("No upgrades dir")
					.dirs()
					.map(|dir| {
						(
							dir.path()
								.file_name()
								.expect("bad upgrade file name")
								.to_string_lossy()
								.to_string(),
							CompiledUpgrade::compile(dir),
						)
					})
					.collect()
			},
			achievements: {
				dir.get_dir("achievements")
					.expect("No achievements dir")
					.dirs()
					.map(|dir| {
						(
							dir.path()
								.file_name()
								.expect("bad achievement file name")
								.to_string_lossy()
								.to_string(),
							CompiledAchievement::compile(dir),
						)
					})
					.collect()
			},
		}
	}
}
