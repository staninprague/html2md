use crate::common::get_tag_attr;

use super::StructuredPrinter;
use super::TagHandler;

use markup5ever_rcdom::Handle;

#[derive(Default)]
pub(super) struct ContainerHandler {
    is_toc: bool,
    is_admin_footer: bool,
}

impl TagHandler for ContainerHandler {
    fn handle(&mut self, _tag: &Handle, printer: &mut StructuredPrinter) {
        //class="goog-toc sites-embed-toc-maxdepth-6"
        if let Some(class) = get_tag_attr(_tag, &"class") {
            if class.contains("goog-toc") {
                self.is_toc = true;
            } else if class.contains("sites-adminfooter") {
                self.is_admin_footer = true;
            }
        }

        printer.insert_newline();
        printer.insert_newline();
    }

    fn after_handle(&mut self, printer: &mut StructuredPrinter) {
        printer.insert_newline();
        printer.insert_newline();
    }

    fn skip_descendants(&self) -> bool {
        return self.is_toc || self.is_admin_footer;
    }
}
