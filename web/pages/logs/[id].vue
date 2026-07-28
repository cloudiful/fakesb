<script setup lang="ts">
const { t } = useI18n()
const route = useRoute()
const api = useApi()
const id = computed(() => Number(route.params.id))
const { data, pending, error } = await useAsyncData(`log-${id.value}`, () => api.getLog(id.value))

function formatDate(value?: string | null) {
  return value ? new Date(value).toLocaleString() : '-'
}
</script>

<template>
  <UPage>
    <UPageHeader :title="`${t('app.logs')} #${id}`">
      <template #right><UButton to="/logs" icon="i-mdi-arrow-left" color="neutral" variant="ghost" /></template>
    </UPageHeader>
    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <USkeleton v-else-if="pending" class="mt-6 h-32" />
    <template v-else-if="data">
      <UPageGrid class="mt-6">
        <UPageCard :title="t('app.fields.service')" :description="`${data.service_code} / ${data.message_type} / ${data.message_code}`" />
        <UPageCard :title="t('app.fields.returnCode')" :description="`${data.http_status_code ?? '-'} / ${data.ret_code ?? '-'}`" />
        <UPageCard :title="t('app.fields.latency')" :description="`${data.latency_ms ?? '-'} ms`" />
        <UPageCard :title="t('app.fields.time')" :description="formatDate(data.occurred_at)" />
      </UPageGrid>
      <UAlert
        v-if="data.error_message"
        color="error"
        :title="t('app.fields.errorMessage')"
        :description="data.error_message"
        class="mt-6"
      />
      <div class="mt-6 grid gap-6 lg:grid-cols-2">
        <UCard v-for="snapshot in data.snapshots" :key="snapshot.id" :ui="{ body: 'p-0' }">
          <template #header><div class="font-medium">{{ snapshot.kind }}</div></template>
          <pre class="max-h-[36rem] overflow-auto p-4 text-xs">{{ snapshot.raw_body }}</pre>
        </UCard>
      </div>
    </template>
  </UPage>
</template>
