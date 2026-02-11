import { useState, useEffect } from 'react'
import { SvgCanvas } from './SvgCanvas'
import { SlideGrid, Slide } from './SlideGrid'
import { ConsolePanel, LogMessage } from './ConsolePanel'

interface WorkspaceLayoutProps {
  projectPath: string | null
  slides: Slide[]
  logs: LogMessage[]
  onClearLogs?: () => void
  onSlideReorder?: (fromIndex: number, toIndex: number) => void
}

/**
 * 工作区布局组件
 * 顶部：进度条
 * 中间：SVG 画布
 * 右侧：缩略图网格 + 控制台面板
 */
export function WorkspaceLayout({
  projectPath,
  slides,
  logs,
  onClearLogs,
  onSlideReorder,
}: WorkspaceLayoutProps) {
  const [currentSlideId, setCurrentSlideId] = useState<string | null>(null)
  const [svgContent, setSvgContent] = useState<string | null>(null)
  const [progress, setProgress] = useState(0)

  // 初始化：选择第一张幻灯片
  useEffect(() => {
    if (slides.length > 0 && !currentSlideId) {
      setCurrentSlideId(slides[0].id)
    }
  }, [slides, currentSlideId])

  // 加载 SVG 内容
  useEffect(() => {
    if (!currentSlideId) {
      setSvgContent(null)
      return
    }

    const slide = slides.find((s) => s.id === currentSlideId)
    if (!slide) {
      setSvgContent(null)
      return
    }

    // 从文件系统加载 SVG
    fetch(`file://${slide.path}`)
      .then((res) => res.text())
      .then((content) => setSvgContent(content))
      .catch((err) => {
        console.error('Failed to load SVG:', err)
        setSvgContent(null)
      })
  }, [currentSlideId, slides])

  // 计算进度
  useEffect(() => {
    if (slides.length === 0) {
      setProgress(0)
      return
    }

    const currentIndex = slides.findIndex((s) => s.id === currentSlideId)
    if (currentIndex === -1) {
      setProgress(0)
      return
    }

    setProgress(((currentIndex + 1) / slides.length) * 100)
  }, [currentSlideId, slides])

  // 翻页
  const handlePrevSlide = () => {
    if (!currentSlideId || slides.length === 0) return

    const currentIndex = slides.findIndex((s) => s.id === currentSlideId)
    if (currentIndex > 0) {
      setCurrentSlideId(slides[currentIndex - 1].id)
    }
  }

  const handleNextSlide = () => {
    if (!currentSlideId || slides.length === 0) return

    const currentIndex = slides.findIndex((s) => s.id === currentSlideId)
    if (currentIndex < slides.length - 1) {
      setCurrentSlideId(slides[currentIndex + 1].id)
    }
  }

  const currentIndex = currentSlideId
    ? slides.findIndex((s) => s.id === currentSlideId)
    : 0

  return (
    <div className="flex flex-col h-screen bg-gray-100">
      {/* 顶部进度条 */}
      <div className="h-1 bg-gray-200">
        <div
          className="h-full bg-blue-500 transition-all duration-300"
          style={{ width: `${progress}%` }}
        />
      </div>

      {/* 主内容区 */}
      <div className="flex-1 flex overflow-hidden">
        {/* 中间：SVG 画布 */}
        <div className="flex-1 flex flex-col min-w-0">
          {projectPath ? (
            <SvgCanvas
              svgContent={svgContent}
              currentIndex={currentIndex}
              totalSlides={slides.length}
              onPrevSlide={handlePrevSlide}
              onNextSlide={handleNextSlide}
            />
          ) : (
            <div className="flex items-center justify-center h-full bg-white">
              <div className="text-center text-gray-400">
                <svg
                  className="w-20 h-20 mx-auto mb-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                  />
                </svg>
                <p className="text-lg font-medium">未打开项目</p>
                <p className="text-sm mt-2">请先创建或打开一个项目</p>
              </div>
            </div>
          )}
        </div>

        {/* 右侧面板 */}
        <div className="w-80 flex flex-col gap-4 p-4 bg-gray-50 border-l border-gray-200">
          {/* 缩略图网格 */}
          <div className="flex-1 min-h-0">
            <SlideGrid
              slides={slides}
              currentSlideId={currentSlideId}
              onSlideSelect={setCurrentSlideId}
              onSlideReorder={onSlideReorder}
            />
          </div>

          {/* 控制台面板 */}
          <div className="h-64">
            <ConsolePanel logs={logs} onClear={onClearLogs} maxHeight="100%" />
          </div>
        </div>
      </div>
    </div>
  )
}
