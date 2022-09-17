fn check(game_state) {
	return game_state.buildings.get("programmer").unwrap_or(0) >= 2
}