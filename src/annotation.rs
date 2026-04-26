use rust_htslib::bcf::{
    IndexedReader, Read, Record,
    header::{HeaderRecord, HeaderView},
};
use rust_htslib::errors::Error::GenomicSeek;
use std::collections::HashMap;

use crate::models::{AnnotationRecord, Variant, VcfDataset};
use crate::output::{format_variant_json, write_json_output};
use crate::vcf::{extract_tags_from_record, load_vcf};

fn extract_alleles(record: &Record) -> Vec<String> {
    record
        .alleles()
        .iter()
        .map(|x| String::from_utf8(x.to_vec()).unwrap())
        .collect()
}

fn extract_contigs(header_view: &HeaderView) -> Vec<String> {
    let mut contigs = Vec::new();
    for record in header_view.header_records() {
        if let HeaderRecord::Contig { key: _, values } = record {
            contigs.push(values.get("ID").unwrap().to_string())
        }
    }
    contigs
}

pub fn annotate_single_variant(
    variant: &Variant,
    vcfs: &Vec<VcfDataset>,
) -> Vec<(String, AnnotationRecord)> {
    let mut annotation_records: Vec<(String, AnnotationRecord)> = Vec::new();
    for vcf in vcfs {
        let mut reader = load_vcf(&vcf.file_path);
        let rid = reader
            .header()
            .name2rid(variant.chromosome.as_bytes())
            .unwrap();
        match reader.fetch(rid, variant.position, Some(variant.position)) {
            Ok(_) => (),
            Err(e) => match e {
                GenomicSeek {
                    contig: _,
                    start: _,
                } => continue,
                _ => panic!("{}", e),
            },
        }

        for vcf_record in reader.records() {
            let record = vcf_record.expect("Failed to read VCF dataset record.");
            let alleles = extract_alleles(&record);
            let ref_allele = alleles[0].clone();
            let alt_alleles = &alleles[1..];
            if variant.position == record.pos() as u64
                && variant.ref_allele == ref_allele
                && alt_alleles.contains(&variant.alt_allele)
            {
                let info_tags = extract_tags_from_record(&record, &vcf.tag_names);
                annotation_records.push((
                    vcf.get_dataset_name(),
                    AnnotationRecord {
                        record_id: String::from_utf8(record.id()).unwrap(),
                        info_tags,
                    },
                ));
                break;
            }
        }
    }
    annotation_records
}

pub fn annotate_vcf(
    input_reader: &mut IndexedReader,
    vcfs: &Vec<VcfDataset>,
    output: String,
    overwrite: bool,
) {
    let mut vcf_readers: Vec<(&VcfDataset, IndexedReader)> = vcfs
        .iter()
        .map(|vcf| (vcf, load_vcf(&vcf.file_path)))
        .collect();

    let chromosomes = extract_contigs(input_reader.header());

    let output_file = Some(output);
    let mut append = false;

    for chrom in chromosomes {
        let mut datasets_cache: Vec<(String, HashMap<(u64, String, String), AnnotationRecord>)> =
            Vec::new();

        for (vcf, vcf_reader) in vcf_readers.iter_mut() {
            let rid = match vcf_reader.header().name2rid(chrom.as_bytes()) {
                Ok(r) => r,
                Err(_) => continue,
            };

            match vcf_reader.fetch(rid, 0, None) {
                Ok(_) => (),
                Err(e) => match e {
                    GenomicSeek {
                        contig: _,
                        start: _,
                    } => continue,
                    _ => panic!("{}", e),
                },
            }

            let mut records_map: HashMap<(u64, String, String), AnnotationRecord> = HashMap::new();
            for vcf_record in vcf_reader.records() {
                let record = vcf_record.expect("Failed to read VCF dataset record.");
                let tag_annotations = extract_tags_from_record(&record, &vcf.tag_names);
                let alleles = extract_alleles(&record);
                let ref_allele = alleles[0].clone();
                for alt in &alleles[1..] {
                    let record_key = (record.pos() as u64, ref_allele.to_string(), alt.to_string());
                    let annotation_record = AnnotationRecord {
                        record_id: String::from_utf8(record.id()).unwrap(),
                        info_tags: tag_annotations.clone(),
                    };
                    records_map.insert(record_key, annotation_record);
                }
            }

            datasets_cache.push((vcf.get_dataset_name(), records_map));
        }

        let rid = match input_reader.header().name2rid(chrom.as_bytes()) {
            Ok(r) => r,
            Err(_) => continue,
        };

        match input_reader.fetch(rid, 0, None) {
            Ok(_) => (),
            Err(e) => match e {
                GenomicSeek {
                    contig: _,
                    start: _,
                } => continue,
                _ => panic!("{}", e),
            },
        }

        for vcf_record in input_reader.records() {
            let record = vcf_record.expect("Failed to read sample VCF record.");
            let position = record.pos();
            let alleles = extract_alleles(&record);
            let ref_allele = alleles[0].clone();
            for alt in &alleles[1..] {
                let variant_str = format!("{}:{}:{}:{}", chrom, position + 1, &ref_allele, alt);
                let variant = Variant::new(variant_str);

                let mut annotation_records: Vec<(String, AnnotationRecord)> = Vec::new();
                for (vcf_basename, records_map) in &datasets_cache {
                    let record_key = (
                        variant.position,
                        variant.ref_allele.clone(),
                        variant.alt_allele.clone(),
                    );
                    if records_map.contains_key(&record_key) {
                        let record = records_map.get(&record_key).unwrap();
                        annotation_records.push((vcf_basename.clone(), record.clone()));
                    }
                }

                let formatted_json = format_variant_json(&variant, annotation_records);
                write_json_output(&[formatted_json], &output_file, overwrite, append);
                append = true;
            }
        }
    }
}
