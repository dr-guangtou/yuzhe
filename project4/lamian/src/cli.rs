use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::inject::{CopyMode, SourceType};

#[derive(Debug, Parser)]
#[command(name = "lamian")]
#[command(about = "LaMian CLI")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    pub vault: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,

    Inject {
        file_path: PathBuf,

        #[arg(long, value_enum)]
        source_type: SourceType,

        #[arg(long)]
        source_key: String,

        #[arg(long, value_enum, default_value = "copy")]
        copy_mode: CopyMode,
    },

    Update {
        figure_id: String,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        caption: Option<String>,

        #[arg(long, value_name = "PATH")]
        note_file: Option<PathBuf>,
    },

    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    Link {
        #[command(subcommand)]
        action: LinkAction,
    },

    Search {
        #[arg(long)]
        tag: Option<String>,

        #[arg(long)]
        source_key: Option<String>,

        #[arg(long)]
        text: Option<String>,
    },

    Export {
        #[arg(long, value_enum, default_value = "yaml")]
        format: ExportFormat,

        #[arg(long, value_name = "PATH")]
        target: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TagAction {
    Add { figure_id: String, tag: String },
    Remove { figure_id: String, tag: String },
    Rename { old_tag: String, new_tag: String },
}

#[derive(Debug, Subcommand)]
pub enum LinkAction {
    Add {
        from_figure_id: String,
        to_figure_id: String,
        #[arg(long, default_value = "related")]
        relation: String,
    },
    Remove {
        from_figure_id: String,
        to_figure_id: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Yaml,
    Json,
}
