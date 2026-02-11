import { useEffect, useRef } from 'react'
import { AlertCircle, Info, AlertTriangle, X } from 'lucide-react'

export type LogLevel = 'info' | 'warning' | 'error'

export interface LogMessage {
  id: string
  level: LogLevel
  message: string
  timestamp: Date
}

interface ConsolePanelProps {
  logs: LogMessage[]
  onClear?: () => void
  maxHeight?: string
}

/**
 * 控制台面板组件
 * 显示日志消息，支持不同级别（info, warning, error）
 * 自动滚动到底部，支持清空日志
 */
export function ConsolePanel({ logs, onClear, maxHeight = '300px' }: ConsolePanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null)

  // 自动滚动到底部
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [logs])

  const getLogIcon = (level: LogLevel) => {
    switch (level) {
      case 'info':
        return <Info className="w-4 h-4 text-blue-500" />
      case 'warning':
        return <AlertTriangle className="w-4 h-4 text-yellow-500" />
      case 'error':
        return <AlertCircle className="w-4 h-4 text-red-500" />
    }
  }

  const getLogColor = (level: LogLevel) => {
    switch (level) {
      case 'info':
        return 'text-gray-700 bg-blue-50 border-blue-200'
      case 'warning':
        return 'text-yellow-800 bg-yellow-50 border-yellow-200'
      case 'error':
        return 'text-red-800 bg-red-50 border-red-200'
    }
  }

  return (
    <div className="flex flex-col h-full bg-white border border-gray-200 rounded-lg">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200">
        <h3 className="text-sm font-semibold text-gray-700">控制台</h3>
        {onClear && logs.length > 0 && (
          <button
            onClick={onClear}
            className="flex items-center gap-1 px-2 py-1 text-xs text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded transition"
            title="清空日志"
          >
            <X className="w-3 h-3" />
            清空
          </button>
        )}
      </div>

      {/* 日志列表 */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto p-2 space-y-1"
        style={{ maxHeight }}
      >
        {logs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-sm text-gray-400">
            暂无日志
          </div>
        ) : (
          logs.map((log) => (
            <div
              key={log.id}
              className={`flex items-start gap-2 px-3 py-2 text-xs border rounded ${getLogColor(log.level)}`}
            >
              <div className="flex-shrink-0 mt-0.5">{getLogIcon(log.level)}</div>
              <div className="flex-1 min-w-0">
                <div className="font-mono break-words">{log.message}</div>
                <div className="text-xs opacity-60 mt-1">
                  {log.timestamp.toLocaleTimeString()}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
