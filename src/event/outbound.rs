pub type UpdateId = String;

#[derive(RustcDecodable, Debug, PartialEq, PartialOrd, Clone)]
pub struct OperationResult {
    id: String,
    result_code: u32,
    result_text: String
}

pub struct OperationResults(pub Vec<OperationResult>);

pub struct UpdateReport {
    update_id: String,
    operation_results: OperationResults
}

impl UpdateReport {
    pub fn new(id: String, res: OperationResults) -> UpdateReport {
        UpdateReport {
            update_id: id,
            operation_results: res
        }
    }
}

pub enum OutBoundEvent {
    InitiateDownload(UpdateId),
    AbortDownload(UpdateId),
    UpdateReport(UpdateReport)
}
