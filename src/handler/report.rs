//! Handles "Get All Packages" messages.

use std::sync::Mutex;

use event::inbound::{InboundEvent, GetInstalledSoftware};
use handler::{Result, RemoteServices, HandleMessageParams};
use persistence::Transfers;

#[derive(RustcDecodable)]
/// Type for "Get All Packages" messages.
pub struct ReportParams;

impl HandleMessageParams for ReportParams {
    fn handle(&self,
              _: &Mutex<RemoteServices>,
              _: &Mutex<Transfers>) -> Result {
        Ok(Some(InboundEvent::GetInstalledSoftware(GetInstalledSoftware {
            include_packages: true,
            include_module_firmware: false
        })))
    }
}
