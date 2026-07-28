<script setup lang="ts">
import type { RuleMode } from '~/types/api'

const { t } = useI18n()
const api = useApi()
const page = ref(1)
const pageSize = 20
const filters = reactive<{
  service_code: string
  message_type: string
  message_code: string
  mode: '' | RuleMode
  ret_code: string
}>({ service_code: '', message_type: '', message_code: '', mode: '', ret_code: '' })
const query = computed(() => ({
  service_code: filters.service_code || undefined,
  message_type: filters.message_type || undefined,
  message_code: filters.message_code || undefined,
  mode: filters.mode || undefined,
  ret_code: filters.ret_code || undefined,
  offset: (page.value - 1) * pageSize,
  limit: pageSize,
}))

const { data, pending, error, refresh } = await useAsyncData(
  'logs',
  () => api.listLogs(query.value),
  { watch: [page, query] },
)

const columns = computed(() => [
  { accessorKey: 'id', header: t('app.fields.id') },
  { accessorKey: 'occurred_at', header: t('app.fields.time') },
  { accessorKey: 'service_code', header: t('app.fields.service') },
  { accessorKey: 'message_type', header: t('app.fields.messageType') },
  { accessorKey: 'message_code', header: t('app.fields.messageCode') },
  { accessorKey: 'mode', header: t('app.fields.mode') },
  { accessorKey: 'ret_code', header: t('app.fields.returnCode') },
  { accessorKey: 'latency_ms', header: t('app.fields.latency') },
])

function formatDate(value: string) {
  return new Date(value).toLocaleString()
}

function clearFilters() {
  Object.assign(filters, { service_code: '', message_type: '', message_code: '', mode: '', ret_code: '' })
  page.value = 1
}
</script>

<template>
  <UPage>
    <UPageHeader :title="t('app.logs')">
      <template #right><UButton icon="i-mdi-refresh" color="neutral" variant="ghost" :aria-label="t('app.refresh')" @click="refresh()" /></template>
    </UPageHeader>

    <UCard class="mt-6">
      <div class="grid gap-3 md:grid-cols-5">
        <UInput v-model="filters.service_code" :placeholder="t('app.fields.service')" />
        <UInput v-model="filters.message_type" :placeholder="t('app.fields.messageType')" />
        <UInput v-model="filters.message_code" :placeholder="t('app.fields.messageCode')" />
        <USelect v-model="filters.mode" :items="[{ label: t('app.modes.all'), value: '' }, { label: t('app.modes.passthrough'), value: 'passthrough' }, { label: t('app.modes.mock'), value: 'mock' }]" />
        <UInput v-model="filters.ret_code" :placeholder="t('app.fields.returnCode')" />
      </div>
      <div class="mt-4 flex justify-end"><UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="clearFilters" /></div>
    </UCard>

    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <UTable :data="data?.items ?? []" :columns="columns" :loading="pending" class="mt-6">
      <template #id-cell="{ row }"><NuxtLink :to="`/logs/${row.original.id}`" class="text-primary hover:underline">{{ row.original.id }}</NuxtLink></template>
      <template #occurred_at-cell="{ row }">{{ formatDate(row.original.occurred_at) }}</template>
      <template #mode-cell="{ row }"><UBadge color="neutral" variant="subtle">{{ row.original.mode || '-' }}</UBadge></template>
      <template #latency_ms-cell="{ row }">{{ row.original.latency_ms ?? '-' }} ms</template>
    </UTable>
    <div class="mt-4 flex justify-end"><UPagination v-model:page="page" :page-count="pageSize" :total="data?.total ?? 0" /></div>
  </UPage>
</template>
