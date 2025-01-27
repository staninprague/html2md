use crate::InputFilePath;

use super::StructuredPrinter;
use super::TagHandler;
use super::{clean_markdown, walk};

use std::{cmp, collections::HashMap};

use markup5ever_rcdom::{Handle, NodeData};

pub(super) struct TableHandler {
    pub(crate) input_file_path: InputFilePath,
}

static TABLE_PIPE: char = ' ';
static TABLE_PAD: &str = "";
static TABLE_SPACE: &str = "";

impl TagHandler for TableHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        let mut table_markup = String::new();

        let td_matcher = |cell: &Handle| tag_name(cell) == "td";
        let th_matcher = |cell: &Handle| tag_name(cell) == "th";
        let any_matcher = |cell: &Handle| {
            let name = tag_name(cell);
            name == "td" || name == "th"
        };

        // detect cell width, counts
        let column_count: usize;
        let mut column_widths: Vec<usize>;
        let mut rows = find_children(tag, "tr");
        {
            // detect row count
            let most_big_row = rows.iter().max_by(|left, right| {
                collect_children(&left, any_matcher)
                    .len()
                    .cmp(&collect_children(&right, any_matcher).len())
            });
            if most_big_row.is_none() {
                // we don't have rows with content at all
                return;
            }
            // have rows with content, set column count
            column_count = collect_children(&most_big_row.unwrap(), any_matcher).len();
            column_widths = vec![3; column_count];

            // detect max column width
            for row in &rows {
                let cells = collect_children(row, any_matcher);
                for index in 0..column_count {
                    // from regular rows
                    if let Some(cell) = cells.get(index) {
                        let text = to_text(cell, &self.input_file_path);
                        column_widths[index] = cmp::max(column_widths[index], text.chars().count());
                    }
                }
            }
        }

        {
            // add header row
            let header_tr = rows
                .iter()
                .find(|row| collect_children(&row, th_matcher).len() > 0);
            let header_cells = if let Some(header_row) = header_tr {
                collect_children(header_row, th_matcher)
            } else {
                Vec::new()
            };

            table_markup.push(TABLE_PIPE);
            for index in 0..column_count {
                let padded_header_text = pad_cell_text(
                    &header_cells.get(index),
                    column_widths[index],
                    &self.input_file_path,
                );
                table_markup.push_str(&padded_header_text);
                table_markup.push(TABLE_PIPE);
            }
            table_markup.push('\n');

            // add header-body divider row
            table_markup.push(TABLE_PIPE);
            for index in 0..column_count {
                let width = column_widths[index];
                if width < 3 {
                    // no point in aligning, just post as-is
                    table_markup.push_str(&TABLE_PAD.repeat(width));
                    table_markup.push(TABLE_PIPE);
                    continue;
                }

                // try to detect alignment
                let mut alignment = String::new();
                if let Some(header_cell) = header_cells.get(index) {
                    // we have a header, try to extract alignment from it
                    alignment = match header_cell.data {
                        NodeData::Element { ref attrs, .. } => {
                            let attrs = attrs.borrow();
                            let align_attr = attrs
                                .iter()
                                .find(|attr| attr.name.local.to_string() == "align");
                            align_attr
                                .map(|attr| attr.value.to_string())
                                .unwrap_or_default()
                        }
                        _ => String::new(),
                    };
                }

                // push lines according to alignment, fallback to default behaviour
                match alignment.as_ref() {
                    "left" => {
                        table_markup.push(':');
                        table_markup.push_str(&TABLE_PAD.repeat(width - 1));
                    }
                    "center" => {
                        table_markup.push(':');
                        table_markup.push_str(&TABLE_PAD.repeat(width - 2));
                        table_markup.push(':');
                    }
                    "right" => {
                        table_markup.push_str(&TABLE_PAD.repeat(width - 1));
                        table_markup.push(':');
                    }
                    _ => table_markup.push_str(&TABLE_PAD.repeat(width)),
                }
                table_markup.push(TABLE_PIPE);
            }
            table_markup.push('\n');
        }

        // remove headers, leave only non-header rows now
        // process table rows
        rows.retain(|row| {
            let children = row.children.borrow();
            return children.iter().any(|child| tag_name(&child) == "td");
        });
        for row in &rows {
            table_markup.push(TABLE_PIPE);
            let cells = collect_children(row, td_matcher);
            for index in 0..column_count {
                // we need to fill all cells in a column, even if some rows don't have enough
                // let padded_cell_text = pad_cell_text(&cells.get(index), column_widths[index]);
                // table_markup.push_str(&padded_cell_text);
                // continue;
                if let Some(cell) = cells.get(index) {
                    match cell.data {
                        NodeData::Element { ref attrs, .. } => {
                            if let Some(tag_id) = attrs
                                .borrow()
                                .iter()
                                .find(|&a| a.name.local.to_string() == "id")
                            {
                                //println!("Cell id: {:?}", tag_id);
                                if tag_id.value.to_string() != "sites-chrome-sidebar-left" {
                                    // let padded_cell_text =
                                    //     pad_cell_text(&Some(cell), column_widths[index]);
                                    // println!("** Pushing padded cell text: {}", padded_cell_text);
                                    // table_markup.push_str(&padded_cell_text);
                                    // table_markup.push(TABLE_PIPE);
                                }
                            } else {
                                let padded_cell_text = pad_cell_text(
                                    &Some(cell),
                                    column_widths[index],
                                    &self.input_file_path,
                                );
                                table_markup.push_str(&padded_cell_text);
                                table_markup.push(TABLE_PIPE);
                            }
                        }
                        _ => {}
                    }
                }
            }
            table_markup.push('\n');
        }

        printer.insert_newline();
        printer.insert_newline();
        printer.append_str(&table_markup);
    }

    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {}

    fn skip_descendants(&self) -> bool {
        return true;
    }
}

