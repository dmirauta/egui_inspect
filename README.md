# egui_inspect

This is a fork of [egui_inspect](https://github.com/Meisterlama/egui_inspect).

Seeks to derive a graphical interface mostly from some light annotations on the structs we wish to expose.

[Source code](egui_example/src/autoprogress.rs)  |  Result
:-----------------------------------------------:|:-------------------------:
![](resources/auto_progress_source.png)          |  ![](resources/progress_bars.gif)

See [examples](./egui_example/) for more.

The `egui_inspect_wrap` subcrate allows for generation of nearly identical but crate defined structs, shadowing the original (with `from` and `into` methods), so that we can effectively derive `EguiInspect` for externally defined structs.

To use this fork, add the following to your `Cargo.toml`.

```toml
[dependencies]
egui_inspect = { git = "https://github.com/dmirauta/egui_inspect" }
# if also requiring the wrapper generation macros
# egui_inspect_wrap = { git = "https://github.com/dmirauta/egui_inspect" }
```
