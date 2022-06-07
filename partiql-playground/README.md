# PartiQL Playground (Proof of Concept)

PartiQL Playground is intended to provide the required code for enabling execution of PartiQL queries on web.

_Please note, at this stage the code within this package is considered experimental and should be used for production._

## Local Usage
For local usage follow the below steps.

1. Pull down the `partiql-rust` package from GitHub:
```bash
git clone --recursive https://github.com/partiql/partiql-lang-rust.git
```
2. Enter the `partiql-playground` root directory:
```bash
cd partiql-lang-rust/partiql-playground
```
3. Start a webserver from the root directory, as an example, you can use [Python's SimpleHTTPServer](https://docs.python.org/3.8/library/http.server.html#http.server.SimpleHTTPRequestHandler):
```bash
python3 -m http.server
```
4. On your browser go to `http://http://localhost:8000/`

## Development
`PartiQL Playground` uses [WebAssembly (Wasm)](https://webassembly.org/) for integrating the front-end with PartiQL Rust back-end.
Considering this, upon any changes to the package's Rust dependencies (E.g. `partiql-parser`) or the wasm code under `./src/lib` of this package, you need to rebuild the Wasm package using the following command from the root of this package:
```bash
wasm-pack build --target web
```

_Please note, as the package is experimental at this stage, all HTML code and assets reside in this package, but this doesn't necessarily mean that it'll be the case in the future._

## Dependencies
| Package                                                                | License                                                                                         |
|------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------|
| [ace Editor](https://ace.c9.io/)                                       | [BSD License](https://github.com/ajaxorg/ace/blob/master/LICENSE)                               |
| [bootstrap](https://getbootstrap.com/)                                 | [MIT License](https://github.com/twbs/bootstrap/blob/main/LICENSE)                              |
| [D3.js](https://d3js.org/)                                             | [ISC License](https://github.com/d3/d3/blob/main/LICENSE)                                       |
| [jquery](https://jquery.com)                                           | [MIT License](https://github.com/jquery/jquery/blob/main/LICENSE.txt)                           |
| [jquery.json-viewer](https://www.npmjs.com/package/jquery.json-viewer) | [MIT License](https://github.com/abodelot/jquery.json-viewer/blob/master/LICENSE)               |  
| [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)               | [Apache License Version 2.0](https://github.com/rustwasm/wasm-bindgen/blob/main/LICENSE-APACHE) | 
| [wasm-pack](https://github.com/rustwasm/wasm-pack)                     | [Apache License Version 2.0](https://github.com/rustwasm/wasm-pack/blob/master/LICENSE-APACHE)  |