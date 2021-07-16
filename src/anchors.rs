use crate::InputFilePath;

use super::StructuredPrinter;
use super::TagHandler;

use markup5ever_rcdom::{Handle, NodeData};

#[derive(Default)]
pub(super) struct AnchorHandler {
    start_pos: usize,
    url: String,
    input_file_path: InputFilePath,
}

impl TagHandler for AnchorHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        self.start_pos = printer.data.len();

        // try to extract a hyperlink
        self.url = match tag.data {
            NodeData::Element { ref attrs, .. } => {
                let attrs = attrs.borrow();
                let href = attrs
                    .iter()
                    .find(|attr| attr.name.local.to_string() == "href");
                match href {
                    Some(link) => self.input_file_path.adjusted_url(&link.value.to_string()),
                    None => String::new(),
                }
            }
            _ => String::new(),
        };
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
        // add braces around already present text, put an url afterwards
        printer.insert_str(self.start_pos, "[");
        printer.append_str(&format!("]({})", self.url))
    }
}
