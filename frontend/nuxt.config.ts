import { defineNuxtConfig } from 'nuxt/config'

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-10-28',
  devtools: { enabled: true },

  modules: [
    '@pinia/nuxt',
    '@nuxtjs/tailwindcss',
  ],

  // CSS 配置
  css: ['~/assets/main.css'],

  // 构建配置
  build: {
    transpile: ['naive-ui', 'vue-echarts', 'echarts'],
  },

  // 运行时配置
  runtimeConfig: {
    // 服务端可用的私有配置
    apiBase: process.env.API_BASE || 'http://localhost:8080',
    // 客户端也可用的公共配置
    public: {
      apiBase: process.env.NUXT_PUBLIC_API_BASE || 'http://localhost:8081',
    },
  },

  // Vite 代理配置：将 /api 请求代理到后端
  vite: {
    server: {
      proxy: {
        '/api': {
          target: 'http://localhost:8081',
          changeOrigin: true,
        },
      },
    },
  },

  // 应用配置
  app: {
    head: {
      title: 'Novel2Script Pro - AI 小说转剧本智能创作平台',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1' },
        {
          hid: 'description',
          name: 'description',
          content: 'AI 驱动的小说转剧本创作平台，支持情感分析、因果图谱、角色关系网络等创新功能',
        },
      ],
      link: [{ rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }],
    },
  },
})
