use ryu;
use serde_json::{self, Value, json};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};

use crate::models::{AnnotatedVariant, TagValueType, Variant};

// Value::Number stores f32 as f64, which introduces decimal digits.
// Using ryu to convert to &str and parsing it back gives the shortest
// decimal, which is saved as f64 with the f32 precision.
fn convert_f32(number: f32) -> Value {
    let mut buffer = ryu::Buffer::new();
    let s = buffer.format(number);
    Value::Number(s.parse().unwrap())
}

pub fn format_variant_json(variant: &Variant, annotated: AnnotatedVariant) -> Value {
    let mut dataset_map = serde_json::Map::new();
    for (name, annot_record) in annotated.annotations {
        let mut tags_map = serde_json::Map::new();
        for tag_annot in annot_record.info_tag_values {
            let value = match &tag_annot.value {
                TagValueType::Float(f) => convert_f32(*f),
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

    let mut output_record = json!({
        "chrom": variant.chromosome,
        "pos": variant.position + 1,
        "ref": variant.ref_allele,
        "alt": variant.alt_allele,
        "annotations": dataset_map
    });
    if let Some(i) = annotated.input_record {
        output_record["record"] = json!(i);
    };

    output_record
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
