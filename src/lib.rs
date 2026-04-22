mod annotation;
mod contants;
mod models;
mod vcf;

use serde_json::{self, json};
use std::collections::HashMap;
use std::fs::File;
use std::process;

use annotation::annotate_variant;
use models::{TagValue, TagValueType, Variant, VcfDataset};

fn format_variant_json(
    variant: &Variant,
    annot_by_dataset: HashMap<String, Vec<TagValue>>,
) -> serde_json::Value {
    let mut dataset_map = serde_json::Map::new();
    for (dataset, annotations) in annot_by_dataset {
        let mut tags_map = serde_json::Map::new();
        for tag_annot in annotations {
            let value = match &tag_annot.value {
                TagValueType::Float(f) => json!(f),
                TagValueType::Integer(i) => json!(i),
                TagValueType::Str(s) => serde_json::Value::String(s.clone()),
            };
            tags_map.insert(tag_annot.tag, value);
        }
        dataset_map.insert(dataset, serde_json::Value::Object(tags_map));
    }
    json!({variant.variant_str: dataset_map})
}

fn write_json_output(variant: &serde_json::Value, output: &Option<String>, overwrite: bool) {
    match output {
        Some(o) => {
            let out_file = if !overwrite {
                File::create_new(o).unwrap_or_else(|err| {
                    eprintln!("Couldn't write file {}: {}", o, err);
                    process::exit(1);
                })
            } else {
                File::create(o).unwrap()
            };
            serde_json::to_writer(out_file, variant).unwrap();
        }
        None => println!("{}", serde_json::to_string_pretty(variant).unwrap()),
    }
}

pub fn query_variant(
    variant: &str,
    datasets: Vec<String>,
    output: Option<String>,
    overwrite: bool,
) {
    let variant = Variant::new(variant);
    let mut vcf_datasets: Vec<VcfDataset> = Vec::new();
    for dataset in &datasets {
        vcf_datasets.push(VcfDataset::new(dataset));
    }
    let annot_by_dataset = annotate_variant(&variant, vcf_datasets);
    let variant_output = format_variant_json(&variant, annot_by_dataset);
    write_json_output(&variant_output, &output, overwrite);
}
