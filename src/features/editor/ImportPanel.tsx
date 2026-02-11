import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Upload, Link as LinkIcon, FileText, Loader2 } from 'lucide-react'

export interface ImportProgress {
  current: number
  total: number
  message: string
}

interface ImportPanelProps {
  onImportComplete?: (markdown: string) => void
  onError?: (error: string) => void
}

/**
 * 文档导入面板组件
 * 支持 PDF 文件选择、URL 输入、拖拽上传
 * 显示转换进度和错误信息
 */
export function ImportPanel({ onImportComplete, onError }: ImportPanelProps) {
  const [url, setUrl] = useState('')
  const [isConverting, setIsConverting] = useState(false)
  const [progress, setProgress] = useState<ImportProgress | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isDragging, setIsDragging] = useState(false)

  // 处理文件选择
  const handleFileSelect = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'PDF 文件',
            extensions: ['pdf']
          }
        ]
      })

      if (selected && typeof selected === 'string') {
        await convertPdfToMarkdown(selected)
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '文件选择失败'
      setError(errorMsg)
      onError?.(errorMsg)
    }
  }, [onError])

  // 处理 URL 转换
  const handleUrlConvert = useCallback(async () => {
    if (!url.trim()) {
      setError('请输入有效的 URL')
      return
    }

    try {
      await convertUrlToMarkdown(url.trim())
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'URL 转换失败'
      setError(errorMsg)
      onError?.(errorMsg)
    }
  }, [url, onError])

  // PDF 转 Markdown
  const convertPdfToMarkdown = async (filePath: string) => {
    setIsConverting(true)
    setError(null)
    setProgress({ current: 0, total: 100, message: '正在转换 PDF...' })

    try {
      const result = await invoke<string>('cmd_pdf_to_markdown', {
        filePath
      })

      setProgress({ current: 100, total: 100, message: '转换完成' })
      onImportComplete?.(result)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'PDF 转换失败'
      setError(errorMsg)
      onError?.(errorMsg)
    } finally {
      setIsConverting(false)
      setProgress(null)
    }
  }

  // URL 转 Markdown
  const convertUrlToMarkdown = async (urlStr: string) => {
    setIsConverting(true)
    setError(null)
    setProgress({ current: 0, total: 100, message: '正在抓取网页...' })

    try {
      const result = await invoke<string>('cmd_url_to_markdown', {
        url: urlStr
      })

      setProgress({ current: 100, total: 100, message: '转换完成' })
      onImportComplete?.(result)
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'URL 转换失败'
      setError(errorMsg)
      onError?.(errorMsg)
    } finally {
      setIsConverting(false)
      setProgress(null)
    }
  }

  // 处理拖拽事件
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
  }, [])

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)

    const files = Array.from(e.dataTransfer.files)
    const pdfFile = files.find(f => f.name.toLowerCase().endsWith('.pdf'))

    if (pdfFile) {
      // Tauri 环境下需要使用文件路径
      // 注意：这里需要后端支持从 File 对象获取路径
      try {
        // 临时方案：提示用户使用文件选择器
        setError('请使用"选择文件"按钮上传 PDF')
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : '文件上传失败'
        setError(errorMsg)
        onError?.(errorMsg)
      }
    } else {
      setError('请拖拽 PDF 文件')
    }
  }, [onError])

  return (
    <div className="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-lg">
      {/* 标题 */}
      <div className="flex items-center gap-2">
        <FileText className="w-5 h-5 text-gray-600" />
        <h3 className="text-lg font-semibold text-gray-800">文档导入</h3>
      </div>

      {/* 拖拽上传区域 */}
      <div
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        className={`
          relative border-2 border-dashed rounded-lg p-8 text-center transition-colors
          ${isDragging ? 'border-blue-500 bg-blue-50' : 'border-gray-300 bg-gray-50'}
          ${isConverting ? 'opacity-50 pointer-events-none' : 'hover:border-gray-400'}
        `}
      >
        <Upload className={`w-12 h-12 mx-auto mb-4 ${isDragging ? 'text-blue-500' : 'text-gray-400'}`} />
        <p className="text-sm text-gray-600 mb-2">
          拖拽 PDF 文件到此处，或
        </p>
        <button
          onClick={handleFileSelect}
          disabled={isConverting}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          选择文件
        </button>
      </div>

      {/* URL 输入 */}
      <div className="flex flex-col gap-2">
        <label className="flex items-center gap-2 text-sm font-medium text-gray-700">
          <LinkIcon className="w-4 h-4" />
          或输入网页 URL
        </label>
        <div className="flex gap-2">
          <input
            type="url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleUrlConvert()}
            placeholder="https://example.com/article"
            disabled={isConverting}
            className="flex-1 px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          />
          <button
            onClick={handleUrlConvert}
            disabled={isConverting || !url.trim()}
            className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed"
          >
            转换
          </button>
        </div>
      </div>

      {/* 进度显示 */}
      {progress && (
        <div className="flex flex-col gap-2 p-3 bg-blue-50 border border-blue-200 rounded">
          <div className="flex items-center gap-2 text-sm text-blue-800">
            <Loader2 className="w-4 h-4 animate-spin" />
            <span>{progress.message}</span>
          </div>
          <div className="w-full bg-blue-200 rounded-full h-2">
            <div
              className="bg-blue-500 h-2 rounded-full transition-all duration-300"
              style={{ width: `${(progress.current / progress.total) * 100}%` }}
            />
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
