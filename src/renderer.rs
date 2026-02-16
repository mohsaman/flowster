use crate::colors::protocol_color;
use crate::model::SignalingDiagram;
use anyhow::{Context, Result};
use printpdf::path::{PaintMode, WindingOrder};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

// Layout constants (all in mm)
const PAGE_WIDTH: f32 = 420.0; // A3 landscape width
const PAGE_HEIGHT: f32 = 297.0; // A3 landscape height
const TOP_MARGIN: f32 = 15.0;
const BOTTOM_MARGIN: f32 = 15.0;
const LEFT_MARGIN: f32 = 35.0; // Space for frame# / timestamp column
const RIGHT_MARGIN: f32 = 15.0;
const HEADER_HEIGHT: f32 = 22.0; // Height of the node header boxes
const ROW_HEIGHT: f32 = 7.0; // Vertical space per message row
const NODE_BOX_WIDTH: f32 = 30.0; // Width of node header box
const NODE_BOX_HEIGHT: f32 = 14.0;
const ARROWHEAD_LEN: f32 = 2.5; // Arrowhead length in mm
const ARROWHEAD_WIDTH: f32 = 1.5; // Arrowhead half-width in mm
const SELF_LOOP_WIDTH: f32 = 12.0; // Width of self-message loop
const LIFELINE_DASH: i64 = 4; // Dash length for lifelines

// Font sizes (in points)
const FONT_SIZE_TITLE: f32 = 14.0;
const FONT_SIZE_NODE: f32 = 7.0;
const FONT_SIZE_MSG: f32 = 6.0;
const FONT_SIZE_TIMESTAMP: f32 = 5.5;
const FONT_SIZE_PAGE: f32 = 6.0;

/// Render a signaling diagram to a PDF file.
pub fn render_pdf(
    diagram: &SignalingDiagram,
    output_path: &str,
    title: Option<&str>,
) -> Result<()> {
    let node_count = diagram.nodes.len();
    if node_count == 0 {
        anyhow::bail!("No nodes to render");
    }

    // Calculate usable area
    let usable_width = PAGE_WIDTH - LEFT_MARGIN - RIGHT_MARGIN;
    let header_y = PAGE_HEIGHT - TOP_MARGIN;
    let first_msg_y = header_y - HEADER_HEIGHT;
    let usable_msg_height = first_msg_y - BOTTOM_MARGIN;
    let msgs_per_page = (usable_msg_height / ROW_HEIGHT).floor() as usize;

    // Calculate node X positions (evenly spaced)
    let node_spacing = if node_count > 1 {
        usable_width / (node_count as f32 - 1.0).max(1.0)
    } else {
        0.0
    };
    let node_x: Vec<f32> = (0..node_count)
        .map(|i| {
            if node_count == 1 {
                LEFT_MARGIN + usable_width / 2.0
            } else {
                LEFT_MARGIN + i as f32 * node_spacing
            }
        })
        .collect();

    // Paginate messages
    let total_messages = diagram.messages.len();
    let total_pages = if total_messages == 0 {
        1
    } else {
        (total_messages + msgs_per_page - 1) / msgs_per_page
    };

    // Create PDF document
    let doc_title = title.unwrap_or("Signaling Flow Diagram");
    let (doc, page1, layer1) =
        PdfDocument::new(doc_title, Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), "Layer 1");

    let font_regular = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .context("Failed to add Helvetica font")?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .context("Failed to add Helvetica Bold font")?;
    let font_mono = doc
        .add_builtin_font(BuiltinFont::Courier)
        .context("Failed to add Courier font")?;

    for page_num in 0..total_pages {
        let (current_page, current_layer) = if page_num == 0 {
            (page1, layer1)
        } else {
            let (p, l) = doc.add_page(
                Mm(PAGE_WIDTH),
                Mm(PAGE_HEIGHT),
                format!("Page {}", page_num + 1),
            );
            (p, l)
        };

        let layer = doc.get_page(current_page).get_layer(current_layer);

        // --- Draw title (first page only) ---
        if page_num == 0 {
            if let Some(t) = title {
                layer.use_text(t, FONT_SIZE_TITLE, Mm(LEFT_MARGIN), Mm(header_y + 2.0), &font_bold);
            }
        }

        // --- Draw page number ---
        let page_label = format!("Page {} / {}", page_num + 1, total_pages);
        layer.use_text(
            &page_label,
            FONT_SIZE_PAGE,
            Mm(PAGE_WIDTH - RIGHT_MARGIN - 20.0),
            Mm(BOTTOM_MARGIN / 2.0),
            &font_regular,
        );

        // --- Draw node header boxes and labels ---
        draw_node_headers(
            &layer,
            diagram,
            &node_x,
            header_y,
            &font_bold,
        );

        // --- Draw lifelines (dashed vertical lines) ---
        draw_lifelines(
            &layer,
            &node_x,
            header_y - NODE_BOX_HEIGHT,
            BOTTOM_MARGIN,
        );

        // --- Draw column headers for the time/frame column ---
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text("Frame", FONT_SIZE_TIMESTAMP, Mm(2.0), Mm(header_y - 5.0), &font_bold);
        layer.use_text("Time", FONT_SIZE_TIMESTAMP, Mm(14.0), Mm(header_y - 5.0), &font_bold);

        // --- Draw messages for this page ---
        let start_idx = page_num * msgs_per_page;
        let end_idx = (start_idx + msgs_per_page).min(total_messages);

        for (row, msg_idx) in (start_idx..end_idx).enumerate() {
            let msg = &diagram.messages[msg_idx];
            let y = first_msg_y - (row as f32 + 0.5) * ROW_HEIGHT;

            // Timestamp and frame number on the left
            layer.set_fill_color(Color::Rgb(Rgb::new(0.35, 0.35, 0.35, None)));
            let frame_str = format!("{}", msg.frame_number);
            layer.use_text(&frame_str, FONT_SIZE_TIMESTAMP, Mm(2.0), Mm(y - 1.0), &font_mono);

            let time_str = format!("{:.4}", msg.timestamp);
            layer.use_text(&time_str, FONT_SIZE_TIMESTAMP, Mm(14.0), Mm(y - 1.0), &font_mono);

            // Get protocol color
            let color = protocol_color(&msg.protocol);

            if msg.source_idx == msg.dest_idx {
                // Self-message: draw a loop
                draw_self_message(
                    &layer,
                    node_x[msg.source_idx],
                    y,
                    &msg.protocol,
                    &msg.info,
                    &color,
                    &font_regular,
                );
            } else {
                // Normal message arrow
                let src_x = node_x[msg.source_idx];
                let dst_x = node_x[msg.dest_idx];
                draw_message_arrow(
                    &layer,
                    src_x,
                    dst_x,
                    y,
                    &msg.protocol,
                    &msg.info,
                    &color,
                    &font_regular,
                );
            }
        }
    }

    // Save
    let file = File::create(output_path)
        .with_context(|| format!("Cannot create output file '{}'", output_path))?;
    doc.save(&mut BufWriter::new(file))
        .context("Failed to write PDF")?;

    Ok(())
}

