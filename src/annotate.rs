use rust_htslib::bcf::Read;
use std::process;

use crate::models::{Variant, VcfAnnotation, VcfDataset};
use crate::vcf::{extract_tags_from_record, get_info_tags, load_vcf};

pub fn annotate_variant(variant: &Variant, vcf_datasets: Vec<VcfDataset>) -> Vec<VcfAnnotation> {
    let mut annotations: Vec<VcfAnnotation> = Vec::new();
    for dataset in vcf_datasets {
        let mut vcf = load_vcf(&dataset.file_path);
        let vcf_header = vcf.header();

        let info_tags = get_info_tags(vcf_header, dataset.clone().tags, &dataset.file_path);

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
                let tag_annotations = extract_tags_from_record(&record, &info_tags);
                annotations.push(VcfAnnotation {
                    dataset,
                    record_id: String::from_utf8(record.id()).unwrap(),
                    info_tags: tag_annotations,
                });
                break;
            }
        }
    }
    annotations
}
