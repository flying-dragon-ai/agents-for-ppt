import { useCallback, useEffect, useState } from 'react';
import { Plus, RefreshCw, Search } from 'lucide-react';
import { useProject, type ProjectInfo } from '../../hooks/useProject';
import { ProjectCard } from './ProjectCard';

interface ProjectListProps {
  refreshToken: number;
  onOpenProject: (projectPath: string) => void;
  onCreateNew: () => void;
}

export function ProjectList({ refreshToken, onOpenProject, onCreateNew }: ProjectListProps) {
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterFormat, setFilterFormat] = useState<string>('all');

  const { deleteProject, listProjects, loading } = useProject();

  const loadProjects = useCallback(async () => {
    try {
      const projectList = await listProjects();
      setProjects(projectList);
    } catch (error) {
      console.error('加载项目列表失败:', error);
    }
  }, [listProjects]);

  useEffect(() => {
    void loadProjects();
  }, [loadProjects, refreshToken]);

  const handleDelete = async (projectPath: string) => {
    try {
      await deleteProject(projectPath);
      await loadProjects();
    } catch (error) {
      console.error('删除项目失败:', error);
    }
  };

  const filteredProjects = projects.filter((project) => {
    const matchesSearch = project.name.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFormat = filterFormat === 'all' || project.format === filterFormat;
    return matchesSearch && matchesFormat;
  });

  const formats = Array.from(new Set(projects.map((project) => project.format)));

  return (
    <div className="flex h-full flex-col bg-gray-50 dark:bg-gray-900">
      <div className="border-b border-gray-200 bg-white p-6 dark:border-gray-700 dark:bg-gray-800">
        <div className="mb-4 flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900 dark:text-white">我的项目</h1>

          <div className="flex gap-2">
            <button
              onClick={() => void loadProjects()}
              disabled={loading}
              className="rounded-lg px-4 py-2 text-gray-700 transition-colors hover:bg-gray-100 disabled:opacity-50 dark:text-gray-300 dark:hover:bg-gray-700"
            >
              <RefreshCw className={`h-5 w-5 ${loading ? 'animate-spin' : ''}`} />
            </button>

            <button
              onClick={onCreateNew}
              className="flex items-center gap-2 rounded-lg bg-blue-500 px-4 py-2 text-white transition-colors hover:bg-blue-600"
            >
              <Plus className="h-5 w-5" />
              新建项目
            </button>
          </div>
        </div>

        <div className="flex gap-4">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 text-gray-400" />
            <input
              type="text"
              placeholder="搜索项目..."
              value={searchQuery}
              onChange={(event) => setSearchQuery(event.target.value)}
              className="w-full rounded-lg border border-gray-300 bg-white py-2 pl-10 pr-4 text-gray-900 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            />
          </div>

          <select
            value={filterFormat}
            onChange={(event) => setFilterFormat(event.target.value)}
            className="rounded-lg border border-gray-300 bg-white px-4 py-2 text-gray-900 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
          >
            <option value="all">所有格式</option>
            {formats.map((format) => (
              <option key={format} value={format}>
                {format}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {loading && projects.length === 0 ? (
          <div className="flex h-64 items-center justify-center">
            <div className="text-gray-500 dark:text-gray-400">加载中...</div>
          </div>
        ) : filteredProjects.length === 0 ? (
          <div className="flex h-64 flex-col items-center justify-center text-gray-500 dark:text-gray-400">
            <p className="mb-2 text-lg">
              {searchQuery || filterFormat !== 'all' ? '没有找到匹配的项目' : '还没有项目'}
            </p>

            {!searchQuery && filterFormat === 'all' && (
              <button
                onClick={onCreateNew}
                className="mt-4 rounded-lg bg-blue-500 px-6 py-3 text-white transition-colors hover:bg-blue-600"
              >
                创建第一个项目
              </button>
            )}
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {filteredProjects.map((project) => (
              <ProjectCard
                key={project.path}
                project={project}
                onOpen={onOpenProject}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
