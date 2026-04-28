mod annotation;
mod contants;
mod json;
mod models;
mod vcf;

use annotation::{annotate_single_variant, annotate_vcf};
use json::{format_variant_json, write_json_output};
use models::{Variant, VcfDataset};
use std::fs;
use std::process;
use std::time::Instant;
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
    check_if_output_exists(&output, overwrite);
    let now = Instant::now();

    let mut vcf = load_vcf(&vcf);
    let vcf_datasets: Vec<VcfDataset> = vcfs.iter().map(|vcf| VcfDataset::new(vcf)).collect();
    annotate_vcf(&mut vcf, &vcf_datasets, keep_records, output);

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
    let annotated_variant = annotate_single_variant(&variant, &vcf_datasets);
    let variant_output = format_variant_json(&variant, annotated_variant);
    write_json_output(&[variant_output], &output, false);
}
