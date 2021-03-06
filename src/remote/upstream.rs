use datatype::UpdateId;
use datatype::report::{InstalledSoftware, UpdateReport};

pub trait Upstream {
    fn send_installed_software(&mut self, m: InstalledSoftware) -> Result<String, String>;
    fn send_start_download(&mut self, id: UpdateId) -> Result<String, String>;
    fn send_update_report(&mut self, m: UpdateReport) -> Result<String, String>;
}