/// Pads cell text from right and left so it looks centered inside the table cell
/// ### Arguments
/// `tag` - optional reference to currently processed handle, text is extracted from here
///
/// `column_width` - precomputed column width to compute padding length from
fn pad_cell_text(
    tag: &Option<&Handle>,
    column_width: usize,
    input_file_path: &InputFilePath,
) -> String {
    let mut result = String::new();
    if let Some(cell) = tag {
        // have header at specified position
        let text = to_text(cell, input_file_path);
        //println!("cell text: {}", text);
        // compute difference between width and text length
        let len_diff = column_width - text.chars().count();
        if len_diff > 0 {
            // should pad
            if len_diff > 1 {
                // should pad from both sides
                let pad_len = len_diff / 2;
                let remainder = len_diff % 2;
                result.push_str(&TABLE_SPACE.repeat(pad_len));
                result.push_str(&text);
                result.push_str(&TABLE_SPACE.repeat(pad_len + remainder));
            } else {
                // it's just one space, add at the end
                result.push_str(&text);
                result.push(' ');
            }
        } else {
            // shouldn't pad, text fills whole cell
            result.push_str(&text);
        }
    } else {
        // no text in this cell, fill cell with spaces
        let pad_len = column_width;
        result.push_str(&TABLE_SPACE.repeat(pad_len));
    }
    //println!("padded cell text: {}", result);
    return result;
}

/// Extracts tag name from passed tag
/// Returns empty string if it's not an html element
fn tag_name(tag: &Handle) -> String {
    return match tag.data {
        NodeData::Element { ref name, .. } => name.local.to_string(),
        _ => String::new(),
    };
}

/// Find descendants of this tag with tag name `name`
/// This includes both direct children and descendants
fn find_children(tag: &Handle, name: &str) -> Vec<Handle> {
    let mut result: Vec<Handle> = vec![];
    let children = tag.children.borrow();
    for child in children.iter() {
        if tag_name(&child) == name {
            result.push(child.clone());
        }

        let mut descendants = find_children(&child, name);
        result.append(&mut descendants);
    }

    return result;
}

/// Collect direct children that satisfy the predicate
/// This doesn't include descendants
fn collect_children<P>(tag: &Handle, predicate: P) -> Vec<Handle>
where
    P: Fn(&Handle) -> bool,
{
    let mut result: Vec<Handle> = vec![];
    let children = tag.children.borrow();
    for child in children.iter() {
        let candidate = child.clone();
        if predicate(&candidate) {
            result.push(candidate);
        }
    }

    return result;
}

/// Convert html tag to text. This collects all tag children in correct order where they're observed
/// and concatenates their text, recursively.
fn to_text(tag: &Handle, input_file_path: &InputFilePath) -> String {
    let mut printer = StructuredPrinter::default();
    walk(tag, &mut printer, &HashMap::default(), input_file_path);
    //println!("((( Walk result: {}", &printer.data);
    let result = clean_markdown(&printer.data);
    return result;
    //return result.replace("\n", "<br/>");
}
