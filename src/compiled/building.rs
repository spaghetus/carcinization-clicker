use std::sync::Arc;

use dyon::Module;
use include_dir::Dir;
// This is used but clippy doesn't see it i guess
#[allow(unused_imports)]
use num::{BigRational, FromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CompiledBuilding {
	pub spec: BuildingSpec,
	pub script: Option<Module>,
}

impl CompiledBuilding {
	pub fn compile(dir: &Dir) -> Self {
		let spec = dir
			.get_file(dir.path().join("building.json5"))
			.unwrap_or_else(|| panic!("missing building.json5 for {:?}", dir))
			.contents_utf8()
			.unwrap_or_else(|| panic!("non-utf8 in json5 for {:?}", dir));
		let spec = json5::from_str(spec).expect("bad building.json5");
		let script = dir
			.get_file(dir.path().join("building.lua"))
			.and_then(|file| file.contents_utf8())
			.map(|script| {
				let mut module = Module::new();
				dyon::load_str("building.dyon", Arc::new(script.to_string()), &mut module)
					.expect("Failed to load script");
				module
			});

		CompiledBuilding { spec, script }
	}
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BuildingSpec {
	pub name: String,
	pub flavor: String,
	pub base_cps: BigRational,
	pub base_cost: BigRational,
}

#[test]
fn test_ser_buildingspec() {
	let spec = BuildingSpec {
		name: "A building spec".to_string(),
		flavor: "A building spec".to_string(),
		base_cps: BigRational::from_float(39.5).unwrap(),
		base_cost: BigRational::from_u8(10).unwrap(),
	};
	let text = json5::to_string(&spec).unwrap();
	println!("{}", text);
}
