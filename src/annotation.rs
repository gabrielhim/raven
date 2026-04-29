use rust_htslib::bcf::{
    IndexedReader, Read, Record,
    header::{HeaderRecord, HeaderView},
};
use rust_htslib::errors::Error::GenomicSeek;
use serde_json::Value;

use crate::json::{format_variant_json, write_json_output};
use crate::models::{AnnotatedVariant, AnnotationRecord, Variant, VcfDataset};
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

fn move_to_next_record(reader: &mut IndexedReader) -> Option<Record> {
    let mut record = reader.empty_record();
    match reader.read(&mut record) {
        Some(Ok(_)) => Some(record),
        _ => None,
    }
}

pub fn annotate_single_variant(variant: &Variant, vcfs: &Vec<VcfDataset>) -> AnnotatedVariant {
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
            let ref_allele = &alleles[0];
            let alt_alleles = &alleles[1..];
            if variant.position == record.pos() as u64
                && ref_allele == &variant.ref_allele
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

    AnnotatedVariant {
        input_record: None,
        annotations: annotation_records,
    }
}

pub fn annotate_vcf(
    input_reader: &mut IndexedReader,
    vcfs: &Vec<VcfDataset>,
    keep_records: bool,
    output: String,
) {
    let mut vcf_readers: Vec<(&VcfDataset, IndexedReader, Option<Record>)> = vcfs
        .iter()
        .map(|vcf| {
            let mut reader = load_vcf(&vcf.file_path);
            let first_record = move_to_next_record(&mut reader);
            (vcf, reader, first_record)
        })
        .collect();

    let chromosomes = extract_contigs(input_reader.header());

    let output_file = Some(output);
    let mut append = false;

    for chrom in chromosomes {
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

        let mut annotated_variants: Vec<Value> = Vec::new();

        for vcf_record in input_reader.records() {
            let record = vcf_record.expect("Failed to read sample VCF record.");
            let record_str: Option<String> = if keep_records {
                record.to_vcf_string().ok()
            } else {
                None
            };
            let alleles = extract_alleles(&record);
            let ref_allele = &alleles[0];
            for alt in &alleles[1..] {
                let variant_str = format!("{}:{}:{}:{}", chrom, record.pos() + 1, ref_allele, alt);
                let variant = Variant::new(variant_str);

                let mut annotation_records: Vec<(String, AnnotationRecord)> = Vec::new();

                for (vcf_dataset, reader, curr_record) in vcf_readers.iter_mut() {
                    loop {
                        match curr_record {
                            Some(r) => {
                                let curr_chrom = {
                                    let curr_rid = r.rid().unwrap();
                                    String::from_utf8(
                                        reader.header().rid2name(curr_rid).unwrap().to_vec(),
                                    )
                                    .unwrap()
                                };
                                if curr_chrom == chrom && r.pos() == record.pos() {
                                    let curr_alleles = extract_alleles(&r);
                                    let curr_ref_allele = &curr_alleles[0];
                                    let curr_alt_alleles = &curr_alleles[1..];
                                    if curr_ref_allele == &variant.ref_allele
                                        && curr_alt_alleles.contains(&variant.alt_allele)
                                    {
                                        let annotation_record = AnnotationRecord {
                                            record_id: String::from_utf8(r.id()).unwrap(),
                                            info_tags: extract_tags_from_record(
                                                &r,
                                                &vcf_dataset.tag_names,
                                            ),
                                        };
                                        annotation_records.push((
                                            vcf_dataset.get_dataset_name(),
                                            annotation_record,
                                        ));
                                        *curr_record = move_to_next_record(reader);
                                        break;
                                    } else {
                                        *curr_record = move_to_next_record(reader);
                                    };
                                } else if r.pos() > record.pos() {
                                    break;
                                } else {
                                    *curr_record = move_to_next_record(reader);
                                }
                            }
                            None => break,
                        }
                    }
                }

                let annotated_variant = AnnotatedVariant {
                    input_record: record_str.clone(),
                    annotations: annotation_records,
                };
                let formatted_json = format_variant_json(&variant, annotated_variant);
                annotated_variants.push(formatted_json);
            }
        }

        write_json_output(&annotated_variants, &output_file, append);
        append = true;
    }
}
