//! CSV writer with cell escaping against formula injection.
//!
//! A cell that starts with `=`, `+`, `-`, `@`, tab or carriage return gets a leading single
//! quote, so a spreadsheet does not run it as a formula. A cell with a comma, a quote or a line
//! break goes in double quotes. Numbers are written with [`number`], which never needs the
//! quote.

/// Escapes one text cell.
#[must_use]
pub fn cell(text: &str) -> String {
    let mut t = text.to_owned();
    if t.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        t.insert(0, '\'');
    }
    if t.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", t.replace('"', "\"\""))
    } else {
        t
    }
}

/// A number cell (no escaping: a finite number cannot start a formula, a negative sign is part
/// of the number).
#[must_use]
pub fn number(v: f64) -> String {
    format!("{v}")
}

/// One CSV line from escaped cells, ended with CRLF.
#[must_use]
pub fn line(cells: &[String]) -> String {
    let mut s = cells.join(",");
    s.push_str("\r\n");
    s
}

/// A header or text line: every cell is escaped.
#[must_use]
pub fn text_line(cells: &[&str]) -> String {
    line(&cells.iter().map(|c| cell(c)).collect::<Vec<_>>())
}
