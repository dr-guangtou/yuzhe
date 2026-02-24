use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

use crate::collection::CollectionReferenceMode;
use crate::inject::{CopyMode, SourceType};
use crate::query::{QueryReferenceMode, QueryRunDetail, QuerySortField, QuerySortOrder};

#[derive(Debug, Parser)]
#[command(name = "lamian")]
#[command(about = "LaMian CLI")]
#[command(version)]
pub struct Cli {
    #[arg(long, global = true, value_name = "PATH")]
    pub vault: Option<PathBuf>,

    #[arg(long = "json", global = true, default_value_t = false)]
    pub json_output: bool,

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

    Source {
        #[command(subcommand)]
        action: SourceAction,
    },

    Link {
        #[command(subcommand)]
        action: LinkAction,
    },

    Search {
        #[arg(long)]
        tag: Option<String>,

        #[arg(long)]
        tag_prefix: Option<String>,

        #[arg(long)]
        source_key: Option<String>,

        #[arg(long)]
        text: Option<String>,
    },

    #[command(alias = "ls")]
    List {
        #[arg(long, value_enum, default_value = "figure-id")]
        sort: ListSortField,

        #[arg(long, value_enum, default_value = "asc")]
        order: ListSortOrder,

        #[arg(long)]
        limit: Option<u32>,
    },

    #[command(alias = "info")]
    Show {
        figure_id: String,
    },

    Open {
        figure_id: String,
    },

    Delete {
        figure_id: String,
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

    Verify,

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

        #[arg(long, value_enum, default_value = "auto")]
        reference_mode: QueryReferenceMode,
    },
    List,
    Delete {
        name_or_id: String,

        #[arg(long, value_enum, default_value = "auto")]
        reference_mode: QueryReferenceMode,
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

        #[arg(long, value_enum, default_value = "auto")]
        reference_mode: CollectionReferenceMode,
    },
    Remove {
        collection: String,
        figure_id: String,

        #[arg(long, value_enum, default_value = "auto")]
        reference_mode: CollectionReferenceMode,
    },
    List {
        #[arg(long)]
        collection: Option<String>,

        #[arg(long, value_enum, default_value = "auto")]
        reference_mode: CollectionReferenceMode,
    },
    Delete {
        collection: String,

        #[arg(long, value_enum, default_value = "auto")]
        reference_mode: CollectionReferenceMode,
    },
    Update {
        collection: String,

        #[arg(long, value_enum, default_value = "auto")]
        reference_mode: CollectionReferenceMode,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        query_id: Option<i64>,

        #[arg(long, default_value_t = false)]
        clear_query_id: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum BundleAction {
    Export {
        #[arg(long, value_name = "PATH")]
        target: PathBuf,
    },
    Inspect {
        bundle_path: PathBuf,
    },
    Import {
        bundle_path: PathBuf,

        #[arg(long, default_value_t = false)]
        fail_on_link_loss: bool,

        #[arg(long, default_value_t = false)]
        dry_run: bool,

        #[arg(long, value_enum, default_value = "skip")]
        on_conflict: BundleImportConflictPolicy,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BundleImportConflictPolicy {
    Skip,
    Error,
    Replace,
}

#[derive(Debug, Subcommand)]
pub enum TagAction {
    Add { figure_id: String, tag: String },
    Remove { figure_id: String, tag: String },
    Rename { old_tag: String, new_tag: String },
    List,
}

#[derive(Debug, Subcommand)]
pub enum SourceAction {
    Update {
        figure_id: String,

        #[arg(long)]
        title: Option<String>,

        #[arg(long)]
        authors: Option<String>,

        #[arg(long)]
        published_at: Option<String>,

        #[arg(long, default_value_t = false)]
        clear_title: bool,

        #[arg(long, default_value_t = false)]
        clear_authors: bool,

        #[arg(long, default_value_t = false)]
        clear_published_at: bool,
    },
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ListSortField {
    FigureId,
    DisplayName,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ListSortOrder {
    Asc,
    Desc,
}
