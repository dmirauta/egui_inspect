use std::{
    io::{self, Read},
    path::PathBuf,
    process::exit,
};

use clap::Parser;
use egui_inspect::{egui, EframeMain, EguiInspect};

/// Creates a dialogue with interactible inputs according to an input spec, whose structure
/// and default values are defined through a toml file, the applied values are printed to stdout,
/// also in toml format.
#[derive(Parser)]
struct TomlFormDialogueArgs {
    #[arg(short, long)]
    prompt: Option<String>,
    #[arg(short, long)]
    inputs: Option<PathBuf>,
    #[arg(short, long)]
    stdin: bool,
}

impl Default for TomlFormDialogue {
    fn default() -> Self {
        let TomlFormDialogueArgs {
            prompt,
            inputs,
            stdin,
        } = TomlFormDialogueArgs::parse();
        let mut buff = vec![];
        let s = if stdin {
            _ = io::stdin().read_to_end(&mut buff);
            String::from_utf8(buff).unwrap()
        } else {
            std::fs::read_to_string(
                inputs.expect("Must specify a fields file or input through stdin (see --help)."),
            )
            .unwrap()
        };
        let inputs: toml::Value = toml::from_str(s.as_str()).unwrap();
        Self { inputs, prompt }
    }
}

#[derive(EframeMain)]
struct TomlFormDialogue {
    prompt: Option<String>,
    inputs: toml::Value,
}

impl EguiInspect for TomlFormDialogue {
    fn inspect_mut(&mut self, _label: &str, ui: &mut egui::Ui) {
        if let Some(message) = self.prompt.as_ref() {
            ui.label(message);
        }
        self.inputs.inspect_mut("Options", ui);
        if ui.button("Apply").clicked() {
            print!("{}", toml::to_string_pretty(&self.inputs).unwrap());
            exit(0);
        }
    }
}