/// Draw the node header boxes at the top of the page.
fn draw_node_headers(
    layer: &PdfLayerReference,
    diagram: &SignalingDiagram,
    node_x: &[f32],
    header_y: f32,
    font_bold: &IndirectFontRef,
) {
    let box_top = header_y - 4.0;

    for (i, node) in diagram.nodes.iter().enumerate() {
        let cx = node_x[i];
        let bx = cx - NODE_BOX_WIDTH / 2.0;
        let by = box_top - NODE_BOX_HEIGHT;

        // Box background
        layer.set_fill_color(Color::Rgb(Rgb::new(0.90, 0.93, 0.98, None)));
        layer.set_outline_color(Color::Rgb(Rgb::new(0.30, 0.40, 0.65, None)));
        layer.set_outline_thickness(0.8);

        let rect = Rect::new(Mm(bx), Mm(by), Mm(bx + NODE_BOX_WIDTH), Mm(box_top))
            .with_mode(PaintMode::FillStroke);
        layer.add_rect(rect);

        // Node label - truncate if needed
        let label = truncate_str(&node.address, 20);
        // Center text approximately
        let text_x = cx - estimate_text_width(&label, FONT_SIZE_NODE) / 2.0;
        let text_y = by + NODE_BOX_HEIGHT / 2.0 - 1.0;

        layer.set_fill_color(Color::Rgb(Rgb::new(0.10, 0.10, 0.30, None)));
        layer.use_text(&label, FONT_SIZE_NODE, Mm(text_x), Mm(text_y), font_bold);
    }
}

