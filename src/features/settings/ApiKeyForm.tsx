import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Eye, EyeOff, Key, CheckCircle2, XCircle, Loader2 } from 'lucide-react'

export interface ApiKeyConfig {
  provider: 'openai' | 'anthropic' | 'deepseek' | 'custom'
  apiKey: string
  baseUrl?: string
}

interface ApiKeyFormProps {
  onSave?: (config: ApiKeyConfig) => void
  onError?: (error: string) => void
}

/**
 * API Key 配置表单组件
 * 支持多个 AI 服务提供商的 API Key 配置
 * 提供验证、保存到本地存储功能
 */
export function ApiKeyForm({ onSave, onError }: ApiKeyFormProps) {
  const [provider, setProvider] = useState<ApiKeyConfig['provider']>('openai')
  const [apiKey, setApiKey] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [showApiKey, setShowApiKey] = useState(false)
  const [isValidating, setIsValidating] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [validationStatus, setValidationStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [validationMessage, setValidationMessage] = useState('')

  // 从本地存储加载配置
  useEffect(() => {
    loadConfig()
  }, [])

  // 加载配置
  const loadConfig = useCallback(async () => {
    try {
      const config = await invoke<ApiKeyConfig | null>('cmd_get_api_key_config')
      if (config) {
        setProvider(config.provider)
        setApiKey(config.apiKey)
        setBaseUrl(config.baseUrl || '')
      }
    } catch (err) {
      console.error('加载配置失败:', err)
    }
  }, [])

  // 验证 API Key
  const handleValidate = useCallback(async () => {
    if (!apiKey.trim()) {
      setValidationStatus('error')
      setValidationMessage('请输入 API Key')
      return
    }

    setIsValidating(true)
    setValidationStatus('idle')
    setValidationMessage('')

    try {
      const result = await invoke<{ valid: boolean; message: string }>('cmd_validate_api_key', {
        provider,
        apiKey: apiKey.trim(),
        baseUrl: baseUrl.trim() || undefined
      })

      if (result.valid) {
        setValidationStatus('success')
        setValidationMessage(result.message || 'API Key 验证成功')
      } else {
        setValidationStatus('error')
        setValidationMessage(result.message || 'API Key 验证失败')
      }
    } catch (err) {
      setValidationStatus('error')
      const errorMsg = err instanceof Error ? err.message : 'API Key 验证失败'
      setValidationMessage(errorMsg)
      onError?.(errorMsg)
    } finally {
      setIsValidating(false)
    }
  }, [provider, apiKey, baseUrl, onError])

  // 保存配置
  const handleSave = useCallback(async () => {
    if (!apiKey.trim()) {
      setValidationStatus('error')
      setValidationMessage('请输入 API Key')
      return
    }

    setIsSaving(true)

    try {
      const config: ApiKeyConfig = {
        provider,
        apiKey: apiKey.trim(),
        baseUrl: baseUrl.trim() || undefined
      }

      await invoke('cmd_save_api_key_config', { config })

      setValidationStatus('success')
      setValidationMessage('配置已保存')
      onSave?.(config)

      // 3秒后清除状态
      setTimeout(() => {
        setValidationStatus('idle')
        setValidationMessage('')
      }, 3000)
    } catch (err) {
      setValidationStatus('error')
      const errorMsg = err instanceof Error ? err.message : '保存配置失败'
      setValidationMessage(errorMsg)
      onError?.(errorMsg)
    } finally {
      setIsSaving(false)
    }
  }, [provider, apiKey, baseUrl, onSave, onError])

  return (
    <div className="flex flex-col gap-4 p-4 bg-white border border-gray-200 rounded-lg">
      {/* 标题 */}
      <div className="flex items-center gap-2">
        <Key className="w-5 h-5 text-gray-600" />
        <h3 className="text-lg font-semibold text-gray-800">API Key 配置</h3>
      </div>

      {/* 服务提供商选择 */}
      <div className="flex flex-col gap-2">
        <label className="text-sm font-medium text-gray-700">服务提供商</label>
        <select
          value={provider}
          onChange={(e) => setProvider(e.target.value as ApiKeyConfig['provider'])}
          disabled={isValidating || isSaving}
          className="px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <option value="openai">OpenAI</option>
          <option value="anthropic">Anthropic (Claude)</option>
          <option value="deepseek">DeepSeek</option>
          <option value="custom">自定义</option>
        </select>
      </div>

      {/* API Key 输入 */}
      <div className="flex flex-col gap-2">
        <label className="text-sm font-medium text-gray-700">API Key</label>
        <div className="relative">
          <input
            type={showApiKey ? 'text' : 'password'}
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="sk-..."
            disabled={isValidating || isSaving}
            className="w-full px-3 py-2 pr-10 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed font-mono text-sm"
          />
          <button
            type="button"
            onClick={() => setShowApiKey(!showApiKey)}
            className="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-gray-500 hover:text-gray-700 transition"
            title={showApiKey ? '隐藏' : '显示'}
          >
            {showApiKey ? (
              <EyeOff className="w-4 h-4" />
            ) : (
              <Eye className="w-4 h-4" />
            )}
          </button>
        </div>
      </div>

      {/* Base URL 输入（自定义提供商） */}
      {provider === 'custom' && (
        <div className="flex flex-col gap-2">
          <label className="text-sm font-medium text-gray-700">Base URL</label>
          <input
            type="url"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder="https://api.example.com/v1"
            disabled={isValidating || isSaving}
            className="px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          />
        </div>
      )}

      {/* 操作按钮 */}
      <div className="flex gap-2">
        <button
          onClick={handleValidate}
          disabled={isValidating || isSaving || !apiKey.trim()}
          className="flex-1 px-4 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isValidating ? (
            <div className="flex items-center justify-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>验证中...</span>
            </div>
          ) : (
            '验证'
          )}
        </button>

        <button
          onClick={handleSave}
          disabled={isValidating || isSaving || !apiKey.trim()}
          className="flex-1 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isSaving ? (
            <div className="flex items-center justify-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>保存中...</span>
            </div>
          ) : (
            '保存'
          )}
        </button>
      </div>

      {/* 验证状态显示 */}
      {validationStatus !== 'idle' && validationMessage && (
        <div
          className={`
            flex items-center gap-2 p-3 rounded border
            ${validationStatus === 'success'
              ? 'bg-green-50 border-green-200 text-green-800'
              : 'bg-red-50 border-red-200 text-red-800'
            }
          `}
        >
          {validationStatus === 'success' ? (
            <CheckCircle2 className="w-4 h-4 flex-shrink-0" />
          ) : (
            <XCircle className="w-4 h-4 flex-shrink-0" />
          )}
          <span className="text-sm">{validationMessage}</span>
        </div>
      )}

      {/* 提示信息 */}
      <div className="text-xs text-gray-500 space-y-1">
        <p>• API Key 将安全存储在本地</p>
        <p>• 不同的服务提供商需要不同的 API Key</p>
        <p>• 建议先验证 API Key 是否有效再保存</p>
      </div>
    </div>
  )
}
