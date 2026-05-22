use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified: i64,
    pub is_hidden: bool,
    pub is_folder: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DirectoryPage {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub total_count: usize,
    pub sort_key: SortKey,
    pub sort_ascending: bool,
    pub filter_kind: FilterKind,
    pub show_hidden: bool,
    pub snapshot_version: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileTask {
    pub id: String,
    pub source: String,
    pub target: Option<String>,
    pub status: TaskStatus,
    pub progress_current: u64,
    pub progress_total: u64,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    Validating,
    Running,
    WaitingForConflictDecision,
    Cancelling,
    Cancelled,
    Completed,
    Failed,
    PartiallyCompleted,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub schema_version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Icons,
    List,
    Details,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    Name,
    Modified,
    Size,
    #[serde(rename = "type")]
    FileType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterKind {
    All,
    Folders,
    Files,
    Images,
    Documents,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictDecision {
    Replace,
    Skip,
    KeepBoth,
    ApplyToAll,
}
