import { useMemo, useState } from 'react';
import { ArrowLeft, ArrowRight, Check, X } from 'lucide-react';
import { useProject } from '../../hooks/useProject';

interface NewProjectWizardProps {
  onClose: () => void;
  onSuccess: (projectPath: string) => void;
}

const CANVAS_FORMATS = [
  { key: 'ppt169', name: 'PPT 16:9', width: 1280, height: 720 },
  { key: 'ppt43', name: 'PPT 4:3', width: 1024, height: 768 },
  { key: 'xiaohongshu', name: '小红书', width: 1242, height: 1660 },
  { key: 'moments', name: '朋友圈/Instagram', width: 1080, height: 1080 },
  { key: 'story', name: 'Story', width: 1080, height: 1920 },
  { key: 'wechat', name: '公众号头图', width: 900, height: 383 },
  { key: 'banner', name: '横版 Banner', width: 1920, height: 1080 },
  { key: 'a4', name: 'A4 打印', width: 1240, height: 1754 },
];

export function NewProjectWizard({ onClose, onSuccess }: NewProjectWizardProps) {
  const [step, setStep] = useState(1);
  const [projectName, setProjectName] = useState('');
  const [selectedFormat, setSelectedFormat] = useState('ppt169');

  const { createProject, error, loading } = useProject();

  const selectedFormatName = useMemo(() => {
    return CANVAS_FORMATS.find((item) => item.key === selectedFormat)?.name ?? selectedFormat;
  }, [selectedFormat]);

  const handleCreate = async () => {
    if (!projectName.trim()) {
      return;
    }

    try {
      const projectPath = await createProject({
        name: projectName.trim(),
        format: selectedFormat,
      });
      onSuccess(projectPath);
    } catch (createError) {
      console.error('创建项目失败:', createError);
    }
  };

  const canGoNext = step === 1 ? projectName.trim().length > 0 : true;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="mx-4 w-full max-w-2xl rounded-lg bg-white shadow-xl dark:bg-gray-800">
        <div className="flex items-center justify-between border-b border-gray-200 p-6 dark:border-gray-700">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">创建新项目</h2>

          <button onClick={onClose} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
            <X className="h-6 w-6" />
          </button>
        </div>

        <div className="flex items-center justify-center gap-4 border-b border-gray-200 p-6 dark:border-gray-700">
          {[1, 2, 3].map((s) => (
            <div key={s} className="flex items-center">
              <div
                className={`flex h-8 w-8 items-center justify-center rounded-full font-medium ${
                  s === step
                    ? 'bg-blue-500 text-white'
                    : s < step
                      ? 'bg-green-500 text-white'
                      : 'bg-gray-200 text-gray-500 dark:bg-gray-700'
                }`}
              >
                {s < step ? <Check className="h-5 w-5" /> : s}
              </div>

              {s < 3 && (
                <div className={`mx-2 h-1 w-16 ${s < step ? 'bg-green-500' : 'bg-gray-200 dark:bg-gray-700'}`} />
              )}
            </div>
          ))}
        </div>

        <div className="min-h-[300px] p-6">
          {step === 1 && (
            <div>
              <h3 className="mb-4 text-lg font-medium text-gray-900 dark:text-white">输入项目名称</h3>
              <input
                type="text"
                value={projectName}
                onChange={(event) => setProjectName(event.target.value)}
                placeholder="例如：产品介绍PPT"
                className="w-full rounded-lg border border-gray-300 bg-white px-4 py-3 text-gray-900 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                autoFocus
              />
              <p className="mt-2 text-sm text-gray-500 dark:text-gray-400">项目名称将用于创建项目文件夹</p>
            </div>
          )}

          {step === 2 && (
            <div>
              <h3 className="mb-4 text-lg font-medium text-gray-900 dark:text-white">选择画布格式</h3>
              <div className="grid grid-cols-2 gap-4">
                {CANVAS_FORMATS.map((format) => (
                  <button
                    key={format.key}
                    onClick={() => setSelectedFormat(format.key)}
                    className={`rounded-lg border-2 p-4 text-left transition-all ${
                      selectedFormat === format.key
                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                        : 'border-gray-200 hover:border-gray-300 dark:border-gray-700 dark:hover:border-gray-600'
                    }`}
                  >
                    <div className="mb-1 font-medium text-gray-900 dark:text-white">{format.name}</div>
                    <div className="text-sm text-gray-500 dark:text-gray-400">
                      {format.width} × {format.height}
                    </div>
                  </button>
                ))}
              </div>
            </div>
          )}

          {step === 3 && (
            <div>
              <h3 className="mb-4 text-lg font-medium text-gray-900 dark:text-white">确认项目信息</h3>

              <div className="space-y-4 rounded-lg bg-gray-50 p-4 dark:bg-gray-700/50">
                <div>
                  <div className="text-sm text-gray-500 dark:text-gray-400">项目名称</div>
                  <div className="text-lg font-medium text-gray-900 dark:text-white">{projectName}</div>
                </div>
                <div>
                  <div className="text-sm text-gray-500 dark:text-gray-400">画布格式</div>
                  <div className="text-lg font-medium text-gray-900 dark:text-white">{selectedFormatName}</div>
                </div>
              </div>

              {error && (
                <div className="mt-4 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-600 dark:border-red-800 dark:bg-red-900/20 dark:text-red-400">
                  {error}
                </div>
              )}
            </div>
          )}
        </div>

        <div className="flex items-center justify-between border-t border-gray-200 p-6 dark:border-gray-700">
          <button
            onClick={() => (step === 1 ? onClose() : setStep(step - 1))}
            disabled={loading}
            className="flex items-center gap-2 rounded-lg px-4 py-2 text-gray-700 transition-colors hover:bg-gray-100 disabled:opacity-50 dark:text-gray-300 dark:hover:bg-gray-700"
          >
            <ArrowLeft className="h-4 w-4" />
            {step === 1 ? '取消' : '上一步'}
          </button>

          {step < 3 ? (
            <button
              onClick={() => setStep(step + 1)}
              disabled={!canGoNext}
              className="flex items-center gap-2 rounded-lg bg-blue-500 px-4 py-2 text-white transition-colors hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
            >
              下一步
              <ArrowRight className="h-4 w-4" />
            </button>
          ) : (
            <button
              onClick={handleCreate}
              disabled={loading}
              className="flex items-center gap-2 rounded-lg bg-green-500 px-6 py-2 text-white transition-colors hover:bg-green-600 disabled:opacity-50"
            >
              {loading ? '创建中...' : '创建项目'}
              <Check className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
