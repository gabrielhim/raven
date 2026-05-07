use rust_htslib::bcf::{Read, Reader};

use crate::contants::VCF_FILE_EXTENSIONS;
use crate::vcf::get_info_tag_names;
use std::process;

#[derive(Clone, Debug)]
pub enum TagValueType {
    Float(f32),
    Integer(i32),
    Str(String),
}

#[derive(Clone, Debug)]
pub struct InfoTag {
    pub name: String,
    pub value: TagValueType,
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub chromosome: String,
    pub position: u64,
    pub ref_allele: String,
    pub alt_allele: String,
}

impl Variant {
    pub fn new(variant_str: String) -> Self {
        let var_split: Vec<String> = variant_str.trim().split(":").map(String::from).collect();
        assert_eq!(var_split.len(), 4, "Specify variant as CHROM:POS:REF:ALT.");

        let position: u64 = var_split[1].parse().unwrap_or_else(|_| {
            eprintln!("Position must be an integer.");
            process::exit(1);
        });
        Self {
            chromosome: var_split[0].clone(),
            position: position - 1, // rust-htslib uses 0-based position
            ref_allele: var_split[2].clone(),
            alt_allele: var_split[3].clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct VcfDataset {
    pub file_path: String,
    pub tag_names: Vec<String>,
}

impl VcfDataset {
    pub fn new(dataset: &str) -> Self {
        let parsed: Vec<&str> = dataset.split(',').collect();
        let file_path = parsed[0].to_string();
        let reader = Reader::from_path(&file_path).unwrap();
        let vcf_header = reader.header();
        let tag_names = if parsed.len() == 1 {
            get_info_tag_names(&vcf_header, &file_path, None)
        } else {
            let tags = Some(parsed[1].split('/').map(String::from).collect());
            get_info_tag_names(&vcf_header, &file_path, tags)
        };
        drop(reader);
        Self {
            file_path,
            tag_names,
        }
    }

    pub fn get_dataset_name(&self) -> String {
        let mut basename = self.file_path.split('/').last().unwrap();
        for ext in VCF_FILE_EXTENSIONS {
            if let Some(x) = basename.strip_suffix(ext) {
                basename = x;
            }
        }
        basename.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct AnnotationRecord {
    pub record_id: String,
    pub info_tags: Vec<InfoTag>,
}

#[derive(Debug)]
pub struct AnnotatedVariant {
    pub input_record: Option<String>,
    pub annotations: Vec<(String, AnnotationRecord)>,
}
