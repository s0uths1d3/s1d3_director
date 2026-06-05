import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface ProjectItem {
  id: string
  title: string
  description: string
  style: string
  status: string
  owner: string
  novel_preview?: string | null
  script_id?: string | null
  created_at: string
  updated_at: string
}

export interface ProjectListData {
  projects: ProjectItem[]
  total: number
  page: number
  page_size: number
}

export const useProjectStore = defineStore('project', () => {
  const projectList = ref<ProjectListData>({ projects: [], total: 0, page: 1, page_size: 10 })
  const loading = ref(false)
  const error = ref('')

  async function fetchProjects(page: number = 1, options?: { search?: string; status?: string }) {
    loading.value = true
    error.value = ''

    try {
      const params = new URLSearchParams()
      params.set('page', String(page))
      params.set('page_size', '10')
      if (options?.search?.trim()) params.set('search', options.search.trim())
      if (options?.status) params.set('status', options.status)

      const res = await fetch(`/api/projects?${params.toString()}`)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)

      const data: ProjectListData = await res.json()
      projectList.value = data
    } catch (e: any) {
      error.value = e.message || '加载失败，请重试'
    } finally {
      loading.value = false
    }
  }

  /** 创建项目后自动追加到列表（乐观更新） */
  function addProjectToLocal(project: ProjectItem) {
    // 避免重复添加
    const exists = projectList.value.projects.some(p => p.id === project.id)
    if (!exists) {
      projectList.value.projects.unshift(project)
      projectList.value.total += 1
    }
  }

  /** 从本地列表移除项目 */
  function removeProjectLocal(id: string) {
    const idx = projectList.value.projects.findIndex(p => p.id === id)
    if (idx !== -1) {
      projectList.value.projects.splice(idx, 1)
      projectList.value.total -= 1
    }
  }

  return {
    projectList,
    loading,
    error,
    fetchProjects,
    addProjectToLocal,
    removeProjectLocal,
  }
})
