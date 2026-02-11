use pptm_pipeline::PipelineOrchestrator;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// 任务状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 任务状态快照（用于对外返回）。
#[derive(Debug, Clone, Serialize)]
pub struct JobInfo {
    pub job_id: String,
    pub status: JobStatus,
    pub message: Option<String>,
}

/// 任务记录（内部使用）
///
/// 包含任务的完整状态信息和取消令牌
#[derive(Debug, Clone)]
struct JobRecord {
    status: JobStatus,
    message: Option<String>,
    cancel_token: CancellationToken,
}

/// Tauri 全局应用状态。
#[derive(Clone)]
pub struct AppState {
    /// 工作区根目录（预留字段，未来用于多工作区支持）
    #[allow(dead_code)]
    pub workspace_root: PathBuf,
    pub orchestrator: PipelineOrchestrator,
    job_registry: Arc<RwLock<HashMap<String, JobRecord>>>,
}

impl AppState {
    pub fn new(workspace_root: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&workspace_root);

        Self {
            workspace_root,
            orchestrator: PipelineOrchestrator::new(),
            job_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新任务
    ///
    /// # 参数
    /// - `job_id`: 任务唯一标识符
    ///
    /// # 返回
    /// 返回任务的取消令牌，用于取消任务
    pub async fn create_job(&self, job_id: String) -> CancellationToken {
        let cancel_token = CancellationToken::new();

        let record = JobRecord {
            status: JobStatus::Pending,
            message: Some("任务已创建，等待执行".to_string()),
            cancel_token: cancel_token.clone(),
        };

        self.job_registry.write().await.insert(job_id, record);
        cancel_token
    }

    /// 更新任务状态
    ///
    /// # 参数
    /// - `job_id`: 任务 ID
    /// - `status`: 新状态
    /// - `message`: 状态消息（可选）
    pub async fn update_job_status(
        &self,
        job_id: &str,
        status: JobStatus,
        message: Option<String>,
    ) {
        if let Some(job) = self.job_registry.write().await.get_mut(job_id) {
            job.status = status;
            job.message = message;
        }
    }

    /// 获取任务信息
    ///
    /// # 参数
    /// - `job_id`: 任务 ID
    ///
    /// # 返回
    /// 如果任务存在，返回任务信息快照；否则返回 None
    pub async fn get_job_info(&self, job_id: &str) -> Option<JobInfo> {
        self.job_registry
            .read()
            .await
            .get(job_id)
            .map(|job| JobInfo {
                job_id: job_id.to_string(),
                status: job.status.clone(),
                message: job.message.clone(),
            })
    }

    /// 取消任务
    ///
    /// # 参数
    /// - `job_id`: 任务 ID
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误信息
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), String> {
        let mut registry = self.job_registry.write().await;
        let job = registry
            .get_mut(job_id)
            .ok_or_else(|| format!("任务不存在: {job_id}"))?;

        job.cancel_token.cancel();
        job.status = JobStatus::Cancelled;
        job.message = Some("已发送取消信号".to_string());

        Ok(())
    }
}
