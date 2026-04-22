use crate::contants::VCF_FILE_EXTENSIONS;
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
pub struct Variant<'a> {
    pub chromosome: &'a str,
    pub position: u64,
    pub ref_allele: &'a str,
    pub alt_allele: &'a str,
    pub variant_str: &'a str,
}

impl<'a> Variant<'a> {
    pub fn new(variant_str: &'a str) -> Self {
        let var_split: Vec<&str> = variant_str.trim().split(":").collect();
        assert_eq!(var_split.len(), 4, "Specify variant as CHROM:POS:REF:ALT.");

        let position: u64 = var_split[1].parse().unwrap_or_else(|_| {
            eprintln!("Position must be an integer.");
            process::exit(1);
        });
        Self {
            chromosome: var_split[0],
            position: position - 1, // rust-htslib uses 0-based position
            ref_allele: var_split[2],
            alt_allele: var_split[3],
            variant_str,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VcfDataset {
    pub file_path: String,
    pub tags: Option<Vec<String>>,
}

impl VcfDataset {
    pub fn new(dataset: &str) -> Self {
        let parsed: Vec<&str> = dataset.split(',').collect();
        let file_path = parsed[0].to_string();
        let tags = if parsed.len() > 1 {
            Some(parsed[1].split('/').map(String::from).collect())
        } else {
            None
        };
        Self { file_path, tags }
    }

    pub fn get_dataset_name(&self) -> &str {
        let extensions = VCF_FILE_EXTENSIONS;
        let mut basename: &str = self.file_path.split('/').last().unwrap();
        for ext in extensions {
            if let Some(x) = basename.strip_suffix(ext) {
                basename = x;
            }
        }
        basename
    }
}

#[derive(Clone, Debug)]
pub struct VcfAnnotation {
    pub dataset: VcfDataset,
    pub record_id: String,
    pub info_tags: Vec<InfoTag>,
}
