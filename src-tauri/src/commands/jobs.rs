use crate::events::{emit_job_event, JobEventPayload};
use crate::state::{AppState, JobInfo, JobStatus};
use pptm_pipeline::{PipelineError, PipelineRequest, ProgressSink};
use serde::{Deserialize, Serialize};
use tauri::{State, Window};
use uuid::Uuid;

/// 执行管线请求。
#[derive(Debug, Clone, Deserialize)]
pub struct RunPipelineRequest {
    pub project_path: String,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub options: serde_json::Value,
}

/// 执行管线响应。
#[derive(Debug, Clone, Serialize)]
pub struct RunPipelineResponse {
    pub job_id: String,
}

/// 查询任务状态响应。
#[derive(Debug, Clone, Serialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub message: Option<String>,
}

struct TauriProgressSink {
    window: Window,
    job_id: String,
}

impl ProgressSink for TauriProgressSink {
    fn report_progress(&self, current: usize, total: usize, message: String) {
        let payload = JobEventPayload::progress(self.job_id.clone(), current, total, message);
        let _ = emit_job_event(&self.window, &payload);
    }

    fn log(&self, level: &str, message: String) {
        let payload = JobEventPayload::log(self.job_id.clone(), level.to_string(), message);
        let _ = emit_job_event(&self.window, &payload);
    }
}

#[tauri::command]
pub async fn cmd_run_pipeline(
    req: RunPipelineRequest,
    state: State<'_, AppState>,
    window: Window,
) -> Result<RunPipelineResponse, String> {
    let project_path = std::path::PathBuf::from(&req.project_path);
    if !project_path.exists() {
        return Err(format!("项目路径不存在: {}", req.project_path));
    }

    let job_id = Uuid::new_v4().to_string();
    let cancel_token = state.create_job(job_id.clone()).await;

    let app_state = state.inner().clone();
    let job_id_clone = job_id.clone();
    let window_clone = window.clone();

    tauri::async_runtime::spawn(async move {
        app_state
            .update_job_status(
                &job_id_clone,
                JobStatus::Running,
                Some("任务执行中".to_string()),
            )
            .await;

        let _ = emit_job_event(
            &window_clone,
            &JobEventPayload::started(job_id_clone.clone(), "任务已开始"),
        );

        let request = PipelineRequest {
            project_path,
            steps: req.steps,
            options: req.options,
        };

        let sink = TauriProgressSink {
            window: window_clone.clone(),
            job_id: job_id_clone.clone(),
        };

        let result = app_state
            .orchestrator
            .run_pipeline(request, &sink, cancel_token.clone())
            .await;

        match result {
            Ok(output) => {
                if cancel_token.is_cancelled() {
                    app_state
                        .update_job_status(
                            &job_id_clone,
                            JobStatus::Cancelled,
                            Some("任务已取消".to_string()),
                        )
                        .await;

                    let _ = emit_job_event(
                        &window_clone,
                        &JobEventPayload::cancelled(job_id_clone, "任务已取消"),
                    );
                } else {
                    app_state
                        .update_job_status(
                            &job_id_clone,
                            JobStatus::Completed,
                            Some("任务完成".to_string()),
                        )
                        .await;

                    let payload = JobEventPayload::completed(
                        job_id_clone,
                        "任务完成",
                        serde_json::json!({
                            "processed_steps": output.processed_steps,
                            "output_path": output.output_path,
                        }),
                    );
                    let _ = emit_job_event(&window_clone, &payload);
                }
            }
            Err(error) => {
                let (status, message) = match error {
                    PipelineError::Cancelled => (JobStatus::Cancelled, "任务已取消".to_string()),
                    PipelineError::ProjectNotFound(path) => (
                        JobStatus::Failed,
                        format!("项目目录不存在: {}", path.display()),
                    ),
                };

                app_state
                    .update_job_status(&job_id_clone, status.clone(), Some(message.clone()))
                    .await;

                let payload = match status {
                    JobStatus::Cancelled => JobEventPayload::cancelled(job_id_clone, message),
                    _ => JobEventPayload::failed(job_id_clone, message),
                };
                let _ = emit_job_event(&window_clone, &payload);
            }
        }
    });

    Ok(RunPipelineResponse { job_id })
}

#[tauri::command]
pub async fn cmd_get_job_status(
    job_id: String,
    state: State<'_, AppState>,
) -> Result<JobStatusResponse, String> {
    let info: JobInfo = state
        .get_job_info(&job_id)
        .await
        .ok_or_else(|| format!("任务不存在: {job_id}"))?;

    Ok(JobStatusResponse {
        job_id: info.job_id,
        status: info.status,
        message: info.message,
    })
}

#[tauri::command]
pub async fn cmd_cancel_job(
    job_id: String,
    state: State<'_, AppState>,
    window: Window,
) -> Result<(), String> {
    state.cancel_job(&job_id).await?;

    emit_job_event(
        &window,
        &JobEventPayload::cancelled(job_id, "已请求取消任务"),
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}
