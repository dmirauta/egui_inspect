# egui_inspect

This is a fork of [egui_inspect](https://github.com/Meisterlama/egui_inspect).

Seeks to derive a graphical interface mostly from some light annotations on the structs we wish to expose.

Source code                              |  Result
:---------------------------------------:|:-------------------------:
![](resources/auto_progress_source.png)  |  ![](resources/progress_bars.gif)

For further options see `egui_example/src/showcase.rs`.

![screenshot](resources/screenshot.png)

To use this fork, add the following to your `Cargo.toml`.

```toml
[dependencies]
egui_inspect = { git = "https://github.com/dmirauta/egui_inspect" }
egui_inspect_wrap = { git = "https://github.com/dmirauta/egui_inspect" }
```
