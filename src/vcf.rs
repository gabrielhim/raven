use rust_htslib::bcf::{
    Format, IndexedReader, Record, Writer,
    header::{Header, HeaderRecord, HeaderView},
};
use std::cmp;
use std::collections::HashSet;
use std::process;

use crate::constants::INFO_ANNOTATION_SEPARATOR;
use crate::models::{AnnotatedVariant, InfoTagValue, TagValueType, VcfDataset};

pub fn check_if_chromosomes_match(header_view1: &HeaderView, header_view2: &HeaderView) -> bool {
    let contigs1 = extract_contigs(header_view1);
    let contigs2 = extract_contigs(header_view2);
    let smaller_len = cmp::min(contigs1.len(), contigs2.len());

    // Check the first 25 chromosomes, corresponding to 1-22, X, Y and MT. If any VCF
    // contains less than that (e.g. lacking MT), compare based on the smaller list.
    if smaller_len >= 25 {
        contigs1[0..25] == contigs2[0..25]
    } else {
        contigs1[0..smaller_len] == contigs2[0..smaller_len]
    }
}

pub fn extract_contigs(header_view: &HeaderView) -> Vec<String> {
    let mut contigs = Vec::new();
    for record in header_view.header_records() {
        if let HeaderRecord::Contig { key: _, values } = record {
            contigs.push(values.get("ID").unwrap().to_string())
        }
    }
    contigs
}

pub fn extract_tags_from_record(
    record: &Record,
    info_tags: &Vec<String>,
    empty_str_if_missing: bool,
) -> Vec<InfoTagValue> {
    let mut annotations: Vec<InfoTagValue> = Vec::new();
    for tag in info_tags {
        if let Ok(Some(values)) = record.info(tag.as_bytes()).string() {
            let decoded: Vec<&str> = values.iter().map(|x| str::from_utf8(x).unwrap()).collect();
            annotations.push(InfoTagValue {
                name: tag.clone(),
                value: TagValueType::Str(decoded.join(",")),
            });
        } else if let Ok(Some(values)) = record.info(tag.as_bytes()).integer() {
            annotations.push(InfoTagValue {
                name: tag.clone(),
                value: TagValueType::Integer(values[0]),
            });
        } else if let Ok(Some(values)) = record.info(tag.as_bytes()).float() {
            annotations.push(InfoTagValue {
                name: tag.clone(),
                value: TagValueType::Float(values[0]),
            });
        } else if empty_str_if_missing {
            annotations.push(InfoTagValue {
                name: tag.clone(),
                value: TagValueType::Str("".to_string()),
            });
        }
    }
    annotations
}

pub fn load_vcf(file_path: &str) -> IndexedReader {
    if file_path.ends_with(".vcf") {
        eprintln!("File '{}' must be compressed and indexed.", file_path);
        process::exit(1);
    }
    IndexedReader::from_path(file_path).expect("Failed to read VCF.")
}

pub fn get_info_tag_names(
    header: &HeaderView,
    vcf_path: &str,
    input_tags: Option<Vec<String>>,
) -> Vec<String> {
    let all_tags: Vec<String> = header
        .header_records()
        .iter()
        .filter_map(|x| {
            if let HeaderRecord::Info { values, .. } = x {
                values.get("ID").cloned()
            } else {
                None
            }
        })
        .collect();

    match input_tags {
        Some(x) => {
            let all_tags_set: HashSet<&str> = all_tags.iter().map(|s| s.as_str()).collect();
            let diff: Vec<&str> = x
                .iter()
                .map(|s| s.as_str())
                .filter(|t| !all_tags_set.contains(t))
                .collect();
            if !diff.is_empty() {
                eprintln!(
                    "INFO tags not present in '{}': {}",
                    vcf_path,
                    diff.join(", ")
                );
                process::exit(1);
            }
            x.to_vec()
        }
        None => all_tags,
    }
}

pub fn create_output_vcf(
    sample_header: &HeaderView,
    vcfs: &Vec<VcfDataset>,
    output: &String,
    uncompressed: bool,
) -> Writer {
    let mut header = Header::from_template(sample_header);
    for vcf in vcfs {
        let header_row = format!(
            "##INFO=<ID={},Number=1,Type=String,Description=\"{}\">",
            vcf.get_dataset_name(),
            vcf.tag_names.join(INFO_ANNOTATION_SEPARATOR)
        );
        header.push_record(header_row.as_bytes());
    }

    Writer::from_path(output, &header, uncompressed, Format::Vcf).unwrap()
}

pub fn format_output_record(
    record: &Record,
    annotated: AnnotatedVariant,
    writer: &mut Writer,
) -> Record {
    let mut output_record = record.clone();
    writer.translate(&mut output_record);
    for (dataset_name, annot_record) in annotated.annotations {
        let value_string = annot_record
            .info_tag_values
            .iter()
            .map(|tag| match &tag.value {
                TagValueType::Float(f) => f.to_string(),
                TagValueType::Integer(i) => i.to_string(),
                TagValueType::Str(s) => s.clone(),
            })
            .collect::<Vec<String>>()
            .join(INFO_ANNOTATION_SEPARATOR);

        output_record
            .push_info_string(dataset_name.as_bytes(), &[value_string.as_bytes()])
            .unwrap();
    }

    output_record
}
