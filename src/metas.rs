use super::StructuredPrinter;
use super::TagHandler;

use markup5ever_rcdom::{Handle, NodeData};

/// Handler that completely copies tag to printer as HTML with all descendants
#[derive(Default)]
pub(super) struct MetaHandler {
    tag_name: String,
}

impl TagHandler for MetaHandler {
    fn handle(&mut self, tag: &Handle, printer: &mut StructuredPrinter) {
        //println!("In the meta handler: {:?}", tag);
        match tag.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let attrs = attrs.borrow();
                self.tag_name = name.local.to_string();
                let mut has_meta_title = false;

                for attr in attrs.iter() {
                    if attr.name.local.to_string() == "name" && attr.value.to_string() == "title" {
                        if let Some(content) = attrs
                            .iter()
                            .find(|&a| a.name.local.to_string() == "content")
                        {
                            //println!("In the meta title: {:?}", &content.value);
                            has_meta_title = true;
                            printer.insert_newline();
                            printer.append_str("---");
                            printer.insert_newline();
                            let sanitized_title =
                                content.value.replace(" - speedohelp", "").replace("\"", "");
                            printer.append_str(&format!("title: \"{}\"", &sanitized_title));
                            printer.insert_newline();
                            break;
                        }
                    }
                }
                if has_meta_title {
                    printer.append_str("---");
                    printer.insert_newline();
                    printer.insert_newline();
                }
            }
            _ => return,
        }
    }

    fn skip_descendants(&self) -> bool {
        return true;
    }

    fn after_handle(&mut self, _printer: &mut StructuredPrinter) {}
}
