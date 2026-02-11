use serde::Serialize;
use tauri::{Emitter, Window};

pub const JOB_EVENT_CHANNEL: &str = "job:event";

/// 任务事件类型。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobEventKind {
    Started,
    Progress,
    Log,
    Completed,
    Failed,
    Cancelled,
}

/// 统一任务事件载荷。
#[derive(Debug, Clone, Serialize)]
pub struct JobEventPayload {
    pub job_id: String,
    pub kind: JobEventKind,
    pub message: Option<String>,
    pub level: Option<String>,
    pub current: Option<usize>,
    pub total: Option<usize>,
    pub result: Option<serde_json::Value>,
}

impl JobEventPayload {
    pub fn started(job_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            kind: JobEventKind::Started,
            message: Some(message.into()),
            level: None,
            current: None,
            total: None,
            result: None,
        }
    }

    pub fn progress(
        job_id: impl Into<String>,
        current: usize,
        total: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            kind: JobEventKind::Progress,
            message: Some(message.into()),
            level: None,
            current: Some(current),
            total: Some(total),
            result: None,
        }
    }

    pub fn log(
        job_id: impl Into<String>,
        level: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            kind: JobEventKind::Log,
            message: Some(message.into()),
            level: Some(level.into()),
            current: None,
            total: None,
            result: None,
        }
    }

    pub fn completed(
        job_id: impl Into<String>,
        message: impl Into<String>,
        result: serde_json::Value,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            kind: JobEventKind::Completed,
            message: Some(message.into()),
            level: None,
            current: None,
            total: None,
            result: Some(result),
        }
    }

    pub fn failed(job_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            kind: JobEventKind::Failed,
            message: Some(message.into()),
            level: None,
            current: None,
            total: None,
            result: None,
        }
    }

    pub fn cancelled(job_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            kind: JobEventKind::Cancelled,
            message: Some(message.into()),
            level: None,
            current: None,
            total: None,
            result: None,
        }
    }
}

pub fn emit_job_event(window: &Window, payload: &JobEventPayload) -> Result<(), tauri::Error> {
    window.emit(JOB_EVENT_CHANNEL, payload)
}
