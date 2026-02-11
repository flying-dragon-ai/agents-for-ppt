import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { Download, FileText, Loader2, CheckCircle2 } from 'lucide-react'

export interface ExportProgress {
  current: number
  total: number
  message: string
  stage: string
}

export interface PostProcessOptions {
  embedIcons: boolean
  embedImages: boolean
  optimizeSvg: boolean
  validateOutput: boolean
}

interface ExportPanelProps {
  projectPath: string
  onExportComplete?: (outputPath: string) => void
  onError?: (error: string) => void
}

/**
 * 导出面板组件
 * 支持后处理选项配置、导出格式选择、进度显示
 */
export function ExportPanel({ projectPath, onExportComplete, onError }: ExportPanelProps) {
  const [exportFormat, setExportFormat] = useState<'pptx' | 'pdf'>('pptx')
  const [isExporting, setIsExporting] = useState(false)
  const [progress, setProgress] = useState<ExportProgress | null>(null)
  const [error, setError] = useState<string | null>(null)

  // 后处理选项
  const [postProcessOptions, setPostProcessOptions] = useState<PostProcessOptions>({
    embedIcons: true,
    embedImages: true,
    optimizeSvg: true,
    validateOutput: true
  })

  // 切换后处理选项
  const toggleOption = useCallback((key: keyof PostProcessOptions) => {
    setPostProcessOptions(prev => ({
      ...prev,
      [key]: !prev[key]
    }))
  }, [])

  // 处理导出
  const handleExport = useCallback(async () => {
    if (!projectPath) {
      setError('请先选择项目')
      return
    }

    try {
      // 选择保存位置
      const savePath = await save({
        defaultPath: `output.${exportFormat}`,
        filters: [
          {
            name: exportFormat === 'pptx' ? 'PowerPoint 文件' : 'PDF 文件',
            extensions: [exportFormat]
          }
        ]
      })

      if (!savePath) {
        return // 用户取消
      }

      setIsExporting(true)
      setError(null)
      setProgress({
        current: 0,
        total: 100,
        message: '准备导出...',
        stage: 'prepare'
      })

      // 调用后端导出命令
      const result = await invoke<string>('cmd_export_project', {
        projectPath,
        outputPath: savePath,
        format: exportFormat,
        postProcessOptions
      })

      setProgress({
        current: 100,
        total: 100,
        message: '导出完成',
        stage: 'complete'
      })

      onExportComplete?.(result)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '导出失败'
      setError(errorMsg)
      onError?.(errorMsg)
    } finally {
      setIsExporting(false)
      // 3秒后清除进度
      setTimeout(() => setProgress(null), 3000)
    }
  }, [projectPath, exportFormat, postProcessOptions, onExportComplete, onError])

  return (
    <div className="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-lg">
      {/* 标题 */}
      <div className="flex items-center gap-2">
        <Download className="w-5 h-5 text-gray-600" />
        <h3 className="text-lg font-semibold text-gray-800">导出设置</h3>
      </div>

      {/* 后处理选项 */}
      <div className="flex flex-col gap-3">
        <h4 className="text-sm font-medium text-gray-700">后处理选项</h4>
        <div className="space-y-2">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={postProcessOptions.embedIcons}
              onChange={() => toggleOption('embedIcons')}
              disabled={isExporting}
              className="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            />
            <span className="text-sm text-gray-700">嵌入图标</span>
          </label>

          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={postProcessOptions.embedImages}
              onChange={() => toggleOption('embedImages')}
              disabled={isExporting}
              className="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            />
            <span className="text-sm text-gray-700">嵌入图片</span>
          </label>

          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={postProcessOptions.optimizeSvg}
              onChange={() => toggleOption('optimizeSvg')}
              disabled={isExporting}
              className="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            />
            <span className="text-sm text-gray-700">优化 SVG</span>
          </label>

          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={postProcessOptions.validateOutput}
              onChange={() => toggleOption('validateOutput')}
              disabled={isExporting}
              className="w-4 h-4 text-blue-500 border-gray-300 rounded focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            />
            <span className="text-sm text-gray-700">验证输出</span>
          </label>
        </div>
      </div>

      {/* 导出格式选择 */}
      <div className="flex flex-col gap-2">
        <h4 className="text-sm font-medium text-gray-700">导出格式</h4>
        <div className="flex gap-2">
          <button
            onClick={() => setExportFormat('pptx')}
            disabled={isExporting}
            className={`
              flex-1 px-4 py-2 rounded border transition
              ${exportFormat === 'pptx'
                ? 'bg-blue-500 text-white border-blue-500'
                : 'bg-white text-gray-700 border-gray-300 hover:border-gray-400'
              }
              disabled:opacity-50 disabled:cursor-not-allowed
            `}
          >
            <div className="flex items-center justify-center gap-2">
              <FileText className="w-4 h-4" />
              <span>PPTX</span>
            </div>
          </button>

          <button
            onClick={() => setExportFormat('pdf')}
            disabled={isExporting}
            className={`
              flex-1 px-4 py-2 rounded border transition
              ${exportFormat === 'pdf'
                ? 'bg-blue-500 text-white border-blue-500'
                : 'bg-white text-gray-700 border-gray-300 hover:border-gray-400'
              }
              disabled:opacity-50 disabled:cursor-not-allowed
            `}
          >
            <div className="flex items-center justify-center gap-2">
              <FileText className="w-4 h-4" />
              <span>PDF</span>
            </div>
          </button>
        </div>
      </div>

      {/* 导出按钮 */}
      <button
        onClick={handleExport}
        disabled={isExporting || !projectPath}
        className="w-full px-4 py-3 bg-blue-500 text-white rounded hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed font-medium"
      >
        {isExporting ? (
          <div className="flex items-center justify-center gap-2">
            <Loader2 className="w-5 h-5 animate-spin" />
            <span>导出中...</span>
          </div>
        ) : (
          <div className="flex items-center justify-center gap-2">
            <Download className="w-5 h-5" />
            <span>导出为 {exportFormat.toUpperCase()}</span>
          </div>
        )}
      </button>

      {/* 进度显示 */}
      {progress && (
        <div className="flex flex-col gap-2 p-3 bg-blue-50 border border-blue-200 rounded">
          <div className="flex items-center gap-2 text-sm text-blue-800">
            {progress.stage === 'complete' ? (
              <CheckCircle2 className="w-4 h-4 text-green-500" />
            ) : (
              <Loader2 className="w-4 h-4 animate-spin" />
            )}
            <span>{progress.message}</span>
          </div>
          <div className="w-full bg-blue-200 rounded-full h-2">
            <div
              className="bg-blue-500 h-2 rounded-full transition-all duration-300"
              style={{ width: `${(progress.current / progress.total) * 100}%` }}
            />
          </div>
          <div className="text-xs text-blue-600">
            {progress.current} / {progress.total}
          </div>
        </div>
      )}

      {/* 错误显示 */}
      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded">
          <p className="text-sm text-red-800">{error}</p>
        </div>
      )}
    </div>
  )
}
