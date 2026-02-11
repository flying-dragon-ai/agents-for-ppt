import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Settings,
  Moon,
  Sun,
  FolderOpen,
  CheckCircle2,
  XCircle,
  Loader2,
  Download,
  AlertTriangle
} from 'lucide-react'
import { ApiKeyForm } from './ApiKeyForm'

export interface EnvironmentStatus {
  pythonInstalled: boolean
  pythonVersion?: string
  dependenciesInstalled: boolean
  missingDependencies?: string[]
}

export interface SettingsConfig {
  pythonPath: string
  theme: 'light' | 'dark'
}

/**
 * 设置页面组件
 * 包含 API Key 配置、Python 路径配置、主题切换、环境检测
 */
export function SettingsPage() {
  const [config, setConfig] = useState<SettingsConfig>({
    pythonPath: '',
    theme: 'light'
  })
  const [envStatus, setEnvStatus] = useState<EnvironmentStatus | null>(null)
  const [isCheckingEnv, setIsCheckingEnv] = useState(false)
  const [isInstallingDeps, setIsInstallingDeps] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [saveMessage, setSaveMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null)

  // 加载配置
  useEffect(() => {
    loadConfig()
    checkEnvironment()
  }, [])

  // 加载配置
  const loadConfig = useCallback(async () => {
    try {
      const savedConfig = await invoke<SettingsConfig | null>('cmd_get_settings_config')
      if (savedConfig) {
        setConfig(savedConfig)
        // 应用主题
        applyTheme(savedConfig.theme)
      }
    } catch (err) {
      console.error('加载配置失败:', err)
    }
  }, [])

  // 应用主题
  const applyTheme = useCallback((theme: 'light' | 'dark') => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [])

  // 切换主题
  const toggleTheme = useCallback(() => {
    const newTheme = config.theme === 'light' ? 'dark' : 'light'
    setConfig(prev => ({ ...prev, theme: newTheme }))
    applyTheme(newTheme)
  }, [config.theme, applyTheme])

  // 选择 Python 路径
  const handleSelectPythonPath = useCallback(async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Python 可执行文件',
            extensions: ['exe', 'py']
          }
        ]
      })

      if (selected && typeof selected === 'string') {
        setConfig(prev => ({ ...prev, pythonPath: selected }))
      }
    } catch (err) {
      console.error('选择 Python 路径失败:', err)
    }
  }, [])

  // 检测环境
  const checkEnvironment = useCallback(async () => {
    setIsCheckingEnv(true)
    try {
      const status = await invoke<EnvironmentStatus>('cmd_check_environment', {
        pythonPath: config.pythonPath || undefined
      })
      setEnvStatus(status)
    } catch (err) {
      console.error('环境检测失败:', err)
      setEnvStatus({
        pythonInstalled: false,
        dependenciesInstalled: false
      })
    } finally {
      setIsCheckingEnv(false)
    }
  }, [config.pythonPath])

  // 一键安装依赖
  const handleInstallDependencies = useCallback(async () => {
    setIsInstallingDeps(true)
    try {
      await invoke('cmd_install_dependencies', {
        pythonPath: config.pythonPath || undefined
      })

      // 重新检测环境
      await checkEnvironment()

      setSaveMessage({ type: 'success', text: '依赖安装成功' })
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '依赖安装失败'
      setSaveMessage({ type: 'error', text: errorMsg })
    } finally {
      setIsInstallingDeps(false)
      // 3秒后清除消息
      setTimeout(() => setSaveMessage(null), 3000)
    }
  }, [config.pythonPath, checkEnvironment])

  // 保存配置
  const handleSaveConfig = useCallback(async () => {
    setIsSaving(true)
    try {
      await invoke('cmd_save_settings_config', { config })
      setSaveMessage({ type: 'success', text: '配置已保存' })
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : '保存配置失败'
      setSaveMessage({ type: 'error', text: errorMsg })
    } finally {
      setIsSaving(false)
      // 3秒后清除消息
      setTimeout(() => setSaveMessage(null), 3000)
    }
  }, [config])

  return (
    <div className="container mx-auto p-6 max-w-4xl">
      {/* 页面标题 */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <Settings className="w-8 h-8 text-gray-700" />
          <h1 className="text-3xl font-bold text-gray-800">设置</h1>
        </div>

        {/* 主题切换 */}
        <button
          onClick={toggleTheme}
          className="p-2 rounded-lg hover:bg-gray-100 transition"
          title={config.theme === 'light' ? '切换到暗色模式' : '切换到亮色模式'}
        >
          {config.theme === 'light' ? (
            <Moon className="w-6 h-6 text-gray-600" />
          ) : (
            <Sun className="w-6 h-6 text-yellow-500" />
          )}
        </button>
      </div>

      <div className="space-y-6">
        {/* API Key 配置 */}
        <ApiKeyForm
          onSave={() => {
            setSaveMessage({ type: 'success', text: 'API Key 配置已保存' })
            setTimeout(() => setSaveMessage(null), 3000)
          }}
          onError={(error) => {
            setSaveMessage({ type: 'error', text: error })
            setTimeout(() => setSaveMessage(null), 3000)
          }}
        />

        {/* Python 配置 */}
        <div className="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-lg">
          <h3 className="text-lg font-semibold text-gray-800">Python 配置</h3>

          {/* Python 路径 */}
          <div className="flex flex-col gap-2">
            <label className="text-sm font-medium text-gray-700">Python 路径</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={config.pythonPath}
                onChange={(e) => setConfig(prev => ({ ...prev, pythonPath: e.target.value }))}
                placeholder="留空使用系统默认 Python"
                className="flex-1 px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <button
                onClick={handleSelectPythonPath}
                className="px-4 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition"
                title="选择 Python 可执行文件"
              >
                <FolderOpen className="w-5 h-5" />
              </button>
            </div>
          </div>

          {/* 环境检测 */}
          <div className="flex flex-col gap-3 p-3 bg-gray-50 rounded border border-gray-200">
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-medium text-gray-700">环境状态</h4>
              <button
                onClick={checkEnvironment}
                disabled={isCheckingEnv}
                className="px-3 py-1 text-sm bg-white border border-gray-300 rounded hover:bg-gray-50 transition disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isCheckingEnv ? (
                  <div className="flex items-center gap-1">
                    <Loader2 className="w-3 h-3 animate-spin" />
                    <span>检测中...</span>
                  </div>
                ) : (
                  '重新检测'
                )}
              </button>
            </div>

            {envStatus && (
              <div className="space-y-2">
                {/* Python 状态 */}
                <div className="flex items-center gap-2 text-sm">
                  {envStatus.pythonInstalled ? (
                    <>
                      <CheckCircle2 className="w-4 h-4 text-green-500" />
                      <span className="text-gray-700">
                        Python 已安装 {envStatus.pythonVersion && `(${envStatus.pythonVersion})`}
                      </span>
                    </>
                  ) : (
                    <>
                      <XCircle className="w-4 h-4 text-red-500" />
                      <span className="text-red-700">Python 未安装</span>
                    </>
                  )}
                </div>

                {/* 依赖状态 */}
                <div className="flex items-center gap-2 text-sm">
                  {envStatus.dependenciesInstalled ? (
                    <>
                      <CheckCircle2 className="w-4 h-4 text-green-500" />
                      <span className="text-gray-700">依赖已安装</span>
                    </>
                  ) : (
                    <>
                      <XCircle className="w-4 h-4 text-red-500" />
                      <span className="text-red-700">依赖未安装</span>
                    </>
                  )}
                </div>

                {/* 缺失的依赖 */}
                {envStatus.missingDependencies && envStatus.missingDependencies.length > 0 && (
                  <div className="flex flex-col gap-1 p-2 bg-yellow-50 border border-yellow-200 rounded">
                    <div className="flex items-center gap-2 text-sm text-yellow-800">
                      <AlertTriangle className="w-4 h-4" />
                      <span className="font-medium">缺失的依赖:</span>
                    </div>
                    <ul className="text-xs text-yellow-700 ml-6 list-disc">
                      {envStatus.missingDependencies.map((dep, index) => (
                        <li key={index}>{dep}</li>
                      ))}
                    </ul>
                  </div>
                )}

                {/* 一键安装按钮 */}
                {!envStatus.dependenciesInstalled && envStatus.pythonInstalled && (
                  <button
                    onClick={handleInstallDependencies}
                    disabled={isInstallingDeps}
                    className="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {isInstallingDeps ? (
                      <div className="flex items-center justify-center gap-2">
                        <Loader2 className="w-4 h-4 animate-spin" />
                        <span>安装中...</span>
                      </div>
                    ) : (
                      <div className="flex items-center justify-center gap-2">
                        <Download className="w-4 h-4" />
                        <span>一键安装依赖</span>
                      </div>
                    )}
                  </button>
                )}
              </div>
            )}
          </div>
        </div>

        {/* 保存按钮 */}
        <div className="flex flex-col gap-3">
          <button
            onClick={handleSaveConfig}
            disabled={isSaving}
            className="w-full px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed font-medium"
          >
            {isSaving ? (
              <div className="flex items-center justify-center gap-2">
                <Loader2 className="w-5 h-5 animate-spin" />
                <span>保存中...</span>
              </div>
            ) : (
              '保存所有设置'
            )}
          </button>

          {/* 保存消息 */}
          {saveMessage && (
            <div
              className={`
                flex items-center gap-2 p-3 rounded border
                ${saveMessage.type === 'success'
                  ? 'bg-green-50 border-green-200 text-green-800'
                  : 'bg-red-50 border-red-200 text-red-800'
                }
              `}
            >
              {saveMessage.type === 'success' ? (
                <CheckCircle2 className="w-4 h-4 flex-shrink-0" />
              ) : (
                <XCircle className="w-4 h-4 flex-shrink-0" />
              )}
              <span className="text-sm">{saveMessage.text}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
