# Brainfuck interpreter GUI (WebAssembly)

## Getting Started

This is a brainfuck interpreter built using [Rust](https://www.rust-lang.org) language with the [egui](https://www.egui.rs) immediate mode GUI library. It's simple enough to demonstrate the usage of egui with WebAssembly (aka Wasm).

To run it locally in your Web browser, you can use [trunk](https://trunkrs.dev), and go to [http://localhost:8080](http://localhost:8080):

```bash
trunk serve
```

Wasm is not the only target of this application, the GUI can also be run as a desktop application with:

```bash
cargo run
```

### Alternative

The Wasm file is served through Cloudflare Pages. To see it in action, open your Web browser and navigate to [https://brainfuck-gui-rs.jaudiger.dev/](https://brainfuck-gui-rs.jaudiger.dev/).
