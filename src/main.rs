mod colors;
mod model;
mod renderer;
mod tshark;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::Path;

/// Flowster - Generate PDF signaling flow diagrams from pcap/pcapng captures.
///
/// Uses tshark (Wireshark CLI) to dissect packets and renders a
/// VisualEther-style sequence diagram showing message flow between
/// network nodes.
#[derive(Parser, Debug)]
#[command(name = "flowster", version, about, long_about = None)]
struct Cli {
    /// Input pcap or pcapng file path
    #[arg(value_name = "INPUT")]
    input: String,

    /// Output PDF file path [default: <input>.pdf]
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<String>,

    /// Wireshark display filter (e.g. "sip || rtp", "ip.addr == 10.0.0.1")
    #[arg(short = 'f', long, value_name = "FILTER")]
    filter: Option<String>,

    /// Maximum number of packets to include in the diagram
    #[arg(short = 'n', long, value_name = "COUNT")]
    max_packets: Option<usize>,

    /// Title displayed at the top of the first page
    #[arg(short, long, value_name = "TITLE")]
    title: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate input file exists
    let input_path = &cli.input;
    if !Path::new(input_path).exists() {
        anyhow::bail!("Input file '{}' does not exist", input_path);
    }

    // Determine output path
    let output_path = cli.output.unwrap_or_else(|| {
        let stem = Path::new(input_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        format!("{}.pdf", stem)
    });

    // Step 1: Parse packets via tshark
    eprintln!("[1/3] Parsing {} with tshark...", input_path);
    let packets =
        tshark::parse_pcapng(input_path, cli.filter.as_deref(), cli.max_packets)
            .context("Failed to parse capture file")?;

    eprintln!(
        "      Found {} packets, {} unique endpoints",
        packets.len(),
        count_unique_endpoints(&packets),
    );

    // Step 2: Build the signaling diagram model
    eprintln!("[2/3] Building signaling diagram...");
    let diagram = model::SignalingDiagram::from_packets(&packets);

    eprintln!(
        "      {} nodes, {} messages, {} pages",
        diagram.nodes.len(),
        diagram.messages.len(),
        estimate_pages(diagram.messages.len()),
    );

    // Step 3: Render to PDF
    eprintln!("[3/3] Rendering PDF to {}...", output_path);
    renderer::render_pdf(&diagram, &output_path, cli.title.as_deref())
        .context("Failed to render PDF")?;

    eprintln!("Done! Output written to {}", output_path);

    Ok(())
}

fn count_unique_endpoints(packets: &[tshark::PacketInfo]) -> usize {
    let mut seen = std::collections::HashSet::new();
    for p in packets {
        seen.insert(&p.src);
        seen.insert(&p.dst);
    }
    seen.len()
}

fn estimate_pages(message_count: usize) -> usize {
    let msgs_per_page = 37;
    if message_count == 0 {
        1
    } else {
        (message_count + msgs_per_page - 1) / msgs_per_page
    }
}
