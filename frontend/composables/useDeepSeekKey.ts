/**
 * DeepSeek API Key 管理（已简化）
 *
 * API Key 和 Base URL 现在由后端 .env 文件统一管理，
 * 前端不再需要传递这些配置。
 */

export function useDeepSeekKey() {
  /**
   * 获取请求头（已简化，不再包含 API Key）
   * 后端从 AppState 中读取配置
   */
  function getHeaders(): Record<string, string> {
    return {
      'Content-Type': 'application/json',
    }
  }

  /**
   * 测试连接
   */
  async function testConnection(): Promise<{ success: boolean; message: string }> {
    try {
      const response = await fetch('/api/health')
      if (response.ok) {
        return { success: true, message: '后端服务连接正常' }
      }
      return { success: false, message: `后端返回 ${response.status}` }
    } catch (e: any) {
      return { success: false, message: e.message || '网络错误' }
    }
  }

  return {
    getHeaders,
    testConnection,
  }
}
