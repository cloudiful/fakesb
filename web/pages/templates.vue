<script setup lang="ts">
import type { ResponseTemplate, TemplatePayload } from '~/types/api'

type TemplateForm = {
  name: string
  content_type: string
  raw_template: string
  format: string
  status_code: number
  headers: string
  enabled: boolean
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
const deleteTarget = ref<ResponseTemplate | null>(null)
const form = reactive<TemplateForm>({
  name: '', content_type: 'application/json', raw_template: '', format: 'json', status_code: 200, headers: '{}', enabled: true, note: '',
})

const { data, pending, error, refresh } = await useAsyncData(
  'templates',
  () => api.listTemplates({ offset: (page.value - 1) * pageSize, limit: pageSize }),
  { watch: [page] },
)

const columns = computed(() => [
  { accessorKey: 'name', header: t('app.fields.name') },
  { accessorKey: 'content_type', header: t('app.fields.contentType') },
  { accessorKey: 'format', header: t('app.fields.format') },
  { accessorKey: 'status_code', header: t('app.fields.statusCode') },
  { accessorKey: 'enabled', header: t('app.fields.status') },
  { id: 'actions', header: '' },
])

function resetForm() {
  Object.assign(form, { name: '', content_type: 'application/json', raw_template: '', format: 'json', status_code: 200, headers: '{}', enabled: true, note: '' })
  editingId.value = null
}

function edit(row: ResponseTemplate) {
  Object.assign(form, {
    name: row.name, content_type: row.content_type, raw_template: row.raw_template, format: row.format,
    status_code: row.status_code, headers: JSON.stringify(row.headers ?? {}, null, 2), enabled: row.enabled, note: row.note ?? '',
  })
  editingId.value = row.id
  open.value = true
}

function payload(): TemplatePayload {
  return { ...form, headers: JSON.parse(form.headers) }
}

async function save() {
  try {
    const body = payload()
    if (editingId.value) await api.updateTemplate(editingId.value, body)
    else await api.createTemplate(body)
    open.value = false
    resetForm()
    await refresh()
    toast.add({ title: t('app.save'), color: 'success' })
  } catch (cause) {
    toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
  }
}

async function onImportChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (file) await importConfig(file, refresh)
}

function askDelete(row: ResponseTemplate) {
  deleteTarget.value = row
  deleteOpen.value = true
}

async function remove() {
  if (!deleteTarget.value) return
  try {
    await api.deleteTemplate(deleteTarget.value.id)
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
    <UPageHeader :title="t('app.templates')">
      <template #links>
        <UButton icon="i-mdi-import" color="neutral" variant="ghost" :label="t('app.import')" @click="fileInput?.click()" />
        <UButton icon="i-mdi-export" color="neutral" variant="ghost" :label="t('app.export')" @click="exportConfig" />
        <UButton icon="i-mdi-plus" :label="t('app.create')" @click="resetForm(); open = true" />
      </template>
    </UPageHeader>
    <input ref="fileInput" type="file" accept="application/json" class="hidden" @change="onImportChange">
    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <UTable :data="data?.items ?? []" :columns="columns" :loading="pending" class="mt-6">
      <template #enabled-cell="{ row }"><StatusBadge :enabled="row.original.enabled" /></template>
      <template #actions-cell="{ row }"><UButton icon="i-mdi-pencil" color="neutral" variant="ghost" :aria-label="t('app.edit')" @click="edit(row.original)" /><UButton icon="i-mdi-delete" color="error" variant="ghost" :aria-label="t('app.delete')" @click="askDelete(row.original)" /></template>
    </UTable>
    <div class="mt-4 flex justify-end"><UPagination v-model:page="page" :page-count="pageSize" :total="data?.total ?? 0" /></div>

    <UModal v-model:open="open" :title="editingId ? t('app.edit') : t('app.create')" :ui="{ content: 'sm:max-w-4xl' }">
      <template #body>
        <UForm :state="form" class="space-y-4" @submit="save">
          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField :label="t('app.fields.name')" name="name" required><UInput v-model="form.name" /></UFormField>
            <UFormField :label="t('app.fields.contentType')" name="content_type" required><UInput v-model="form.content_type" /></UFormField>
            <UFormField :label="t('app.fields.format')" name="format" required><USelect v-model="form.format" :items="['json', 'xml', 'text']" /></UFormField>
            <UFormField :label="t('app.fields.statusCode')" name="status_code" required><UInput v-model.number="form.status_code" type="number" /></UFormField>
          </div>
          <UFormField :label="t('app.fields.responseBody')" name="raw_template" required><UTextarea v-model="form.raw_template" :rows="14" class="font-mono" /></UFormField>
          <UFormField :label="t('app.fields.responseHeaders')" name="headers"><UTextarea v-model="form.headers" :rows="4" class="font-mono" /></UFormField>
          <UFormField :label="t('app.fields.note')" name="note"><UTextarea v-model="form.note" /></UFormField>
          <USwitch v-model="form.enabled" :label="t('app.enabled')" />
          <div class="flex justify-end gap-2"><UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="open = false" /><UButton type="submit" :label="t('app.save')" /></div>
        </UForm>
      </template>
    </UModal>

    <ConfirmDialog
      v-model:open="deleteOpen"
      :title="t('app.deleteConfirm')"
      :description="deleteTarget ? t('app.deleteTemplateConfirm', { name: deleteTarget.name }) : ''"
      @confirm="remove"
    />
  </UPage>
</template>
