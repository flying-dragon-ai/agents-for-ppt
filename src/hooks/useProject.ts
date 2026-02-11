import { useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface CanvasInfo {
  key: string;
  name: string;
  dimensions: string;
  viewbox: string;
  width: number;
  height: number;
  aspect_ratio: string;
  category: string;
}

export interface ProjectInfo {
  path: string;
  dir_name: string;
  name: string;
  format: string;
  format_name: string;
  date: string;
  date_formatted: string;
  exists: boolean;
  svg_count: number;
  has_spec: boolean;
  has_readme: boolean;
  has_source: boolean;
  spec_file: string | null;
  svg_files: string[];
  canvas_info: CanvasInfo | null;
}

export interface ValidationResult {
  is_valid: boolean;
  errors: string[];
  warnings: string[];
}

export interface CreateProjectParams {
  name: string;
  format: string;
  baseDir?: string;
}

interface InitProjectResponse {
  projectPath: string;
}

export function useProject() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const createProject = useCallback(async (params: CreateProjectParams): Promise<string> => {
    setLoading(true);
    setError(null);

    try {
      const response = await invoke<InitProjectResponse>('cmd_init_project', {
        request: {
          name: params.name,
          format: params.format,
          baseDir: params.baseDir ?? null,
        },
      });

      return response.projectPath;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getProjectInfo = useCallback(async (projectPath: string): Promise<ProjectInfo> => {
    return invoke<ProjectInfo>('cmd_get_project_info', { projectPath });
  }, []);

  const listProjects = useCallback(async (): Promise<ProjectInfo[]> => {
    setLoading(true);
    setError(null);

    try {
      const paths = await invoke<string[]>('cmd_list_projects');
      const details = await Promise.all(paths.map((path) => getProjectInfo(path)));

      return details.sort((left, right) => right.date.localeCompare(left.date));
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [getProjectInfo]);

  const deleteProject = useCallback(async (projectPath: string): Promise<void> => {
    setLoading(true);
    setError(null);

    try {
      await invoke('cmd_delete_project', { projectPath });
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(errorMsg);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const validateProject = useCallback(async (projectPath: string): Promise<ValidationResult> => {
    return invoke<ValidationResult>('cmd_validate_project', { projectPath });
  }, []);

  return {
    loading,
    error,
    createProject,
    listProjects,
    getProjectInfo,
    deleteProject,
    validateProject,
  };
}
