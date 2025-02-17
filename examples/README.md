# Examples

See run commands below.

## Autoprogress

A somewhat (syntactically) minimal example, with tasks running in background threads and reporting progress to the gui.

```
cargo r -r --bin autoprogress
```

## (Derive annotations) Showcase

A more exhaustive example of the available derive options.

```
cargo r -r --bin showcase
```

## Toml Form Dialogue

CLI initialisation through clap.

```
cargo r -r --bin toml_form_dialogue -- -p "Dial in the following options:" -i example/toml_form_dialogue/fields.toml
```

or

```
cargo b -r --bin toml_form_dialogue
echo "foo = 1.0\nbar = \"baz\"" | $CARGO_TARGET_DIR/release/toml_form_dialogue -p "Dial in the following options:" --stdin
```

## Log test

Quick setup of simultaneous logging to gui, terminal, and file (using fern).

```
cargo r -r --bin log_test
```

## Frag viewport

A minimal fragment shader viewport example (for native or WASM).

For running on WASM one merely needs to add the `assets` folder and `index.html` file from https://github.com/emilk/eframe_template and simply run `trunk serve` (for testing locally).
All the other boilerplate has been added to the EframeMain derive and the logging module.

## Others

- [table_ocr](https://github.com/dmirauta/table_ocr)

- [live_plotting](https://github.com/dmirauta/live_plotting) - WASM demo

- [egui-opencl-fractals](https://github.com/dmirauta/egui-opencl-fractals) - GPU compute

- [pacmap](https://github.com/dmirauta/pacmap) - dependency viewer (graph visualisation) for the pacman package manager
