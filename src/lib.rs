mod annotation;
mod contants;
mod models;
mod output;
mod vcf;

use annotation::{annotate_single_variant, annotate_vcf};
use models::{Variant, VcfDataset};
use output::{format_variant_json, write_json_output};
use std::time::Instant;
use vcf::load_vcf;

pub fn annotate_variants(vcf: String, vcfs: Vec<String>, output: String, overwrite: bool) {
    let now = Instant::now();
    let mut vcf = load_vcf(&vcf);
    let vcf_datasets: Vec<VcfDataset> = vcfs.iter().map(|vcf| VcfDataset::new(vcf)).collect();
    annotate_vcf(&mut vcf, &vcf_datasets, output, overwrite);
    let elapsed_time = now.elapsed().as_secs_f32();
    if elapsed_time < 60.0 {
        println!("Elapsed time: {} seconds", elapsed_time);
    } else {
        println!("Elapsed time: {} minutes", elapsed_time / 60.0);
    }
}

pub fn query_variant(variant: String, vcfs: Vec<String>, output: Option<String>, overwrite: bool) {
    let variant = Variant::new(variant);
    let vcf_datasets: Vec<VcfDataset> = vcfs.iter().map(|vcf| VcfDataset::new(vcf)).collect();
    let annotations = annotate_single_variant(&variant, &vcf_datasets);
    let variant_output = format_variant_json(&variant, annotations);
    write_json_output(&[variant_output], &output, overwrite, false);
}