/// Draw dashed vertical lifelines from under each node header to the bottom.
fn draw_lifelines(
    layer: &PdfLayerReference,
    node_x: &[f32],
    top_y: f32,
    bottom_y: f32,
) {
    layer.set_outline_color(Color::Rgb(Rgb::new(0.70, 0.70, 0.70, None)));
    layer.set_outline_thickness(0.4);
    layer.set_line_dash_pattern(LineDashPattern {
        dash_1: Some(LIFELINE_DASH),
        gap_1: Some(LIFELINE_DASH / 2),
        ..Default::default()
    });

    for &x in node_x {
        let line = Line {
            points: vec![
                (Point::new(Mm(x), Mm(top_y)), false),
                (Point::new(Mm(x), Mm(bottom_y)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    // Reset dash pattern
    layer.set_line_dash_pattern(LineDashPattern::default());
}

/// Draw a message arrow between two nodes.
fn draw_message_arrow(
    layer: &PdfLayerReference,
    src_x: f32,
    dst_x: f32,
    y: f32,
    protocol: &str,
    info: &str,
    color: &Rgb,
    font: &IndirectFontRef,
) {
    let pdf_color = Color::Rgb(color.clone());

    // Draw the line
    layer.set_outline_color(pdf_color.clone());
    layer.set_outline_thickness(0.6);
    layer.add_line(Line {
        points: vec![
            (Point::new(Mm(src_x), Mm(y)), false),
            (Point::new(Mm(dst_x), Mm(y)), false),
        ],
        is_closed: false,
    });

    // Draw arrowhead at destination
    layer.set_fill_color(pdf_color.clone());
    let direction: f32 = if dst_x > src_x { -1.0 } else { 1.0 };
    let arrow_tip_x = dst_x;
    let arrow_base_x = dst_x + direction * ARROWHEAD_LEN;

    let arrowhead = Polygon {
        rings: vec![vec![
            (Point::new(Mm(arrow_tip_x), Mm(y)), false),
            (
                Point::new(Mm(arrow_base_x), Mm(y + ARROWHEAD_WIDTH)),
                false,
            ),
            (
                Point::new(Mm(arrow_base_x), Mm(y - ARROWHEAD_WIDTH)),
                false,
            ),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(arrowhead);

    // Label: "PROTOCOL: info" above the arrow
    let label = build_msg_label(protocol, info, src_x, dst_x);
    let mid_x = (src_x + dst_x) / 2.0;
    let text_w = estimate_text_width(&label, FONT_SIZE_MSG);
    let text_x = mid_x - text_w / 2.0;

    layer.set_fill_color(pdf_color);
    layer.use_text(&label, FONT_SIZE_MSG, Mm(text_x), Mm(y + 1.2), font);
}

/// Draw a self-message (loop back to same node).
fn draw_self_message(
    layer: &PdfLayerReference,
    x: f32,
    y: f32,
    protocol: &str,
    info: &str,
    color: &Rgb,
    font: &IndirectFontRef,
) {
    let pdf_color = Color::Rgb(color.clone());
    layer.set_outline_color(pdf_color.clone());
    layer.set_outline_thickness(0.6);

    let right = x + SELF_LOOP_WIDTH;
    let top = y + 2.0;
    let bot = y - 2.0;

    // Draw three sides of a rectangle (right-opening loop)
    let loop_line = Line {
        points: vec![
            (Point::new(Mm(x), Mm(top)), false),
            (Point::new(Mm(right), Mm(top)), false),
            (Point::new(Mm(right), Mm(bot)), false),
            (Point::new(Mm(x), Mm(bot)), false),
        ],
        is_closed: false,
    };
    layer.add_line(loop_line);

    // Arrowhead pointing left at the return
    layer.set_fill_color(pdf_color.clone());
    let arrowhead = Polygon {
        rings: vec![vec![
            (Point::new(Mm(x), Mm(bot)), false),
            (
                Point::new(Mm(x + ARROWHEAD_LEN), Mm(bot + ARROWHEAD_WIDTH)),
                false,
            ),
            (
                Point::new(Mm(x + ARROWHEAD_LEN), Mm(bot - ARROWHEAD_WIDTH)),
                false,
            ),
        ]],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(arrowhead);

    // Label to the right
    let label = build_msg_label(protocol, info, x, x + SELF_LOOP_WIDTH);
    layer.set_fill_color(pdf_color);
    layer.use_text(&label, FONT_SIZE_MSG, Mm(x + SELF_LOOP_WIDTH + 1.0), Mm(y), font);
}

/// Build a message label from protocol and info, truncated to fit.
fn build_msg_label(protocol: &str, info: &str, src_x: f32, dst_x: f32) -> String {
    let available_mm = (dst_x - src_x).abs();
    // Rough estimate: each character is ~1.2mm at FONT_SIZE_MSG
    let max_chars = ((available_mm / 1.2) as usize).max(15);

    let full = format!("{}: {}", protocol, info);
    truncate_str(&full, max_chars)
}

/// Truncate a string to max_len characters, appending "..." if truncated.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s[..max_len].to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Rough estimate of text width in mm for Helvetica at a given font size.
/// Helvetica average character width ≈ 0.52 × font_size_pt, converted to mm.
fn estimate_text_width(text: &str, font_size_pt: f32) -> f32 {
    let avg_char_width_pt = 0.50 * font_size_pt;
    let avg_char_width_mm = avg_char_width_pt * 0.3528; // 1pt ≈ 0.3528mm
    text.len() as f32 * avg_char_width_mm
}
