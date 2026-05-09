use rust_htslib::bcf::{IndexedReader, Read, Record};
use rust_htslib::errors::Error::GenomicSeek;
use serde_json::Value;
use std::{collections::HashMap, process};

use crate::json::{format_variant_json, write_json_output};
use crate::models::{AnnotatedVariant, AnnotationRecord, OutputFormat, Variant, VcfDataset};
use crate::vcf::{
    check_if_chromosomes_match, create_output_vcf, extract_contigs, extract_tags_from_record,
    format_output_record, load_vcf,
};

fn extract_alleles(record: &Record) -> Vec<String> {
    record
        .alleles()
        .iter()
        .map(|x| String::from_utf8(x.to_vec()).unwrap())
        .collect()
}

fn move_to_next_record(reader: &mut IndexedReader) -> Option<Record> {
    let mut record = reader.empty_record();
    match reader.read(&mut record) {
        Some(Ok(_)) => Some(record),
        _ => None,
    }
}

pub fn annotate_single_variant(variant: &Variant, vcfs: &Vec<VcfDataset>) -> Value {
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
                let info_tag_values = extract_tags_from_record(&record, &vcf.tag_names, false);
                annotation_records.push((
                    vcf.get_dataset_name(),
                    AnnotationRecord {
                        record_id: String::from_utf8(record.id()).unwrap(),
                        info_tag_values,
                    },
                ));
                break;
            }
        }
    }

    let annotated_variant = AnnotatedVariant {
        input_record: None,
        annotations: annotation_records,
    };

    format_variant_json(&variant, annotated_variant)
}

pub fn annotate_vcf(
    input_reader: &mut IndexedReader,
    vcfs: &Vec<VcfDataset>,
    keep_records: bool,
    output: String,
    output_format: OutputFormat,
) {
    let mut vcf_readers: Vec<(&VcfDataset, IndexedReader, Option<Record>)> = Vec::new();
    for vcf in vcfs {
        let mut reader = load_vcf(&vcf.file_path);
        if !check_if_chromosomes_match(input_reader.header(), reader.header()) {
            eprintln!("Chromosomes from '{}' and input VCF don't match.", {
                &vcf.file_path
            });
            process::exit(1);
        };
        let first_record = move_to_next_record(&mut reader);
        vcf_readers.push((vcf, reader, first_record));
    }

    let mut output_vcf = match output_format {
        OutputFormat::Vcf => {
            let uncompressed = !output.ends_with(".gz");
            let output_vcf = create_output_vcf(input_reader.header(), vcfs, &output, uncompressed);
            Some(output_vcf)
        }
        _ => None,
    };

    let empty_str_if_missing = match output_format {
        OutputFormat::Json => false,
        OutputFormat::Vcf => true,
    };

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

        let mut annotated_json_out: Vec<Value> = Vec::new();

        let mut position_records: Vec<(String, HashMap<(String, String), AnnotationRecord>)> =
            Vec::new();
        let mut position: i64 = 0;

        for vcf_record in input_reader.records() {
            let record = vcf_record.expect("Failed to read sample VCF record.");
            let record_str: Option<String> = if keep_records {
                record.to_vcf_string().ok()
            } else {
                None
            };
            let alleles = extract_alleles(&record);
            let ref_allele = &alleles[0];

            if record.pos() != position {
                position = record.pos();
                position_records.clear();
                for (vcf_dataset, reader, ds_record_pointer) in vcf_readers.iter_mut() {
                    let mut dataset_records: HashMap<(String, String), AnnotationRecord> =
                        HashMap::new();
                    loop {
                        match ds_record_pointer {
                            Some(r) => {
                                let ds_rid = r.rid().unwrap();
                                if ds_rid == rid && r.pos() == record.pos() {
                                    let ds_alleles = extract_alleles(&r);
                                    let ds_ref_allele = &ds_alleles[0];
                                    for ds_alt in &ds_alleles[1..] {
                                        let annotation_record = AnnotationRecord {
                                            record_id: String::from_utf8(r.id()).unwrap(),
                                            info_tag_values: extract_tags_from_record(
                                                &r,
                                                &vcf_dataset.tag_names,
                                                empty_str_if_missing,
                                            ),
                                        };
                                        dataset_records.insert(
                                            (ds_ref_allele.clone(), ds_alt.clone()),
                                            annotation_record,
                                        );
                                    }
                                    *ds_record_pointer = move_to_next_record(reader);
                                } else if (ds_rid == rid && r.pos() > record.pos()) || ds_rid > rid
                                {
                                    break;
                                } else {
                                    *ds_record_pointer = move_to_next_record(reader);
                                }
                            }
                            None => break,
                        }
                    }
                    position_records.push((vcf_dataset.get_dataset_name(), dataset_records));
                }
            }

            for alt in &alleles[1..] {
                let variant = Variant {
                    chromosome: chrom.clone(),
                    position: record.pos() as u64,
                    ref_allele: ref_allele.clone(),
                    alt_allele: alt.clone(),
                };

                let mut annotation_records: Vec<(String, AnnotationRecord)> = Vec::new();
                for (dataset_name, dataset_records) in &position_records {
                    let maybe_annotation = dataset_records.get(&(ref_allele.clone(), alt.clone()));
                    if let Some(a) = maybe_annotation {
                        annotation_records.push((dataset_name.clone(), a.clone()));
                    }
                }

                let annotated_variant = AnnotatedVariant {
                    input_record: record_str.clone(),
                    annotations: annotation_records,
                };

                match output_format {
                    OutputFormat::Json => {
                        let out_value = format_variant_json(&variant, annotated_variant);
                        annotated_json_out.push(out_value);
                    }
                    OutputFormat::Vcf => {
                        let out_record = format_output_record(
                            &record,
                            annotated_variant,
                            &mut output_vcf.as_mut().unwrap(),
                        );
                        output_vcf.as_mut().unwrap().write(&out_record).unwrap();
                    }
                };
            }
        }

        if annotated_json_out.len() > 0 {
            write_json_output(&annotated_json_out, &output_file, append);
        }
        append = true;
    }
}
