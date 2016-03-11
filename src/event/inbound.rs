
pub struct UpdateAvailable {
    pub update_id: String,
    pub signature: String,
    pub description: String,
    pub request_confirmation: bool
}

pub struct DownloadComplete {
    pub update_image: String,
    pub signature: String
}

pub struct GetInstalledSoftware {
    pub include_packages: bool,
    pub include_module_firmware: bool
}

pub enum InboundEvent {
    UpdateAvailable(UpdateAvailable),
    DownloadComplete(DownloadComplete),
    GetInstalledSoftware(GetInstalledSoftware)
}
