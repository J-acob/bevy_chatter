wasm_bind:
	wasm-bindgen --no-typescript --out-dir wasm --target web target/wasm32-unknown-unknown/release/bevy_template_wsl2.wasm

monkey_patch:
	sed -i.bak \
		-e 's/getObject(arg0).fetch(/window.bevyLoadingBarFetch(/' \
		-e 's/input = fetch(/input = window.bevyLoadingBarFetch(/' \
		wasm/bevy_template_wsl2.js

distrib_web: wasm_bind monkey_patch