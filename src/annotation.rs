use rust_htslib::bcf::Read;
use std::collections::HashMap;
use std::process;

use crate::models::{TagValue, Variant, VcfDataset};
use crate::vcf::{extract_tags_from_record, get_header_info_tags, load_vcf};

pub fn annotate_variant(
    variant: &Variant,
    vcf_datasets: Vec<VcfDataset>,
) -> HashMap<String, Vec<TagValue>> {
    let mut annot_by_dataset: HashMap<String, Vec<TagValue>> = HashMap::new();
    for dataset in vcf_datasets {
        let mut vcf = load_vcf(dataset.file_path);
        let vcf_header = vcf.header();

        let info_tags = get_header_info_tags(vcf_header, dataset.clone().tags, dataset.file_path);

        let rid = vcf_header.name2rid(variant.chromosome.as_bytes()).unwrap();
        match vcf.fetch(rid, variant.position, Some(variant.position)) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
        for vcf_record in vcf.records() {
            let record = vcf_record.expect("Failed to read record.");
            let alleles: Vec<&str> = record
                .alleles()
                .iter()
                .map(|x| str::from_utf8(x).unwrap())
                .collect();
            let ref_allele = alleles[0];
            let alt_alleles = &alleles[1..];
            if variant.position == record.pos() as u64
                && variant.ref_allele == ref_allele
                && alt_alleles.contains(&variant.alt_allele)
            {
                let tag_annotations = extract_tags_from_record(record, &info_tags);
                annot_by_dataset.insert(dataset.get_dataset_name().to_string(), tag_annotations);
            }
        }
    }
    annot_by_dataset
}
