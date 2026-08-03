<script setup lang="ts">
const { t } = useI18n()
const route = useRoute()
const api = useApi()
const toast = useToast()
const id = computed(() => Number(route.params.id))
const deleteOpen = ref(false)
const { data, pending, error } = await useAsyncData(`log-${id.value}`, () => api.getLog(id.value))

function formatDate(value?: string | null) { return value ? new Date(value).toLocaleString() : '-' }
async function remove() {
  try {
    await api.deleteLog(id.value)
    await navigateTo('/logs')
    toast.add({ title: t('app.deleted'), color: 'success' })
  } catch (cause) {
    toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
  }
}
</script>

<template>
  <UPage>
    <UPageHeader :title="`${t('app.logs')} #${id}`">
      <template #right>
        <UButton to="/logs" icon="i-mdi-arrow-left" color="neutral" variant="ghost" />
        <UButton icon="i-mdi-delete" color="error" variant="ghost" :aria-label="t('app.delete')" @click="deleteOpen = true" />
      </template>
    </UPageHeader>
    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <USkeleton v-else-if="pending" class="mt-6 h-32" />
    <template v-else-if="data">
      <UPageGrid class="mt-6">
        <UPageCard :title="t('app.fields.request')" :description="`${data.method} ${data.path}`" />
        <UPageCard :title="t('app.fields.action')" :description="`${data.action ?? '-'} / ${data.http_status_code ?? '-'}`" />
        <UPageCard :title="t('app.fields.bodyFormat')" :description="data.body_format" />
        <UPageCard :title="t('app.fields.latency')" :description="`${data.latency_ms ?? '-'} ms`" />
        <UPageCard :title="t('app.fields.time')" :description="formatDate(data.occurred_at)" />
      </UPageGrid>
      <UAlert v-if="data.error_message" color="error" :title="t('app.fields.errorMessage')" :description="data.error_message" class="mt-6" />
      <div class="mt-6 grid gap-6 lg:grid-cols-2">
        <UCard v-for="snapshot in data.snapshots" :key="snapshot.id" :ui="{ body: 'p-0' }">
          <template #header><div class="font-medium">{{ snapshot.kind }}</div></template>
          <pre class="max-h-[36rem] overflow-auto p-4 text-xs">{{ snapshot.raw_body }}</pre>
        </UCard>
      </div>
    </template>

    <ConfirmDialog v-model:open="deleteOpen" :title="t('app.deleteConfirm')" :description="t('app.deleteLogConfirm')" @confirm="remove" />
  </UPage>
</template>
