pub mod config;
pub mod errors;
pub mod model;
pub mod project_utils;

pub use config::{normalize_canvas_format, CanvasFormat, CANVAS_FORMATS};
pub use model::{ProjectInfo as ProjectModelInfo, ProjectMetadata};
pub use project_utils::{
    find_all_projects, get_project_info, parse_project_name, validate_project_structure,
    ParsedProjectName, ProjectInfo, ValidationResult,
};
