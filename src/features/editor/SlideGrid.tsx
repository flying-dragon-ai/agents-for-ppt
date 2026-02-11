import { useState, useEffect } from 'react'

export interface Slide {
  id: string
  path: string
  thumbnail?: string
  index: number
}

interface SlideGridProps {
  slides: Slide[]
  currentSlideId: string | null
  onSlideSelect: (slideId: string) => void
  onSlideReorder?: (fromIndex: number, toIndex: number) => void
}

/**
 * 幻灯片缩略图网格组件
 * 显示所有 SVG 缩略图，支持点击切换、高亮当前页、拖拽排序
 */
export function SlideGrid({
  slides,
  currentSlideId,
  onSlideSelect,
  onSlideReorder,
}: SlideGridProps) {
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null)
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null)

  // 处理拖拽开始
  const handleDragStart = (e: React.DragEvent, index: number) => {
    setDraggedIndex(index)
    e.dataTransfer.effectAllowed = 'move'
  }

  // 处理拖拽结束
  const handleDragEnd = () => {
    setDraggedIndex(null)
    setHoveredIndex(null)
  }

  // 处理拖拽悬停
  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault()
    if (draggedIndex !== null && draggedIndex !== index) {
      setHoveredIndex(index)
    }
  }

  // 处理放置
  const handleDrop = (e: React.DragEvent, toIndex: number) => {
    e.preventDefault()
    if (draggedIndex !== null && draggedIndex !== toIndex && onSlideReorder) {
      onSlideReorder(draggedIndex, toIndex)
    }
    setDraggedIndex(null)
    setHoveredIndex(null)
  }

  // 键盘导航
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!currentSlideId || slides.length === 0) return

      const currentIndex = slides.findIndex((s) => s.id === currentSlideId)
      if (currentIndex === -1) return

      if (e.key === 'ArrowUp' && currentIndex > 0) {
        e.preventDefault()
        onSlideSelect(slides[currentIndex - 1].id)
      } else if (e.key === 'ArrowDown' && currentIndex < slides.length - 1) {
        e.preventDefault()
        onSlideSelect(slides[currentIndex + 1].id)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [currentSlideId, slides, onSlideSelect])

  if (slides.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-gray-400">
        暂无幻灯片
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-white border border-gray-200 rounded-lg">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200">
        <h3 className="text-sm font-semibold text-gray-700">幻灯片</h3>
        <span className="text-xs text-gray-500">{slides.length} 页</span>
      </div>

      {/* 缩略图网格 */}
      <div className="flex-1 overflow-y-auto p-2">
        <div className="space-y-2">
          {slides.map((slide, index) => {
            const isActive = slide.id === currentSlideId
            const isDragging = draggedIndex === index
            const isHovered = hoveredIndex === index

            return (
              <div
                key={slide.id}
                draggable={!!onSlideReorder}
                onDragStart={(e) => handleDragStart(e, index)}
                onDragEnd={handleDragEnd}
                onDragOver={(e) => handleDragOver(e, index)}
                onDrop={(e) => handleDrop(e, index)}
                onClick={() => onSlideSelect(slide.id)}
                className={`
                  relative group cursor-pointer rounded-lg border-2 transition-all
                  ${isActive ? 'border-blue-500 bg-blue-50' : 'border-gray-200 hover:border-gray-300'}
                  ${isDragging ? 'opacity-50' : ''}
                  ${isHovered ? 'border-blue-300' : ''}
                `}
              >
                {/* 页码 */}
                <div className="absolute top-2 left-2 z-10">
                  <span
                    className={`
                      inline-block px-2 py-0.5 text-xs font-semibold rounded
                      ${isActive ? 'bg-blue-500 text-white' : 'bg-gray-800 text-white'}
                    `}
                  >
                    {index + 1}
                  </span>
                </div>

                {/* 缩略图 */}
                <div className="aspect-video bg-gray-100 rounded-lg overflow-hidden">
                  {slide.thumbnail ? (
                    <img
                      src={slide.thumbnail}
                      alt={`Slide ${index + 1}`}
                      className="w-full h-full object-contain"
                    />
                  ) : (
                    <div className="flex items-center justify-center h-full text-gray-400">
                      <svg
                        className="w-12 h-12"
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
                    </div>
                  )}
                </div>

                {/* 拖拽提示 */}
                {onSlideReorder && (
                  <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none">
                    <div className="bg-black bg-opacity-50 text-white text-xs px-2 py-1 rounded">
                      拖拽排序
                    </div>
                  </div>
                )}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}

