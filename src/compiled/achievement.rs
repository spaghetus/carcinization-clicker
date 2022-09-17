use include_dir::Dir;
use rhai::{Engine, AST};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CompiledAchievement {
	pub spec: AchievementSpec,
	pub script: Option<AST>,
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
			.get_file(dir.path().join("achievement.rhai"))
			.and_then(|file| file.contents_utf8())
			.map(|script| Engine::new().compile(script).expect("Failed to parse rhai"));

		CompiledAchievement { spec, script }
	}
}
