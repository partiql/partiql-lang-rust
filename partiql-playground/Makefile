build:
	wasm-pack --version && \
		wasm-pack build --target web --out-dir pkg-web/ && \
		wasm-pack build --target nodejs --out-dir pkg-node/ && \
		npm i

container-build:
	docker build . -t partiql-team/partiql-playground

container-run:
	docker run -d -p 8000:8000 partiql-team/partiql-playground