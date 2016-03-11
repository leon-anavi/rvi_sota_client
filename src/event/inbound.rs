
#[allow(dead_code)]
pub struct UpdateAvailable {
    update_id: String,
    signature: String,
    description: String,
    request_confirmation: bool
}

#[allow(dead_code)]
pub struct GetInstalledSoftware {
    include_packages: bool,
    include_module_firmware: bool
}

#[allow(dead_code)]
pub enum InboundEvent {
    UpdateAvailable(UpdateAvailable),
    GetInstalledSoftware(GetInstalledSoftware)
}
