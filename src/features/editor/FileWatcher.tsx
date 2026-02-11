import { useEffect, useRef } from 'react'

interface FileWatcherProps {
  projectPath: string | null
  watchPaths: string[]
  onFileChange: (path: string) => void
  pollInterval?: number
}

/**
 * 文件监听组件
 *
 * 注意：当前使用轮询方式实现文件监听
 *
 * TODO: 集成 tauri-plugin-fs-watch 以实现更高效的文件监听
 * 需要在 src-tauri/Cargo.toml 中添加：
 * tauri-plugin-fs-watch = "2.0"
 *
 * 并在 Rust 代码中注册插件：
 * .plugin(tauri_plugin_fs_watch::init())
 *
 * 前端使用示例：
 * import { watch } from '@tauri-apps/plugin-fs-watch'
 *
 * const unwatch = await watch(
 *   path,
 *   (event) => {
 *     console.log('File changed:', event)
 *     onFileChange(event.path)
 *   },
 *   { recursive: true }
 * )
 */
export function FileWatcher({
  projectPath,
  watchPaths,
  onFileChange,
  pollInterval = 2000,
}: FileWatcherProps) {
  const lastModifiedRef = useRef<Map<string, number>>(new Map())
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    if (!projectPath || watchPaths.length === 0) {
      return
    }

    // 清理之前的定时器
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
    }

    // 初始化文件修改时间
    const initFileStats = async () => {
      for (const path of watchPaths) {
        try {
          const fullPath = `${projectPath}/${path}`
          const response = await fetch(`file://${fullPath}`)
          if (response.ok) {
            const lastModified = response.headers.get('last-modified')
            if (lastModified) {
              lastModifiedRef.current.set(path, new Date(lastModified).getTime())
            }
          }
        } catch (err) {
          console.error(`Failed to get file stats for ${path}:`, err)
        }
      }
    }

    // 检查文件变更
    const checkFileChanges = async () => {
      for (const path of watchPaths) {
        try {
          const fullPath = `${projectPath}/${path}`
          const response = await fetch(`file://${fullPath}`)
          if (response.ok) {
            const lastModified = response.headers.get('last-modified')
            if (lastModified) {
              const newTime = new Date(lastModified).getTime()
              const oldTime = lastModifiedRef.current.get(path)

              if (oldTime && newTime > oldTime) {
                console.log(`File changed: ${path}`)
                onFileChange(fullPath)
              }

              lastModifiedRef.current.set(path, newTime)
            }
          }
        } catch (err) {
          console.error(`Failed to check file ${path}:`, err)
        }
      }
    }

    // 初始化并启动轮询
    initFileStats().then(() => {
      intervalRef.current = setInterval(checkFileChanges, pollInterval)
    })

    // 清理
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current)
      }
    }
  }, [projectPath, watchPaths, onFileChange, pollInterval])

  // 这是一个无 UI 的组件，仅用于副作用
  return null
}

/**
 * 使用 tauri-plugin-fs-watch 的实现示例（需要先安装插件）
 *
 * import { watch } from '@tauri-apps/plugin-fs-watch'
 *
 * export function FileWatcher({
 *   projectPath,
 *   watchPaths,
 *   onFileChange,
 * }: FileWatcherProps) {
 *   useEffect(() => {
 *     if (!projectPath || watchPaths.length === 0) {
 *       return
 *     }
 *
 *     const unwatchers: (() => void)[] = []
 *
 *     const setupWatchers = async () => {
 *       for (const path of watchPaths) {
 *         try {
 *           const fullPath = `${projectPath}/${path}`
 *           const unwatch = await watch(
 *             fullPath,
 *             (event) => {
 *               if (event.type === 'modify' || event.type === 'create') {
 *                 onFileChange(event.path)
 *               }
 *             },
 *             { recursive: true }
 *           )
 *           unwatchers.push(unwatch)
 *         } catch (err) {
 *           console.error(`Failed to watch ${path}:`, err)
 *         }
 *       }
 *     }
 *
 *     setupWatchers()
 *
 *     return () => {
 *       unwatchers.forEach((unwatch) => unwatch())
 *     }
 *   }, [projectPath, watchPaths, onFileChange])
 *
 *   return null
 * }
 */

