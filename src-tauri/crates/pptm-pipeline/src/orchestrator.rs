use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

/// 管线执行请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRequest {
    pub project_path: PathBuf,
    pub steps: Vec<String>,
    pub options: serde_json::Value,
}

/// 管线执行结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineResult {
    pub processed_steps: Vec<String>,
    pub output_path: PathBuf,
}

/// 管线错误类型。
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("项目目录不存在: {0}")]
    ProjectNotFound(PathBuf),
    #[error("任务已取消")]
    Cancelled,
}

/// 进度上报接口（供 Tauri 层实现事件转发）。
pub trait ProgressSink: Send + Sync {
    fn report_progress(&self, current: usize, total: usize, message: String);
    fn log(&self, level: &str, message: String);
}

/// 管线调度入口（当前为占位实现，后续逐步接入真实步骤）。
#[derive(Debug, Clone, Default)]
pub struct PipelineOrchestrator;

impl PipelineOrchestrator {
    pub fn new() -> Self {
        Self
    }

    /// 运行通用处理管线。
    pub async fn run_pipeline<S: ProgressSink>(
        &self,
        request: PipelineRequest,
        sink: &S,
        cancel_token: CancellationToken,
    ) -> Result<PipelineResult, PipelineError> {
        if !request.project_path.exists() {
            return Err(PipelineError::ProjectNotFound(request.project_path));
        }

        let steps = normalize_steps(&request.steps);
        let total = steps.len();

        sink.log("info", format!("开始执行管线，共 {} 个步骤", total));

        for (index, step) in steps.iter().enumerate() {
            if cancel_token.is_cancelled() {
                sink.log("warn", "检测到取消信号，停止执行".to_string());
                return Err(PipelineError::Cancelled);
            }

            sink.report_progress(index + 1, total, format!("执行步骤: {}", step));

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let output_path = request.project_path.join("svg_final");
        sink.log("info", "管线执行完成".to_string());

        Ok(PipelineResult {
            processed_steps: steps,
            output_path,
        })
    }
}

fn normalize_steps(steps: &[String]) -> Vec<String> {
    if steps.is_empty() {
        return vec![
            "total_md_split".to_string(),
            "finalize_svg".to_string(),
            "svg_to_pptx".to_string(),
        ];
    }

    steps.iter().map(|s| s.trim().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Debug, Default)]
    struct MemorySink {
        progress: Mutex<Vec<String>>,
        logs: Mutex<Vec<String>>,
    }

    impl ProgressSink for MemorySink {
        fn report_progress(&self, current: usize, total: usize, message: String) {
            self.progress
                .lock()
                .expect("progress 锁应可用")
                .push(format!("{current}/{total}:{message}"));
        }

        fn log(&self, level: &str, message: String) {
            self.logs
                .lock()
                .expect("logs 锁应可用")
                .push(format!("{level}:{message}"));
        }
    }

    #[tokio::test]
    async fn test_run_pipeline_success() {
        let temp_dir = tempfile::tempdir().expect("应能创建临时目录");

        let request = PipelineRequest {
            project_path: temp_dir.path().to_path_buf(),
            steps: vec!["step_a".to_string(), "step_b".to_string()],
            options: serde_json::json!({}),
        };

        let sink = MemorySink::default();
        let cancel_token = CancellationToken::new();
        let orchestrator = PipelineOrchestrator::new();

        let result = orchestrator
            .run_pipeline(request, &sink, cancel_token)
            .await
            .expect("管线应执行成功");

        assert_eq!(result.processed_steps, vec!["step_a", "step_b"]);
        assert_eq!(
            result.output_path.file_name().and_then(|n| n.to_str()),
            Some("svg_final")
        );

        let progress_events = sink.progress.lock().expect("应能读取进度事件");
        assert_eq!(progress_events.len(), 2);
    }

    #[tokio::test]
    async fn test_run_pipeline_cancelled() {
        let temp_dir = tempfile::tempdir().expect("应能创建临时目录");

        let request = PipelineRequest {
            project_path: temp_dir.path().to_path_buf(),
            steps: vec!["step_a".to_string(), "step_b".to_string()],
            options: serde_json::json!({}),
        };

        let sink = MemorySink::default();
        let cancel_token = CancellationToken::new();
        cancel_token.cancel();

        let orchestrator = PipelineOrchestrator::new();
        let result = orchestrator
            .run_pipeline(request, &sink, cancel_token)
            .await;

        assert!(matches!(result, Err(PipelineError::Cancelled)));
    }

    #[tokio::test]
    async fn test_run_pipeline_project_not_found() {
        let request = PipelineRequest {
            project_path: PathBuf::from("D:/this/path/should/not/exist"),
            steps: vec![],
            options: serde_json::json!({}),
        };

        let sink = MemorySink::default();
        let cancel_token = CancellationToken::new();

        let orchestrator = PipelineOrchestrator::new();
        let result = orchestrator
            .run_pipeline(request, &sink, cancel_token)
            .await;

        assert!(matches!(result, Err(PipelineError::ProjectNotFound(_))));
    }
}
