//! QR code of a payment link as inline SVG, drawn on the server so the
//! payment page needs no script and no image request. Dark modules on a
//! white square with the standard four-module quiet zone, whatever the
//! page theme: wallet scanners expect dark on light.

use qrcodegen::{QrCode, QrCodeEcc};

/// Quiet zone around the symbol, in modules (ISO/IEC 18004).
const BORDER: i32 = 4;

/// An `<svg>` element encoding `text`, `None` when the text does not fit in
/// a QR code (far beyond any payment link). Only path data and numbers are
/// written, so the result is safe to insert into a page unescaped.
pub fn svg(text: &str) -> Option<String> {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium).ok()?;
    let size = qr.size();
    let full = size + BORDER * 2;
    let mut path = String::with_capacity((size * size) as usize * 8);
    for y in 0..size {
        for x in 0..size {
            if qr.get_module(x, y) {
                path.push_str(&format!("M{},{}h1v1h-1z", x + BORDER, y + BORDER));
            }
        }
    }
    Some(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {full} {full}\" \
         shape-rendering=\"crispEdges\" role=\"img\">\
         <rect width=\"{full}\" height=\"{full}\" fill=\"#ffffff\"/>\
         <path d=\"{path}\" fill=\"#000000\"/></svg>"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_payment_link_becomes_an_svg_of_plain_path_data() {
        let link = "ton://transfer/UQCD39VS5jcptHL8vMjEXrzGaRcCVYto7HUn4bpAOg8xqEBI?amount=12350000000&text=B-7KQ3M9XD2F&exp=1794300000";
        let svg = svg(link).unwrap();
        assert!(svg.starts_with("<svg "));
        assert!(svg.ends_with("</svg>"));
        // Nothing of the link itself ends up in the markup.
        assert!(!svg.contains("ton://"));
        assert!(!svg.contains("B-7KQ3"));
        let d = svg.split("d=\"").nth(1).unwrap().split('"').next().unwrap();
        assert!(d.chars().all(|c| "Mhvz-,0123456789".contains(c)));
        // Same input, same symbol.
        assert_eq!(svg, super::svg(link).unwrap());
    }

    #[test]
    fn the_symbol_has_its_quiet_zone() {
        let svg = svg("ton://transfer/x").unwrap();
        let view: Vec<i32> = svg
            .split("viewBox=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .split(' ')
            .map(|n| n.parse().unwrap())
            .collect();
        // Version 1 is 21 modules; plus 4 on each side.
        assert!(view[2] >= 21 + 8);
        assert!(!svg.contains("M0,") && !svg.contains("M3,"));
    }

    #[test]
    fn text_too_long_for_a_qr_code_is_refused() {
        assert!(svg(&"x".repeat(5000)).is_none());
    }
}
