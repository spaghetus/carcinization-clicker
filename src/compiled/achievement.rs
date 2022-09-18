use std::sync::Arc;

use dyon::Module;
use include_dir::Dir;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CompiledAchievement {
	pub spec: AchievementSpec,
	pub script: Option<Module>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AchievementSpec {
	pub depends: Vec<String>,
	pub name: String,
	pub flavor: String,
}

impl CompiledAchievement {
	pub fn compile(dir: &Dir) -> Self {
		let spec = dir
			.get_file(dir.path().join("achievement.json5"))
			.unwrap_or_else(|| panic!("missing achievement.json5 for {:?}", dir))
			.contents_utf8()
			.unwrap_or_else(|| panic!("non-utf8 in json5 for {:?}", dir));
		let spec = json5::from_str(spec).expect("bad achievement.json5");
		let script = dir
			.get_file(dir.path().join("achievement.lua"))
			.and_then(|file| file.contents_utf8())
			.map(|script| {
				let mut module = Module::new();
				dyon::load_str("building.dyon", Arc::new(script.to_string()), &mut module)
					.expect("Failed to load script");
				module
			});

		CompiledAchievement { spec, script }
	}
}
