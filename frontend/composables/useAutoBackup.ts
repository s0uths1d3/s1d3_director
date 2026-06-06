import { ref, watchEffect, onMounted, onUnmounted } from 'vue'

const STORAGE_PREFIX = 'editor-auto-backup'
const MAX_BACKUPS = 10
const AUTO_SAVE_INTERVAL_MS = 30_000 // 30 秒

export interface BackupEntry {
  id: string           // 唯一 ID（时间戳）
  label: string        // 标签描述
  yaml: string         // YAML 内容
  timestamp: number    // 创建时间戳
  size: number         // 字节数
}

/**
 * 编辑器自动备份 composable
 *
 * 功能：
 * - 每 30 秒自动将当前内容备份到 localStorage
 * - 手动保存时也会创建快照
 * - 最多保留 10 个本地备份，超出删除最旧的
 * - 支持列出/恢复/清除备份
 */
export function useAutoBackup() {
  const backups = ref<BackupEntry[]>([])
  const lastBackupTime = ref<number>(0)
  let timer: ReturnType<typeof setInterval> | null = null

  /** 从 localStorage 加载备份列表 */
  function loadBackups(): BackupEntry[] {
    try {
      const raw = localStorage.getItem(STORAGE_PREFIX)
      if (!raw) return []
      return JSON.parse(raw) as BackupEntry[]
    } catch { return [] }
  }

  /** 持久化备份列表到 localStorage */
  function persist(list: BackupEntry[]) {
    try {
      localStorage.setItem(STORAGE_PREFIX, JSON.stringify(list))
    } catch { /* storage full */ }
  }

  /** 初始化：加载已有备份 */
  function init() {
    backups.value = loadBackups()
    if (backups.value.length > 0) {
      lastBackupTime.value = backups.value[0].timestamp
    }
  }

  /** 创建一个新备份 */
  function createBackup(yamlContent: string, label = '自动保存'): BackupEntry | null {
    if (!yamlContent || !yamlContent.trim()) return null

    const entry: BackupEntry = {
      id: Date.now().toString(36) + Math.random().toString(36).slice(2, 6),
      label,
      yaml: yamlContent,
      timestamp: Date.now(),
      size: new Blob([yamlContent]).size,
    }

    const list = [entry, ...backups.value]
    // 限制数量
    if (list.length > MAX_BACKUPS) list.splice(MAX_BACKUPS)
    backups.value = list
    persist(list)
    lastBackupTime.value = entry.timestamp
    return entry
  }

  /** 删除指定备份 */
  function removeBackup(id: string) {
    const list = backups.value.filter(b => b.id !== id)
    backups.value = list
    persist(list)
  }

  /** 清除所有本地备份 */
  function clearAllBackups() {
    backups.value = []
    try { localStorage.removeItem(STORAGE_PREFIX) } catch {}
    lastBackupTime.value = 0
  }

  /** 启动定时自动保存（传入获取当前 YAML 内容的函数） */
  function startAutoSave(getYamlFn: () => string | null) {
    stopAutoSave()
    timer = setInterval(() => {
      const yaml = getYamlFn()
      if (yaml) createBackup(yaml, '自动保存')
    }, AUTO_SAVE_INTERVAL_MS)
  }

  /** 停止定时自动保存 */
  function stopAutoSave() {
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  /** 格式化时间差（如 "3分钟前"） */
  function timeAgo(ts: number): string {
    const diff = Date.now() - ts
    const mins = Math.floor(diff / 60000)
    if (mins < 1) return '刚刚'
    if (mins < 60) return `${mins}分钟前`
    const hours = Math.floor(mins / 60)
    if (hours < 24) return `${hours}小时前`
    const days = Math.floor(hours / 24)
    return `${days}天前`
  }

  /** 格式化文件大小 */
  function formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }

  onMounted(() => init())
  onUnmounted(() => stopAutoSave())

  return {
    backups,            // reactive 备份列表
    lastBackupTime,     // 上次备份时间戳
    createBackup,
    removeBackup,
    clearAllBackups,
    startAutoSave,
    stopAutoSave,
    timeAgo,
    formatSize,
  }
}
