use rust_htslib::bcf::{
    IndexedReader, Record,
    header::{HeaderRecord::Info, HeaderView},
};
use std::collections::HashSet;
use std::process;

use crate::models::{InfoTag, TagValueType};

pub fn extract_tags_from_record(record: &Record, info_tags: &Vec<String>) -> Vec<InfoTag> {
    let mut annotations: Vec<InfoTag> = Vec::new();
    for tag in info_tags {
        if let Ok(Some(values)) = record.info(tag.as_bytes()).string() {
            let decoded: Vec<&str> = values.iter().map(|x| str::from_utf8(x).unwrap()).collect();
            annotations.push(InfoTag {
                name: tag.clone(),
                value: TagValueType::Str(decoded.join(",")),
            });
        } else if let Ok(Some(values)) = record.info(tag.as_bytes()).integer() {
            annotations.push(InfoTag {
                name: tag.clone(),
                value: TagValueType::Integer(values[0]),
            });
        } else if let Ok(Some(values)) = record.info(tag.as_bytes()).float() {
            annotations.push(InfoTag {
                name: tag.clone(),
                value: TagValueType::Float(values[0]),
            });
        }
    }
    annotations
}

pub fn load_vcf(file_path: &str) -> IndexedReader {
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
            if let Info { values, .. } = x {
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
                eprintln!("INFO tags not present in {}: {:?}", vcf_path, diff);
                process::exit(1);
            }
            x.to_vec()
        }
        None => all_tags,
    }
}
