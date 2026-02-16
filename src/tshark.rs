use anyhow::{bail, Context, Result};
use std::process::Command;

/// Represents a single parsed packet from tshark output.
#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub frame_number: u32,
    pub time_relative: f64,
    pub src: String,
    pub dst: String,
    pub protocol: String,
    pub info: String,
}

/// Run tshark on the given pcapng/pcap file and return structured packet info.
pub fn parse_pcapng(
    file_path: &str,
    display_filter: Option<&str>,
    max_packets: Option<usize>,
) -> Result<Vec<PacketInfo>> {
    let mut cmd = Command::new("tshark");
    cmd.args([
        "-r",
        file_path,
        "-T",
        "fields",
        "-e",
        "frame.number",
        "-e",
        "frame.time_relative",
        "-e",
        "ip.src",
        "-e",
        "ip.dst",
        "-e",
        "ipv6.src",
        "-e",
        "ipv6.dst",
        "-e",
        "eth.src",
        "-e",
        "eth.dst",
        "-e",
        "_ws.col.Protocol",
        "-e",
        "_ws.col.Info",
        "-E",
        "separator=\t",
        "-E",
        "quote=n",
        "-E",
        "header=n",
    ]);

    if let Some(filter) = display_filter {
        cmd.args(["-Y", filter]);
    }

    let output = cmd
        .output()
        .context("Failed to run tshark. Is Wireshark/tshark installed and in PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("tshark exited with error: {}", stderr.trim());
    }

    let stdout = String::from_utf8(output.stdout)
        .context("tshark output contained invalid UTF-8")?;

    let mut packets = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 10 {
            continue;
        }

        let frame_number: u32 = fields[0].parse().unwrap_or(0);
        let time_relative: f64 = fields[1].parse().unwrap_or(0.0);

        // Prefer IP > IPv6 > Ethernet for source/destination addressing
        let (src, dst) = if !fields[2].is_empty() && !fields[3].is_empty() {
            (fields[2].to_string(), fields[3].to_string())
        } else if !fields[4].is_empty() && !fields[5].is_empty() {
            (fields[4].to_string(), fields[5].to_string())
        } else if !fields[6].is_empty() && !fields[7].is_empty() {
            (fields[6].to_string(), fields[7].to_string())
        } else {
            // Skip packets with no identifiable source/destination
            continue;
        };

        let protocol = fields[8].to_string();
        // Info field may contain tabs if there are extra fields; join the rest
        let info = fields[9..].join(" ").to_string();

        if protocol.is_empty() {
            continue;
        }

        packets.push(PacketInfo {
            frame_number,
            time_relative,
            src,
            dst,
            protocol,
            info,
        });

        if let Some(max) = max_packets {
            if packets.len() >= max {
                break;
            }
        }
    }

    if packets.is_empty() {
        bail!(
            "No packets were extracted from '{}'. \
             Check that the file is a valid pcap/pcapng and contains network traffic.",
            file_path
        );
    }

    Ok(packets)
}
