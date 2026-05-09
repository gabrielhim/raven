use serde_json::{self, Value, json};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};

use crate::models::{AnnotatedVariant, TagValueType, Variant};

pub fn format_variant_json(variant: &Variant, annotated: AnnotatedVariant) -> Value {
    let mut dataset_map = serde_json::Map::new();
    for (name, annot_record) in annotated.annotations {
        let mut tags_map = serde_json::Map::new();
        for tag_annot in annot_record.info_tag_values {
            let value = match &tag_annot.value {
                TagValueType::Float(f) => json!(f),
                TagValueType::Integer(i) => json!(i),
                TagValueType::Str(s) => Value::String(s.clone()),
            };
            tags_map.insert(tag_annot.name, value);
        }
        dataset_map.insert(
            name,
            json!({"id": Value::String(annot_record.record_id), "tags": Value::Object(tags_map)}),
        );
    }
    if let Some(_) = annotated.input_record {
        dataset_map.insert(
            String::from("record"),
            Value::String(annotated.input_record.unwrap()),
        );
    };
    json!({
        "chrom": variant.chromosome,
        "pos": variant.position + 1,
        "ref": variant.ref_allele,
        "alt": variant.alt_allele,
        "annotations": dataset_map
    })
}

pub fn write_json_output(contents: &[Value], output: &Option<String>, append: bool) {
    match output {
        Some(o) => {
            let out_file = if append {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(o)
                    .unwrap()
            } else {
                File::create(o).unwrap()
            };
            let mut writer = BufWriter::new(out_file);
            for content in contents {
                serde_json::to_writer(&mut writer, content).unwrap();
                writeln!(writer).unwrap();
            }
        }
        None => {
            for content in contents {
                println!("{}", serde_json::to_string_pretty(content).unwrap());
            }
        }
    }
}
