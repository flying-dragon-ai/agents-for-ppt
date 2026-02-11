import { useState } from 'react';
import { Calendar, FileText, FolderOpen, Trash2 } from 'lucide-react';
import type { ProjectInfo } from '../../hooks/useProject';

interface ProjectCardProps {
  project: ProjectInfo;
  onOpen: (projectPath: string) => void;
  onDelete: (projectPath: string) => void;
}

export function ProjectCard({ project, onOpen, onDelete }: ProjectCardProps) {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleDelete = (event: React.MouseEvent) => {
    event.stopPropagation();

    if (showDeleteConfirm) {
      onDelete(project.path);
      return;
    }

    setShowDeleteConfirm(true);
    setTimeout(() => setShowDeleteConfirm(false), 3000);
  };

  const formatDate = (dateStr: string) => {
    if (!dateStr || dateStr === 'unknown') {
      return '未知日期';
    }

    if (/^\d{8}$/.test(dateStr)) {
      return `${dateStr.slice(0, 4)}-${dateStr.slice(4, 6)}-${dateStr.slice(6, 8)}`;
    }

    return dateStr;
  };

  const displayDate = project.date_formatted || formatDate(project.date);

  return (
    <div
      className="group relative cursor-pointer rounded-lg border border-gray-200 bg-white p-4 transition-all hover:shadow-lg dark:border-gray-700 dark:bg-gray-800"
      onClick={() => onOpen(project.path)}
    >
      <div className="mb-3 flex items-start justify-between">
        <h3 className="flex-1 truncate text-lg font-semibold text-gray-900 dark:text-white">
          {project.name}
        </h3>

        <button
          onClick={handleDelete}
          className={`ml-2 rounded p-1.5 transition-colors ${
            showDeleteConfirm
              ? 'bg-red-500 text-white'
              : 'text-gray-400 hover:bg-red-50 hover:text-red-500 dark:hover:bg-red-900/20'
          }`}
          title={showDeleteConfirm ? '确认删除' : '删除项目'}
        >
          <Trash2 className="h-4 w-4" />
        </button>
      </div>

      <div className="space-y-2 text-sm text-gray-600 dark:text-gray-400">
        <div className="flex items-center gap-2">
          <FileText className="h-4 w-4" />
          <span>{project.format_name}</span>
        </div>

        <div className="flex items-center gap-2">
          <Calendar className="h-4 w-4" />
          <span>{displayDate}</span>
        </div>

        <div className="flex items-center gap-2">
          <FolderOpen className="h-4 w-4" />
          <span>{project.svg_count} 页</span>
        </div>
      </div>

      {project.has_spec && (
        <div className="absolute right-2 top-2">
          <span className="inline-flex items-center rounded-full bg-blue-100 px-2 py-1 text-xs font-medium text-blue-800 dark:bg-blue-900 dark:text-blue-200">
            有规范
          </span>
        </div>
      )}

      <div className="pointer-events-none absolute inset-0 rounded-lg border-2 border-blue-500 opacity-0 transition-opacity group-hover:opacity-100" />
    </div>
  );
}
