export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  future: {
    compatibilityVersion: 4,
  },
  srcDir: '.',
  ssr: false,
  devtools: { enabled: false },
  modules: ['@nuxt/ui', '@nuxtjs/i18n', '@nuxt/eslint'],
  ui: { fonts: false },
  css: ['~/assets/css/main.css'],
  runtimeConfig: {
    public: {
      apiBase: process.env.NUXT_PUBLIC_API_BASE || '/',
    },
  },
  icon: {
    serverBundle: {
      collections: ['mdi'],
    },
  },
  i18n: {
    defaultLocale: 'zh-CN',
    strategy: 'no_prefix',
    locales: [
      { code: 'zh-CN', file: 'zh-CN.ts', name: '简体中文' },
      { code: 'en', file: 'en.ts', name: 'English' },
    ],
    langDir: 'locales',
  },
  app: {
    head: {
      title: 'fakESB',
      meta: [{ name: 'description', content: 'Configurable ESB test gateway' }],
    },
  },
})
