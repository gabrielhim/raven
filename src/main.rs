use anstyle::{AnsiColor, Color, Style};
use clap::{ArgAction, Parser, Subcommand, builder::Styles};

use raven::{annotate_variants, query_variant};

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

#[derive(Subcommand)]
enum Subcmd {
    #[command(
        about = "Annotate variants in a VCF file.",
        arg_required_else_help = true
    )]
    Annotate(AnnotateArgs),
    #[command(about = "Annotate a single variant.", arg_required_else_help = true)]
    Query(QueryArgs),
}

#[derive(Parser)]
struct AnnotateArgs {
    /// Indexed input VCF
    #[arg(short, long)]
    input: String,

    /// Indexed VCF with known annotations. Can be specified multiple times.
    #[arg(short, long, action = ArgAction::Append)]
    vcf: Vec<String>,

    /// JSON-lines or VCF file to write annotation to
    #[arg(short, long)]
    output: String,

    /// Overwrites an existing file with the provided output file name
    #[arg(short = 'w', long)]
    overwrite: bool,
}

#[derive(Parser)]
struct QueryArgs {
    /// Variant to query (format must be CHROM:POS:REF:ALT)
    #[arg(short, long)]
    input: String,

    /// Indexed VCF with known annotations. Can be specified multiple times.
    #[arg(short, long, action = ArgAction::Append)]
    vcf: Vec<String>,

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
        Subcmd::Annotate(args) => {
            annotate_variants(args.input, args.vcf, args.output, args.overwrite)
        }
        Subcmd::Query(args) => query_variant(args.input, args.vcf, args.output, args.overwrite),
    }
}
