<script setup lang="ts">
const { t } = useI18n()
const api = useApi()
const toast = useToast()

const requests = await Promise.all([
  useAsyncData('dashboard-targets', () => api.listTargets({ limit: 1 })),
  useAsyncData('dashboard-rules', () => api.listRules({ limit: 1 })),
  useAsyncData('dashboard-templates', () => api.listTemplates({ limit: 1 })),
  useAsyncData('dashboard-logs', () => api.listLogs({ limit: 1 })),
])
const [{ data: targets }, { data: rules }, { data: templates }, { data: logs }] = requests
const pending = computed(() => requests.some((request) => request.pending.value))
const hasError = computed(() => requests.some((request) => request.error.value))

const cards = computed(() => [
  { label: t('app.targets'), value: targets.value?.total ?? 0, to: '/targets', icon: 'i-mdi-server-network' },
  { label: t('app.rules'), value: rules.value?.total ?? 0, to: '/rules', icon: 'i-mdi-source-branch' },
  { label: t('app.templates'), value: templates.value?.total ?? 0, to: '/templates', icon: 'i-mdi-file-code-outline' },
  { label: t('app.logs'), value: logs.value?.total ?? 0, to: '/logs', icon: 'i-mdi-history' },
])

onErrorCaptured((error) => {
  toast.add({ title: t('app.error'), description: String(error), color: 'error' })
  return false
})
</script>

<template>
  <UPage>
    <UPageHeader :title="t('app.dashboard')" />
    <UAlert v-if="hasError" color="error" :title="t('app.error')" class="mt-6" />
    <UPageGrid v-else-if="pending" class="mt-6">
      <USkeleton v-for="card in 4" :key="card" class="h-32" />
    </UPageGrid>
    <UPageGrid v-else class="mt-6">
      <UPageCard
        v-for="card in cards"
        :key="card.to"
        :to="card.to"
        :title="card.label"
        :description="String(card.value)"
        :icon="card.icon"
        spotlight
      />
    </UPageGrid>
  </UPage>
</template>
