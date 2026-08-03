<script setup lang="ts">
import type { Target, TargetPayload } from '~/types/api'

type TargetForm = Omit<TargetPayload, 'note'> & {
  enabled: boolean
  timeout_ms: number
  note: string
}

const { t } = useI18n()
const api = useApi()
const toast = useToast()
const { exportConfig, importConfig } = useConfigIO()
const fileInput = ref<HTMLInputElement | null>(null)
const page = ref(1)
const pageSize = 20
const open = ref(false)
const editingId = ref<number | null>(null)
const deleteOpen = ref(false)
const deleteTarget = ref<Target | null>(null)
const form = reactive<TargetForm>({
  name: '',
  base_url: '',
  enabled: true,
  timeout_ms: 10000,
  note: '',
})

const { data, pending, error, refresh } = await useAsyncData(
  'targets',
  () => api.listTargets({ offset: (page.value - 1) * pageSize, limit: pageSize }),
  { watch: [page] },
)

const columns = computed(() => [
  { accessorKey: 'name', header: t('app.fields.name') },
  { accessorKey: 'base_url', header: t('app.fields.url') },
  { accessorKey: 'timeout_ms', header: t('app.fields.timeout') },
  { accessorKey: 'enabled', header: t('app.fields.status') },
  { id: 'actions', header: '' },
])

function resetForm() {
  Object.assign(form, { name: '', base_url: '', enabled: true, timeout_ms: 10000, note: '' })
  editingId.value = null
}

function edit(row: Target) {
  Object.assign(form, {
    name: row.name,
    base_url: row.base_url,
    enabled: row.enabled,
    timeout_ms: row.timeout_ms,
    note: row.note ?? '',
  })
  editingId.value = row.id
  open.value = true
}

async function save() {
  try {
    if (editingId.value) await api.updateTarget(editingId.value, form)
    else await api.createTarget(form)
    open.value = false
    resetForm()
    await refresh()
    toast.add({ title: t('app.save'), color: 'success' })
  } catch (cause) {
    toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
  }
}

function askDelete(row: Target) {
  deleteTarget.value = row
  deleteOpen.value = true
}

async function onImportChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (file) await importConfig(file, refresh)
}

async function remove() {
  if (!deleteTarget.value) return
  try {
    await api.deleteTarget(deleteTarget.value.id)
    deleteOpen.value = false
    deleteTarget.value = null
    await refresh()
    toast.add({ title: t('app.deleted'), color: 'success' })
  } catch (cause) {
    toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
  }
}
</script>

<template>
  <UPage>
    <UPageHeader :title="t('app.targets')">
      <template #right>
        <UButton icon="i-mdi-import" color="neutral" variant="ghost" :label="t('app.import')" @click="fileInput?.click()" />
        <UButton icon="i-mdi-export" color="neutral" variant="ghost" :label="t('app.export')" @click="exportConfig" />
        <UButton icon="i-mdi-plus" :label="t('app.create')" @click="resetForm(); open = true" />
      </template>
    </UPageHeader>
    <input ref="fileInput" type="file" accept="application/json" class="hidden" @change="onImportChange">

    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <UTable :data="data?.items ?? []" :columns="columns" :loading="pending" class="mt-6">
      <template #enabled-cell="{ row }">
        <StatusBadge :enabled="row.original.enabled" />
      </template>
      <template #actions-cell="{ row }">
        <UButton icon="i-mdi-pencil" color="neutral" variant="ghost" :aria-label="t('app.edit')" @click="edit(row.original)" />
        <UButton icon="i-mdi-delete" color="error" variant="ghost" :aria-label="t('app.delete')" @click="askDelete(row.original)" />
      </template>
    </UTable>
    <div class="mt-4 flex justify-end">
      <UPagination v-model:page="page" :page-count="pageSize" :total="data?.total ?? 0" />
    </div>

    <UModal v-model:open="open" :title="editingId ? t('app.edit') : t('app.create')">
      <template #body>
        <UForm :state="form" class="space-y-4" @submit="save">
          <UFormField :label="t('app.fields.name')" name="name" required><UInput v-model="form.name" /></UFormField>
          <UFormField :label="t('app.fields.url')" name="base_url" required><UInput v-model="form.base_url" /></UFormField>
          <UFormField :label="t('app.fields.timeout')" name="timeout_ms" required><UInput v-model.number="form.timeout_ms" type="number" /></UFormField>
          <UFormField :label="t('app.fields.note')" name="note"><UTextarea v-model="form.note" /></UFormField>
          <USwitch v-model="form.enabled" :label="t('app.enabled')" />
          <div class="flex justify-end gap-2"><UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="open = false" /><UButton type="submit" :label="t('app.save')" /></div>
        </UForm>
      </template>
    </UModal>

    <ConfirmDialog
      v-model:open="deleteOpen"
      :title="t('app.deleteConfirm')"
      :description="deleteTarget ? t('app.deleteTargetConfirm', { name: deleteTarget.name }) : ''"
      @confirm="remove"
    />
  </UPage>
</template>
