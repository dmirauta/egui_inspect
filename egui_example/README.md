# Examples

See run commands below.

## Autoprogress

A somewhat (syntactically) minimal example.

```
cargo r -r --bin autoprogress
```

## Showcase

A more exhaustive example of the available options.

```
cargo r -r --bin showcase
```

## Toml Form Dialogue

```
cargo r -r --bin toml_form_dialogue -- -p "Dial in the following options:" -i fields.toml
```

or

```
cargo b -r --bin toml_form_dialogue
echo "foo = 1.0\nbar = \"baz\"" | $CARGO_TARGET_DIR/release/toml_form_dialogue -p "Dial in the following options:" --stdin
```
