use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::inject::{CopyMode, SourceType};
use crate::query::{QueryRunDetail, QuerySortField, QuerySortOrder};

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

        #[arg(long, default_value_t = false)]
        clear_caption: bool,

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

    Query {
        #[command(subcommand)]
        action: QueryAction,
    },

    Collection {
        #[command(subcommand)]
        action: CollectionAction,
    },

    Bundle {
        #[command(subcommand)]
        action: BundleAction,
    },

    Doctor {
        #[arg(long, default_value_t = false)]
        fix: bool,
    },

    Import {
        input_path: PathBuf,

        #[arg(long, value_enum)]
        source_type: SourceType,

        #[arg(long)]
        source_key_template: String,

        #[arg(long, value_enum, default_value = "copy")]
        copy_mode: CopyMode,

        #[arg(long, default_value_t = false)]
        recursive: bool,

        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    Export {
        #[arg(long, value_enum, default_value = "yaml")]
        format: ExportFormat,

        #[arg(long, value_name = "PATH")]
        target: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueryAction {
    Save {
        name: String,

        #[arg(long)]
        tag: Option<String>,

        #[arg(long)]
        source_key: Option<String>,

        #[arg(long)]
        text: Option<String>,

        #[arg(long, value_enum, default_value = "figure-id")]
        sort: QuerySortField,

        #[arg(long, value_enum, default_value = "asc")]
        order: QuerySortOrder,

        #[arg(long)]
        limit: Option<u32>,
    },
    Run {
        name_or_id: String,

        #[arg(long, value_enum, default_value = "ids")]
        detail: QueryRunDetail,
    },
    List,
    Delete {
        name_or_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum CollectionAction {
    Create {
        name: String,

        #[arg(long)]
        query_id: Option<i64>,
    },
    Add {
        collection: String,
        figure_id: String,
    },
    Remove {
        collection: String,
        figure_id: String,
    },
    List {
        #[arg(long)]
        collection: Option<String>,
    },
    Delete {
        collection: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BundleAction {
    Export {
        #[arg(long, value_name = "PATH")]
        target: PathBuf,
    },
    Import {
        bundle_path: PathBuf,
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
