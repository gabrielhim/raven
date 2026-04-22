use anstyle::{AnsiColor, Color, Style};
use clap::{ArgAction, Parser, Subcommand, ValueEnum, builder::Styles};

use raven::query_variant;

fn define_styles() -> Styles {
    Styles::styled()
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))),
        )
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Vcf,
}

#[derive(Subcommand)]
enum Subcmd {
    #[command(about = "Annotate a single variant.", arg_required_else_help = true)]
    Query(QueryArgs),
}

#[derive(Parser)]
struct QueryArgs {
    /// Variant to query (format must be CHROM:POS:REF:ALT)
    #[arg(short, long)]
    variant: String,

    /// Indexed VCF with known annotations. Can be specified multiple times.
    #[arg(short, long, action = ArgAction::Append)]
    dataset: Vec<String>,

    /// JSON file to write annotation to (default is stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Overwrites an existing file with the provided output file name
    #[arg(short = 'w', long)]
    overwrite: bool,
}

#[derive(Parser)]
#[command(
    version,
    about = "Variant handling and annotation tool.",
    styles = define_styles(),
    arg_required_else_help = true
)]
struct Args {
    #[clap(subcommand)]
    subcmd: Subcmd,
}

fn main() {
    let args = Args::parse();

    match args.subcmd {
        Subcmd::Query(args) => {
            query_variant(&args.variant, args.dataset, args.output, args.overwrite)
        }
    }
}
