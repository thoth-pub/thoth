#[macro_export]
macro_rules! pagination_helpers {
    ($component:ident, $pagination_text:expr) => {
        impl $component {
            fn display_count(&self) -> String {
                let offset_display = match self.offset == 0 && self.result_count > 0 {
                    true => 1,
                    false => self.offset,
                };
                let limit_display = match self.limit > self.result_count {
                    true => self.result_count,
                    false => self.limit,
                };
                format!("{} {}-{} of {}", $pagination_text, offset_display, limit_display, self.result_count)
            }

            fn is_previous_disabled(&self) -> bool {
                self.offset < self.page_size
            }

            fn is_next_disabled(&self) -> bool {
                self.limit >= self.result_count
            }
        }
    }
}
pub mod admin;
pub mod catalogue;
pub mod contributions_form;
pub mod contributor;
pub mod contributors;
pub mod dashboard;
pub mod funder;
pub mod funders;
pub mod fundings_form;
pub mod imprint;
pub mod imprints;
pub mod issues_form;
pub mod languages_form;
pub mod login;
pub mod menu;
pub mod navbar;
pub mod new_contributor;
pub mod new_funder;
pub mod new_imprint;
pub mod new_publisher;
pub mod new_series;
pub mod new_work;
pub mod notification;
pub mod publications;
pub mod publications_form;
pub mod publisher;
pub mod publishers;
pub mod root;
pub mod series;
pub mod serieses;
pub mod subjects_form;
pub mod utils;
pub mod work;
pub mod works;
