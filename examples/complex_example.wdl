version 1.1

import "https://raw.githubusercontent.com/broadinstitute/wdl-tools/main/scripts/bioinformatics.wdl" as bio
import "util.wdl" as utils

struct Sample {
    String name
    File fastq1
    File fastq2
    String? library_id
}

struct AlignmentResults {
    File bam
    File bai
    Float alignment_rate
    Int total_reads
}

task align_reads {
    input {
        Sample sample
        File reference_genome
        File reference_index
        String aligner = "bwa"
        Int threads = 4
    }

    String base_name = basename(sample.fastq1, "_R1.fastq.gz")

    command <<<
        set -euo pipefail

        # Align reads
        if [ "~{aligner}" == "bwa" ]; then
            bwa mem -t ~{threads} ~{reference_genome} ~{sample.fastq1} ~{sample.fastq2} | \
                samtools sort -@ ~{threads} -o ~{base_name}.sorted.bam -
        else
            echo "Unknown aligner: ~{aligner}"
            exit 1
        fi

        # Index BAM
        samtools index ~{base_name}.sorted.bam

        # Calculate alignment stats
        samtools flagstat ~{base_name}.sorted.bam > ~{base_name}.flagstat

        # Extract metrics
        TOTAL_READS=$(grep "total" ~{base_name}.flagstat | cut -d' ' -f1)
        MAPPED_READS=$(grep "mapped (" ~{base_name}.flagstat | cut -d' ' -f1)
        ALIGNMENT_RATE=$(echo "scale=4; $MAPPED_READS / $TOTAL_READS" | bc)

        echo $TOTAL_READS > total_reads.txt
        echo $ALIGNMENT_RATE > alignment_rate.txt
    >>>

    output {
        AlignmentResults results = object {
            bam: "~{base_name}.sorted.bam",
            bai: "~{base_name}.sorted.bam.bai",
            alignment_rate: read_float("alignment_rate.txt"),
            total_reads: read_int("total_reads.txt")
        }
        File flagstat = "~{base_name}.flagstat"
    }

    runtime {
        docker: "biocontainers/bwa:v0.7.17-3-deb_cv1"
        memory: "8GB"
        cpu: threads
        disks: "local-disk 100 SSD"
    }

    meta {
        description: "Align paired-end reads using BWA-MEM"
    }

    parameter_meta {
        sample: "Sample information including FASTQ files"
        reference_genome: "Reference genome FASTA file"
        reference_index: "BWA index files"
        aligner: "Alignment tool to use"
        threads: "Number of threads for alignment"
    }
}

task call_variants {
    input {
        AlignmentResults alignment
        File reference_genome
        File? known_sites
        String sample_name
    }

    command <<<
        set -euo pipefail

        # Call variants with GATK HaplotypeCaller
        gatk HaplotypeCaller \
            -R ~{reference_genome} \
            -I ~{alignment.bam} \
            -O ~{sample_name}.vcf.gz \
            ~{if defined(known_sites) then "--dbsnp " + known_sites else ""}

        # Index VCF
        gatk IndexFeatureFile -I ~{sample_name}.vcf.gz

        # Basic variant stats
        bcftools stats ~{sample_name}.vcf.gz > ~{sample_name}.vcf.stats
    >>>

    output {
        File vcf = "~{sample_name}.vcf.gz"
        File vcf_index = "~{sample_name}.vcf.gz.tbi"
        File stats = "~{sample_name}.vcf.stats"
    }

    runtime {
        docker: "broadinstitute/gatk:4.2.6.1"
        memory: "16GB"
        cpu: 2
        disks: "local-disk 50 SSD"
    }
}

workflow genomics_pipeline {
    input {
        Array[Sample] samples
        File reference_genome
        File reference_index
        File? known_variants
        String output_prefix = "results"
    }

    # Align all samples
    scatter (sample in samples) {
        call align_reads {
            input:
                sample = sample,
                reference_genome = reference_genome,
                reference_index = reference_index
        }

        call call_variants {
            input:
                alignment = align_reads.results,
                reference_genome = reference_genome,
                known_sites = known_variants,
                sample_name = sample.name
        }
    }

    # Merge VCFs if multiple samples
    if (length(samples) > 1) {
        call utils.merge_vcfs {
            input:
                vcfs = call_variants.vcf,
                output_name = "~{output_prefix}.merged.vcf.gz"
        }
    }

    # Calculate summary statistics
    call bio.calculate_pipeline_stats {
        input:
            alignment_results = align_reads.results,
            vcf_files = call_variants.vcf
    }

    output {
        Array[AlignmentResults] alignments = align_reads.results
        Array[File] vcfs = call_variants.vcf
        Array[File] variant_stats = call_variants.stats
        File? merged_vcf = utils.merge_vcfs.merged_vcf
        File pipeline_summary = bio.calculate_pipeline_stats.summary
    }

    meta {
        author: "Genomics Team"
        email: "genomics@example.com"
        description: "Complete genomics pipeline from FASTQ to VCF"
        version: "1.0.0"
    }

    parameter_meta {
        samples: "Array of sample information with FASTQ files"
        reference_genome: "Reference genome FASTA file"
        reference_index: "BWA index for reference genome"
        known_variants: "Optional known variants file for GATK"
        output_prefix: "Prefix for output files"
    }
}
