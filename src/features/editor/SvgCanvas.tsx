import { useRef, useState } from 'react'
import { TransformWrapper, TransformComponent } from 'react-zoom-pan-pinch'
import { useHotkeys } from 'react-hotkeys-hook'
import { ZoomIn, ZoomOut, Maximize2, ChevronLeft, ChevronRight } from 'lucide-react'

interface SvgCanvasProps {
  svgContent: string | null
  currentIndex: number
  totalSlides: number
  onPrevSlide: () => void
  onNextSlide: () => void
  onZoomChange?: (scale: number) => void
}

/**
 * SVG 画布组件
 * 集成 react-zoom-pan-pinch，支持缩放、平移、键盘快捷键
 */
export function SvgCanvas({
  svgContent,
  currentIndex,
  totalSlides,
  onPrevSlide,
  onNextSlide,
  onZoomChange,
}: SvgCanvasProps) {
  const [scale, setScale] = useState(1)
  const transformRef = useRef<any>(null)

  // 键盘快捷键
  useHotkeys('left', onPrevSlide, [onPrevSlide])
  useHotkeys('right', onNextSlide, [onNextSlide])
  useHotkeys('up', onPrevSlide, [onPrevSlide])
  useHotkeys('down', onNextSlide, [onNextSlide])
  useHotkeys('space', onNextSlide, [onNextSlide])
  useHotkeys('shift+space', onPrevSlide, [onPrevSlide])

  // 缩放快捷键
  useHotkeys('ctrl+=', () => transformRef.current?.zoomIn(), [])
  useHotkeys('ctrl+-', () => transformRef.current?.zoomOut(), [])
  useHotkeys('ctrl+0', () => transformRef.current?.resetTransform(), [])

  // 处理缩放变化
  const handleTransform = (ref: any) => {
    const newScale = ref.state.scale
    setScale(newScale)
    onZoomChange?.(newScale)
  }

  // 重置缩放
  const handleReset = () => {
    transformRef.current?.resetTransform()
  }

  // 放大
  const handleZoomIn = () => {
    transformRef.current?.zoomIn()
  }

  // 缩小
  const handleZoomOut = () => {
    transformRef.current?.zoomOut()
  }

  return (
    <div className="flex flex-col h-full bg-gray-50">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 bg-white border-b border-gray-200">
        {/* 左侧：翻页控制 */}
        <div className="flex items-center gap-2">
          <button
            onClick={onPrevSlide}
            disabled={currentIndex === 0}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded disabled:opacity-30 disabled:cursor-not-allowed transition"
            title="上一页 (←)"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <span className="text-sm text-gray-600 min-w-[80px] text-center">
            {currentIndex + 1} / {totalSlides}
          </span>
          <button
            onClick={onNextSlide}
            disabled={currentIndex >= totalSlides - 1}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded disabled:opacity-30 disabled:cursor-not-allowed transition"
            title="下一页 (→)"
          >
            <ChevronRight className="w-5 h-5" />
          </button>
        </div>

        {/* 右侧：缩放控制 */}
        <div className="flex items-center gap-2">
          <button
            onClick={handleZoomOut}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition"
            title="缩小 (Ctrl + -)"
          >
            <ZoomOut className="w-5 h-5" />
          </button>
          <span className="text-sm text-gray-600 min-w-[60px] text-center">
            {Math.round(scale * 100)}%
          </span>
          <button
            onClick={handleZoomIn}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition"
            title="放大 (Ctrl + +)"
          >
            <ZoomIn className="w-5 h-5" />
          </button>
          <button
            onClick={handleReset}
            className="p-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition"
            title="适应窗口 (Ctrl + 0)"
          >
            <Maximize2 className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* SVG 画布 */}
      <div className="flex-1 overflow-hidden">
        {svgContent ? (
          <TransformWrapper
            ref={transformRef}
            initialScale={1}
            minScale={0.1}
            maxScale={5}
            centerOnInit
            onTransformed={handleTransform}
            doubleClick={{ mode: 'reset' }}
            wheel={{ step: 0.1 }}
          >
            <TransformComponent
              wrapperClass="w-full h-full"
              contentClass="w-full h-full flex items-center justify-center"
            >
              <div
                className="bg-white shadow-lg"
                dangerouslySetInnerHTML={{ __html: svgContent }}
              />
            </TransformComponent>
          </TransformWrapper>
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-400">
              <svg
                className="w-16 h-16 mx-auto mb-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
                />
              </svg>
              <p className="text-sm">暂无内容</p>
              <p className="text-xs mt-1">请选择一个幻灯片</p>
            </div>
          </div>
        )}
      </div>

      {/* 快捷键提示 */}
      <div className="px-4 py-2 bg-white border-t border-gray-200">
        <div className="flex items-center justify-center gap-4 text-xs text-gray-500">
          <span>← → 翻页</span>
          <span>Ctrl + / - 缩放</span>
          <span>Ctrl + 0 重置</span>
          <span>双击重置</span>
        </div>
      </div>
    </div>
  )
}

