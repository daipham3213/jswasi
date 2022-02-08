alias be := build-embedded
alias b := build
alias s := start
alias r := record

shell:
	cargo build --manifest-path=./shell/Cargo.toml --target wasm32-wasi --release
	cp ./shell/target/wasm32-wasi/release/*.wasm ./dist/resources

build-embedded:
	if [ -d dist ]; then touch dist/dummy.txt; rm dist/*.*; fi
	if [ -d dist/resources ]; then rm -rf dist/resources; fi
	mkdir -p dist/resources
	cp -r vendor/ dist/
	cargo build --manifest-path=shell/Cargo.toml --target wasm32-wasi --release
	cp shell/target/wasm32-wasi/release/*.wasm dist/resources
	cp wasm_binaries/*.wasm dist/resources
	cp src/motd.txt dist/resources
	npx tsc

build: build-embedded
	cp src/favicon.ico dist
	cp src/index.html dist/

default_port := "8000"

start PORT=default_port:
	cd dist && python3 ../custom_server.py {{PORT}}

dev PORT=default_port:
	cd dist && python3 ../custom_server.py {{PORT}} &
	npx tsc --watch

record:
    cd ci && node grab-screencast.js "$@" && rm -r images && rm images.txt
