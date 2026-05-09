mod annotation;
mod constants;
mod json;
mod models;
mod vcf;

use annotation::{annotate_single_variant, annotate_vcf};
use clap::ValueEnum;
use constants::{JSON_FILE_EXTENSIONS, VCF_FILE_EXTENSIONS};
use json::write_json_output;
use models::{OutputFormat, Variant, VcfDataset};
use std::{fs, process, time::Instant};
use vcf::load_vcf;

fn check_if_output_exists(output: &str, overwrite: bool) {
    if !overwrite && fs::exists(output).unwrap() {
        eprintln!(
            "File '{}' already exists. Enable the overwrite option.",
            output
        );
        process::exit(1);
    };
}

pub fn annotate_variants(
    vcf: String,
    vcfs: Vec<String>,
    keep_records: bool,
    output: String,
    overwrite: bool,
) {
    let now = Instant::now();
    check_if_output_exists(&output, overwrite);

    let output_format = if JSON_FILE_EXTENSIONS.iter().any(|p| output.ends_with(p)) {
        OutputFormat::Json
    } else if VCF_FILE_EXTENSIONS.iter().any(|p| output.ends_with(p)) {
        OutputFormat::Vcf
    } else {
        eprintln!(
            "Invalid output extension. Output formats supported: {}",
            OutputFormat::value_variants()
                .iter()
                .map(|v| v.to_possible_value().unwrap().get_name().to_uppercase())
                .collect::<Vec<_>>()
                .join(", ")
        );
        process::exit(1);
    };

    let mut vcf = load_vcf(&vcf);
    let vcf_datasets: Vec<VcfDataset> = vcfs.iter().map(|vcf| VcfDataset::new(vcf)).collect();
    annotate_vcf(&mut vcf, &vcf_datasets, keep_records, output, output_format);

    let elapsed_time = now.elapsed().as_secs_f32();
    if elapsed_time < 60.0 {
        println!("Elapsed time: {:.2} seconds", elapsed_time);
    } else {
        println!("Elapsed time: {:.2} minutes", elapsed_time / 60.0);
    }
}

pub fn query_variant(variant: String, vcfs: Vec<String>, output: Option<String>, overwrite: bool) {
    if let Some(o) = &output {
        check_if_output_exists(o, overwrite);
    }

    let variant = Variant::new(variant);
    let vcf_datasets: Vec<VcfDataset> = vcfs.iter().map(|vcf| VcfDataset::new(vcf)).collect();
    let variant_output = annotate_single_variant(&variant, &vcf_datasets);
    write_json_output(&[variant_output], &output, false);
}
