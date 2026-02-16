use printpdf::Rgb;

/// Return an RGB color for a given protocol name.
/// Colors are chosen to be distinct and readable on white background.
pub fn protocol_color(protocol: &str) -> Rgb {
    let proto = protocol.to_uppercase();
    match proto.as_str() {
        // Transport
        "TCP" => Rgb::new(0.40, 0.40, 0.40, None),
        "UDP" => Rgb::new(0.50, 0.50, 0.50, None),
        "SCTP" => Rgb::new(0.45, 0.35, 0.55, None),

        // Web / HTTP
        "HTTP" => Rgb::new(0.13, 0.55, 0.13, None),
        "HTTP/JSON" => Rgb::new(0.13, 0.55, 0.13, None),
        "HTTP/XML" => Rgb::new(0.13, 0.55, 0.13, None),
        "HTTP2" => Rgb::new(0.00, 0.50, 0.25, None),

        // TLS / Security
        "TLS" | "TLSV1" | "TLSV1.1" | "TLSV1.2" | "TLSV1.3" => {
            Rgb::new(0.00, 0.39, 0.00, None)
        }
        "SSL" => Rgb::new(0.00, 0.39, 0.00, None),
        "QUIC" => Rgb::new(0.00, 0.45, 0.35, None),

        // DNS
        "DNS" => Rgb::new(1.00, 0.55, 0.00, None),
        "MDNS" => Rgb::new(0.85, 0.45, 0.00, None),
        "LLMNR" => Rgb::new(0.85, 0.50, 0.10, None),

        // VoIP / Telephony
        "SIP" => Rgb::new(0.25, 0.41, 0.88, None),
        "SIP/SDP" => Rgb::new(0.25, 0.41, 0.88, None),
        "SDP" => Rgb::new(0.30, 0.50, 0.90, None),
        "RTP" => Rgb::new(0.58, 0.44, 0.86, None),
        "RTCP" => Rgb::new(0.48, 0.34, 0.76, None),
        "MEGACO" => Rgb::new(0.60, 0.20, 0.80, None),
        "H225" | "H245" | "H323" => Rgb::new(0.33, 0.33, 0.88, None),
        "MGCP" => Rgb::new(0.55, 0.25, 0.78, None),

        // Diameter / RADIUS / SS7
        "DIAMETER" => Rgb::new(0.80, 0.20, 0.60, None),
        "RADIUS" => Rgb::new(0.70, 0.30, 0.50, None),
        "SCCP" => Rgb::new(0.60, 0.10, 0.40, None),
        "ISUP" | "BICC" => Rgb::new(0.55, 0.15, 0.45, None),
        "MAP" => Rgb::new(0.50, 0.20, 0.50, None),
        "TCAP" => Rgb::new(0.45, 0.15, 0.55, None),
        "GTP" | "GTPV2" => Rgb::new(0.70, 0.35, 0.15, None),
        "PFCP" => Rgb::new(0.65, 0.30, 0.10, None),

        // ICMP
        "ICMP" => Rgb::new(0.86, 0.08, 0.24, None),
        "ICMPV6" => Rgb::new(0.78, 0.08, 0.30, None),

        // ARP
        "ARP" => Rgb::new(0.55, 0.27, 0.07, None),

        // DHCP
        "DHCP" | "DHCPV6" | "BOOTP" => Rgb::new(0.60, 0.40, 0.00, None),

        // SNMP
        "SNMP" => Rgb::new(0.40, 0.60, 0.20, None),

        // SSH / Telnet
        "SSH" => Rgb::new(0.20, 0.20, 0.60, None),
        "TELNET" => Rgb::new(0.30, 0.30, 0.50, None),

        // FTP / SMTP / POP / IMAP
        "FTP" | "FTP-DATA" => Rgb::new(0.50, 0.35, 0.00, None),
        "SMTP" => Rgb::new(0.10, 0.50, 0.50, None),
        "POP" => Rgb::new(0.20, 0.50, 0.40, None),
        "IMAP" => Rgb::new(0.15, 0.55, 0.45, None),

        // LDAP / Kerberos
        "LDAP" => Rgb::new(0.45, 0.25, 0.00, None),
        "KRB5" | "KERBEROS" => Rgb::new(0.55, 0.20, 0.00, None),

        // SMB / NFS
        "SMB" | "SMB2" => Rgb::new(0.35, 0.45, 0.15, None),
        "NFS" => Rgb::new(0.30, 0.50, 0.10, None),

        // MQTT / AMQP / CoAP (IoT)
        "MQTT" => Rgb::new(0.50, 0.00, 0.50, None),
        "AMQP" => Rgb::new(0.45, 0.05, 0.55, None),
        "COAP" => Rgb::new(0.55, 0.10, 0.45, None),

        // 5G / LTE
        "NGAP" => Rgb::new(0.00, 0.35, 0.70, None),
        "NAS-5GS" | "NAS-5G" => Rgb::new(0.00, 0.40, 0.75, None),
        "S1AP" => Rgb::new(0.10, 0.35, 0.65, None),
        "X2AP" => Rgb::new(0.15, 0.30, 0.60, None),

        // Default: black
        _ => Rgb::new(0.0, 0.0, 0.0, None),
    }
}
