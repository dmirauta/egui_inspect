#!/bin/bash

for cargo_toml in $(find . -name "Cargo.toml"); do
	sed -i "s/^version = .*$/version = \"$1\"/" $cargo_toml
done
