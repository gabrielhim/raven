# Raven :black_bird:

Raven is a variant handling and annotation command-line tool. It implements `rust-htslib` for fast variant processing.

## Installation

One way to install raven is to clone the project and run `cargo install`. This requires Rust and the Cargo package, follow the instructions in the [Rust Book](https://doc.rust-lang.org/book/ch01-01-installation.html) to install them.

Once you are ready, run the following commands
```bash
git clone https://github.com/gabrielhim/raven.git
cd raven
cargo install --path .
raven --help
```

If you are using a Linux environment, you can also download the binary directly from the release and copy it to a directory in $PATH:
```bash
wget https://github.com/gabrielhim/raven/releases/download/v0.1.0/raven
chmod +x raven
cp raven /usr/local/bin/
raven --help
```

## Usage

### annotate

Annotates an input VCF with information from VCF files containing known variants. Each VCF dataset is specified in the `-v` (or `--vcf`) parameter, which can be repeated multiple times:
```bash
raven annotate -i HG005_exome_20.vcf.gz -v clinvar_20.vcf.gz -v gnomad.exomes.r2.1.1.sites.20.vcf.bgz -o output.json
```

By default, raven annotates all INFO tags. If you want to direct annotation to specific tags, specify them separated by slash with their respective dataset:
```bash
raven annotate -i HG005_exome_20.vcf.gz -v clinvar_20.vcf.gz,ALLELEID/CLNHGVS/CLNSIG/CLNREVSTAT/GENEINFO -v gnomad.exomes.r2.1.1.sites.20.vcf.bgz,AC/AF/AN/segdup/lcr -o output.json
```

Output is written to a file in JSON-lines format:
```json
{"20:76962:T:C":{"gnomad.exomes.r2.1.1.sites.20":{"id":"rs6111385","tags":{"AC":197171,"AF":0.7841730117797852,"AN":251438}}}}
{"20:126310:ACC:A":{"clinvar_20":{"id":"402586","tags":{"ALLELEID":390451,"CLNHGVS":"NC_000020.10:g.126314_126315del","CLNREVSTAT":"criteria_provided,_single_submitter","CLNSIG":"Benign","GENEINFO":"DEFB126:81623"}},"gnomad.exomes.r2.1.1.sites.20":{"id":"rs111739970","tags":{"AC":137216,"AF":0.5522220134735107,"AN":248480}}}}
{"20:138125:G:T":{}}
```

Check `raven annotate --help` for other command options.

### query

Looks for a given variant in VCF files with known variants. Variant is specified in the format `{chromosome}:{position}:{ref_allele}:{alt_allele}` and VCFs are provided using `-v` just like for the `annotate` command:
```bash
raven query -i 20:400365:A:G -v clinvar_20.vcf.gz,ALLELEID/CLNHGVS/CLNSIG/CLNREVSTAT/GENEINFO -v gnomad.exomes.r2.1.1.sites.20.vcf.bgz,AC/AF/AN
```

If no output file is specified, `query` directs the output to stdout:
```
{
  "20:400365:A:G": {
    "clinvar_20": {
      "id": "573181",
      "tags": {
        "ALLELEID": 573501,
        "CLNHGVS": "NC_000020.10:g.400365A>G",
        "CLNREVSTAT": "criteria_provided,_conflicting_classifications",
        "CLNSIG": "Conflicting_classifications_of_pathogenicity",
        "GENEINFO": "RBCK1:10616"
      }
    },
    "gnomad.exomes.r2.1.1.sites.20": {
      "id": "rs763971799",
      "tags": {
        "AC": 63,
        "AF": 0.00038754899287596345,
        "AN": 162560
      }
    }
  }
}
```
