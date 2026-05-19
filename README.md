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

Alternatively, you can build a Docker image with raven by running `docker build` in the root of the cloned project:
```bash
docker build -t raven:v0.2 .
docker run --rm -it raven:v0.2 bash
raven --help
```

If you are using a Linux environment, you can also download the binary directly from the release and copy it to a directory in $PATH:
```bash
wget https://github.com/gabrielhim/raven/releases/download/v0.2.1/raven
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

The output format is inferred by the output file extension. Two formats are currently supported:

**JSON**

Output is written to a file in JSON-lines format:
```json
{"alt":"C","annotations":{"gnomad.exomes.r2.1.1.sites.20":{"id":"rs6111385","tags":{"AC":197171,"AF":0.7841730117797852,"AN":251438}}},"chrom":"20","pos":76962,"ref":"T"}
{"alt":"A","annotations":{"clinvar_20":{"id":"402586","tags":{"ALLELEID":390451,"CLNHGVS":"NC_000020.10:g.126314_126315del","CLNREVSTAT":"criteria_provided,_single_submitter","CLNSIG":"Benign","GENEINFO":"DEFB126:81623"}},"gnomad.exomes.r2.1.1.sites.20":{"id":"rs111739970","tags":{"AC":137216,"AF":0.5522220134735107,"AN":248480}}},"chrom":"20","pos":126310,"ref":"ACC"}
{"alt":"T","annotations":{},"chrom":"20","pos":138125,"ref":"G"}
```

**VCF**

Output is written to a VCF:
```
##fileformat=VCFv4.1
(...)
##INFO=<ID=clinvar_20,Number=1,Type=String,Description="CLNSIG?CLNSIGCONF?CLNREVSTAT">
##INFO=<ID=gnomad.exomes.r2.1.1.sites.20,Number=1,Type=String,Description="AF?AC?AN">
#CHROM	POS	ID	REF	ALT	QUAL	FILTER	INFO	FORMAT	Sample_Diag-excap51-HG005-EEogPU
20	76962	rs6111385	T	C	8033.77	PASS	AC=1;AF=0.5;AN=2;BaseQRankSum=-2.805;ClippingRankSum=1.177;DB;DP=612;FS=0.517;MLEAC=1;MLEAF=0.5;MQ=60;MQRankSum=-0.373;POSITIVE_TRAIN_SITE;QD=13.13;ReadPosRankSum=1.744;SOR=0.734;VQSLOD=15.9;culprit=MQ;gnomad.exomes.r2.1.1.sites.20=0.784173?197171?251438	GT:AD:DP:GQ:PL	0/1:288,324:612:99:8062,0,7483
20	126310	rs140685149	ACC	A	20197.7	PASS	AC=2;AF=1;AN=2;DB;DP=483;FS=0;MLEAC=2;MLEAF=1;MQ=60;QD=31.2;SOR=1.646;clinvar_20=Benign??criteria_provided,_single_submitter;gnomad.exomes.r2.1.1.sites.20=0.552222?137216?248480	GT:AD:DP:GQ:PL	1/1:0,458:458:99:20235,1382,0
20	138125	rs2298108	G	T	1575.77	PASS	AC=2;AF=1;AN=2;DB;DP=49;FS=0;MLEAC=2;MLEAF=1;MQ=60;POSITIVE_TRAIN_SITE;QD=32.16;SOR=1.609;VQSLOD=22.12;culprit=MQ	GT:AD:DP:GQ:PL	1/1:0,49:49:99:1604,146,0
```

Raven compresses the output VCF if the specified file name ends with ".gz".

Check `raven annotate --help` for other command options.

### query

Looks for a given variant in VCF files with known variants. Variant is specified in the format `{chromosome}:{position}:{ref_allele}:{alt_allele}` and VCFs are provided using `-v` just like for the `annotate` command:
```bash
raven query -i 20:400365:A:G -v clinvar_20.vcf.gz,ALLELEID/CLNHGVS/CLNSIG/CLNREVSTAT/GENEINFO -v gnomad.exomes.r2.1.1.sites.20.vcf.bgz,AC/AF/AN
```

The only output format supported is JSON. If no file is specified, `query` directs the output to stdout:
```
{
  "alt": "G",
  "annotations": {
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
  },
  "chrom": "20",
  "pos": 400365,
  "ref": "A"
}
```
