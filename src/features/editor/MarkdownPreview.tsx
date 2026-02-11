import { useState, useEffect } from 'react'
import ReactMarkdown from 'react-markdown'
import { Eye, Edit, Save, X } from 'lucide-react'

interface MarkdownPreviewProps {
  content: string
  editable?: boolean
  onContentChange?: (content: string) => void
  maxHeight?: string
}

/**
 * Markdown 预览组件
 * 支持代码高亮、表格、图片
 * 可切换编辑模式
 */
export function MarkdownPreview({
  content,
  editable = false,
  onContentChange,
  maxHeight = '600px'
}: MarkdownPreviewProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [editContent, setEditContent] = useState(content)

  // 同步外部内容变化
  useEffect(() => {
    if (!isEditing) {
      setEditContent(content)
    }
  }, [content, isEditing])

  // 保存编辑
  const handleSave = () => {
    onContentChange?.(editContent)
    setIsEditing(false)
  }

  // 取消编辑
  const handleCancel = () => {
    setEditContent(content)
    setIsEditing(false)
  }

  return (
    <div className="flex flex-col h-full bg-white border border-gray-200 rounded-lg">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200">
        <h3 className="text-sm font-semibold text-gray-700">Markdown 预览</h3>
        {editable && (
          <div className="flex items-center gap-2">
            {isEditing ? (
              <>
                <button
                  onClick={handleSave}
                  className="flex items-center gap-1 px-2 py-1 text-xs text-white bg-blue-500 hover:bg-blue-600 rounded transition"
                  title="保存"
                >
                  <Save className="w-3 h-3" />
                  保存
                </button>
                <button
                  onClick={handleCancel}
                  className="flex items-center gap-1 px-2 py-1 text-xs text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition"
                  title="取消"
                >
                  <X className="w-3 h-3" />
                  取消
                </button>
              </>
            ) : (
              <button
                onClick={() => setIsEditing(true)}
                className="flex items-center gap-1 px-2 py-1 text-xs text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition"
                title="编辑"
              >
                <Edit className="w-3 h-3" />
                编辑
              </button>
            )}
          </div>
        )}
      </div>

      {/* 内容区域 */}
      <div
        className="flex-1 overflow-y-auto p-4"
        style={{ maxHeight }}
      >
        {isEditing ? (
          // 编辑模式
          <textarea
            value={editContent}
            onChange={(e) => setEditContent(e.target.value)}
            className="w-full h-full min-h-[400px] p-3 font-mono text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
            placeholder="输入 Markdown 内容..."
          />
        ) : (
          // 预览模式
          <div className="prose prose-sm max-w-none">
            {content ? (
              <ReactMarkdown
                components={{
                  // 代码块样式
                  code({ className, children, ...props }) {
                    const isBlock = Boolean(className)

                    if (!isBlock) {
                      return (
                        <code
                          className="rounded bg-gray-100 px-1.5 py-0.5 font-mono text-sm text-red-600"
                          {...props}
                        >
                          {children}
                        </code>
                      )
                    }

                    return (
                      <pre className="my-4 overflow-x-auto rounded-lg bg-gray-900 p-4 text-gray-100">
                        <code className={className} {...props}>
                          {children}
                        </code>
                      </pre>
                    )
                  },
                  // 表格样式
                  table({ children }) {
                    return (
                      <div className="overflow-x-auto">
                        <table className="min-w-full divide-y divide-gray-200 border border-gray-300">
                          {children}
                        </table>
                      </div>
                    )
                  },
                  thead({ children }) {
                    return (
                      <thead className="bg-gray-50">
                        {children}
                      </thead>
                    )
                  },
                  th({ children }) {
                    return (
                      <th className="px-4 py-2 text-left text-xs font-medium text-gray-700 uppercase tracking-wider border border-gray-300">
                        {children}
                      </th>
                    )
                  },
                  td({ children }) {
                    return (
                      <td className="px-4 py-2 text-sm text-gray-900 border border-gray-300">
                        {children}
                      </td>
                    )
                  },
                  // 图片样式
                  img({ src, alt }) {
                    return (
                      <img
                        src={src}
                        alt={alt}
                        className="max-w-full h-auto rounded-lg shadow-md my-4"
                        loading="lazy"
                      />
                    )
                  },
                  // 链接样式
                  a({ href, children }) {
                    return (
                      <a
                        href={href}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-blue-600 hover:text-blue-800 underline"
                      >
                        {children}
                      </a>
                    )
                  },
                  // 标题样式
                  h1({ children }) {
                    return <h1 className="text-2xl font-bold mt-6 mb-4 text-gray-900">{children}</h1>
                  },
                  h2({ children }) {
                    return <h2 className="text-xl font-bold mt-5 mb-3 text-gray-900">{children}</h2>
                  },
                  h3({ children }) {
                    return <h3 className="text-lg font-semibold mt-4 mb-2 text-gray-900">{children}</h3>
                  },
                  // 段落样式
                  p({ children }) {
                    return <p className="mb-4 text-gray-700 leading-relaxed">{children}</p>
                  },
                  // 列表样式
                  ul({ children }) {
                    return <ul className="list-disc list-inside mb-4 space-y-1 text-gray-700">{children}</ul>
                  },
                  ol({ children }) {
                    return <ol className="list-decimal list-inside mb-4 space-y-1 text-gray-700">{children}</ol>
                  },
                  // 引用样式
                  blockquote({ children }) {
                    return (
                      <blockquote className="border-l-4 border-gray-300 pl-4 py-2 my-4 italic text-gray-600 bg-gray-50">
                        {children}
                      </blockquote>
                    )
                  },
                  // 水平线
                  hr() {
                    return <hr className="my-6 border-t border-gray-300" />
                  }
                }}
              >
                {content}
              </ReactMarkdown>
            ) : (
              <div className="flex flex-col items-center justify-center h-full text-gray-400">
                <Eye className="w-12 h-12 mb-2" />
                <p className="text-sm">暂无内容</p>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  )
}


