# PartiQL Playground (Proof of Concept)

PartiQL Playground is intended to provide the required code for enabling execution of PartiQL queries on web.

_Please note, at this stage the code within this package is considered experimental and should not be used for production._

## Local Usage
For local usage follow the below steps.

1. Ensure `wasm-pack` is installed on your machine by running the following command; if not, install it from [here](https://rustwasm.github.io/wasm-pack/installer/):
```bash
wasm-pack --version
# Sample output
wasm-pack 0.10.2
```
2. Ensure `npm` is intalled on your machine; see [here](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm) for more details:
```bash
npm --version
# Sample output
8.13.2
```
3. Pull down the `partiql-lang-rust` repository from GitHub:
```bash
git clone --recursive https://github.com/partiql/partiql-lang-rust.git
```
4. Enter the `partiql-playground` root directory:
```bash
cd partiql-lang-rust/partiql-playground
```
5. Run `make build`:
```
make
```
6. Start the node server from `partiql-playground` package's root directory:
```bash
node src/server.ts
```
7. your browser go to `http://localhost:8000/`

## Run via docker container

1. Ensure `docker` is installed on your machine, by running the following:
```bash
docker --version
# Example output
Docker version 20.10.17, build 100c701
```
2. Build the package:
```bash
make build
```
3. Build the container:
```bash
make container-build
```
4. Run the container:
```bash
make container-run
```
5. Confirm it's running:
```bash
docker ps
# Example output
CONTAINER ID   IMAGE                         COMMAND                  CREATED          STATUS          PORTS                              NAMES
1d666bba30c2   partiql-team/partiql-playground   "docker-entrypoint.s…"   4 minutes ago    Up 4 minutes    0.0.0.0:8000->8000/tcp, 8080/tcp   infallible_goldberg

# Ensure `connected` is in the curl output 
curl -v  http://localhost:8000 2>&1 |grep -i connected
# Example output
0     0    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0* Connected to localhost (127.0.0.1) port 8000 (#0)
```

## Development
`PartiQL Playground` uses [WebAssembly (Wasm)](https://webassembly.org/) for integrating the front-end with PartiQL Rust back-end.
Considering this, please install the `wasm-pack` by following the instructions [here](https://github.com/rustwasm/wasm-pack#-prerequisities).

Upon any changes to the package's Rust dependencies (E.g. `partiql-parser`) or the wasm code under `./src/lib` of this package, you need to rebuild the Wasm package using the following command from the root of this package:
```bash
wasm-pack build --target web
```

_Please note, as the package is experimental at this stage, all HTML code and assets reside in this package, but this doesn't necessarily mean that it'll be the case in the future._

## Dependencies
| Package                                                                | License                                                                                         |
|------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------|
| [ace Editor](https://ace.c9.io/)                                       | [BSD License](https://github.com/ajaxorg/ace/blob/master/LICENSE)                               |
| [body-parser](https://github.com/expressjs/body-parser)                | [MIT License](https://github.com/expressjs/body-parser/blob/master/LICENSE)                     |
| [bootstrap](https://getbootstrap.com/)                                 | [MIT License](https://github.com/twbs/bootstrap/blob/main/LICENSE)                              |
| [D3.js](https://d3js.org/)                                             | [ISC License](https://github.com/d3/d3/blob/main/LICENSE)                                       |
| [jquery](https://jquery.com)                                           | [MIT License](https://github.com/jquery/jquery/blob/main/LICENSE.txt)                           |
| [jquery.json-viewer](https://www.npmjs.com/package/jquery.json-viewer) | [MIT License](https://github.com/abodelot/jquery.json-viewer/blob/master/LICENSE)               |
| [node](https://nodejs.org/en/)                                         | [MIT License](https://github.com/nodejs/node/blob/main/LICENSE)                                 |
| [popper.js](https://github.com/floating-ui/floating-ui)                | [MIT License](https://github.com/floating-ui/floating-ui/blob/master/LICENSE)                   | 
| [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)               | [Apache License Version 2.0](https://github.com/rustwasm/wasm-bindgen/blob/main/LICENSE-APACHE) | 
| [wasm-pack](https://github.com/rustwasm/wasm-pack)                     | [Apache License Version 2.0](https://github.com/rustwasm/wasm-pack/blob/master/LICENSE-APACHE)  |