<script setup lang="ts">
import type { RuleAction } from '~/types/api'

const { t } = useI18n()
const api = useApi()
const page = ref(1)
const pageSize = 20
const filters = reactive<{ method: string; path: string; action: '' | RuleAction; status_code?: number }>({ method: '', path: '', action: '', status_code: undefined })
const query = computed(() => ({
  method: filters.method || undefined,
  path: filters.path || undefined,
  action: filters.action || undefined,
  status_code: filters.status_code,
  offset: (page.value - 1) * pageSize,
  limit: pageSize,
}))

const { data, pending, error, refresh } = await useAsyncData('logs', () => api.listLogs(query.value), { watch: [page, query] })
const columns = computed(() => [
  { accessorKey: 'id', header: t('app.fields.id') },
  { accessorKey: 'occurred_at', header: t('app.fields.time') },
  { accessorKey: 'method', header: t('app.fields.method') },
  { accessorKey: 'path', header: t('app.fields.path') },
  { accessorKey: 'action', header: t('app.fields.action') },
  { accessorKey: 'http_status_code', header: t('app.fields.statusCode') },
  { accessorKey: 'latency_ms', header: t('app.fields.latency') },
])

function formatDate(value: string) { return new Date(value).toLocaleString() }
function clearFilters() { Object.assign(filters, { method: '', path: '', action: '', status_code: undefined }); page.value = 1 }
</script>

<template>
  <UPage>
    <UPageHeader :title="t('app.logs')">
      <template #right><UButton icon="i-mdi-refresh" color="neutral" variant="ghost" :aria-label="t('app.refresh')" @click="refresh()" /></template>
    </UPageHeader>
    <UCard class="mt-6">
      <div class="grid gap-3 md:grid-cols-4">
        <UInput v-model="filters.method" :placeholder="t('app.fields.method')" />
        <UInput v-model="filters.path" :placeholder="t('app.fields.path')" />
        <USelect v-model="filters.action" :items="[{ label: t('app.actions.all'), value: '' }, { label: t('app.actions.proxy'), value: 'proxy' }, { label: t('app.actions.static'), value: 'static' }]" />
        <UInput v-model.number="filters.status_code" :placeholder="t('app.fields.statusCode')" type="number" />
      </div>
      <div class="mt-4 flex justify-end"><UButton color="neutral" variant="ghost" :label="t('app.clear')" @click="clearFilters" /></div>
    </UCard>
    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <UTable :data="data?.items ?? []" :columns="columns" :loading="pending" class="mt-6">
      <template #id-cell="{ row }"><NuxtLink :to="`/logs/${row.original.id}`" class="text-primary hover:underline">{{ row.original.id }}</NuxtLink></template>
      <template #occurred_at-cell="{ row }">{{ formatDate(row.original.occurred_at) }}</template>
      <template #action-cell="{ row }"><UBadge color="neutral" variant="subtle">{{ row.original.action || '-' }}</UBadge></template>
      <template #latency_ms-cell="{ row }">{{ row.original.latency_ms ?? '-' }} ms</template>
    </UTable>
    <div class="mt-4 flex justify-end"><UPagination v-model:page="page" :page-count="pageSize" :total="data?.total ?? 0" /></div>
  </UPage>
</template>
