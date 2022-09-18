use std::sync::Arc;

use dyon::Module;
use include_dir::Dir;
use num::BigRational;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CompiledUpgrade {
	pub spec: UpgradeSpec,
	pub script: Option<Module>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UpgradeSpec {
	pub name: String,
	pub flavor: String,
	pub depends: Vec<String>,
	pub effects: Vec<UpgradeEffect>,
	pub cost: BigRational,
}

impl CompiledUpgrade {
	pub fn compile(dir: &Dir) -> Self {
		let spec = dir
			.get_file(dir.path().join("upgrade.json5"))
			.unwrap_or_else(|| panic!("missing upgrade.json5 for {:?}", dir))
			.contents_utf8()
			.unwrap_or_else(|| panic!("non-utf8 in json5 for {:?}", dir));
		let spec = json5::from_str(spec).expect("bad upgrade.json5");
		let script = dir
			.get_file(dir.path().join("upgrade.lua"))
			.and_then(|file| file.contents_utf8())
			.map(|script| {
				let mut module = Module::new();
				dyon::load_str("building.dyon", Arc::new(script.to_string()), &mut module)
					.expect("Failed to load script");
				module
			});

		CompiledUpgrade { spec, script }
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UpgradeEffect {
	pub affects: String,
	pub cps_fac: Option<BigRational>,
	pub cps_add: Option<BigRational>,
}
