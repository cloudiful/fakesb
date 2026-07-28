<script setup lang="ts">
import type { ResponseTemplate, TemplatePayload } from '~/types/api'

type TemplateForm = Omit<TemplatePayload, 'note'> & {
  enabled: boolean
  note: string
}

const { t } = useI18n()
const api = useApi()
const toast = useToast()
const page = ref(1)
const pageSize = 20
const open = ref(false)
const editingId = ref<number | null>(null)
const form = reactive<TemplateForm>({
  name: '',
  content_type: 'application/xml',
  raw_template: '',
  format: 'xml',
  enabled: true,
  note: '',
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
  { accessorKey: 'enabled', header: t('app.fields.status') },
  { id: 'actions', header: '' },
])

function resetForm() {
  Object.assign(form, { name: '', content_type: 'application/xml', raw_template: '', format: 'xml', enabled: true, note: '' })
  editingId.value = null
}

function edit(row: ResponseTemplate) {
  Object.assign(form, {
    name: row.name,
    content_type: row.content_type,
    raw_template: row.raw_template,
    format: row.format,
    enabled: row.enabled,
    note: row.note ?? '',
  })
  editingId.value = row.id
  open.value = true
}

async function save() {
  try {
    if (editingId.value) await api.updateTemplate(editingId.value, form)
    else await api.createTemplate(form)
    open.value = false
    resetForm()
    await refresh()
    toast.add({ title: t('app.save'), color: 'success' })
  } catch (cause) {
    toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
  }
}
</script>

<template>
  <UPage>
    <UPageHeader :title="t('app.templates')">
      <template #right><UButton icon="i-mdi-plus" :label="t('app.create')" @click="resetForm(); open = true" /></template>
    </UPageHeader>
    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <UTable :data="data?.items ?? []" :columns="columns" :loading="pending" class="mt-6">
      <template #enabled-cell="{ row }"><StatusBadge :enabled="row.original.enabled" /></template>
      <template #actions-cell="{ row }"><UButton icon="i-mdi-pencil" color="neutral" variant="ghost" :aria-label="t('app.edit')" @click="edit(row.original)" /></template>
    </UTable>
    <div class="mt-4 flex justify-end"><UPagination v-model:page="page" :page-count="pageSize" :total="data?.total ?? 0" /></div>

    <UModal v-model:open="open" :title="editingId ? t('app.edit') : t('app.create')" :ui="{ content: 'sm:max-w-4xl' }">
      <template #body>
        <UForm :state="form" class="space-y-4" @submit="save">
          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField :label="t('app.fields.name')" name="name" required><UInput v-model="form.name" /></UFormField>
            <UFormField :label="t('app.fields.contentType')" name="content_type" required><UInput v-model="form.content_type" /></UFormField>
          </div>
          <UFormField :label="t('app.fields.responseXml')" name="raw_template" required><UTextarea v-model="form.raw_template" :rows="16" class="font-mono" /></UFormField>
          <UFormField :label="t('app.fields.note')" name="note"><UTextarea v-model="form.note" /></UFormField>
          <USwitch v-model="form.enabled" :label="t('app.enabled')" />
          <div class="flex justify-end gap-2"><UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="open = false" /><UButton type="submit" :label="t('app.save')" /></div>
        </UForm>
      </template>
    </UModal>
  </UPage>
</template>
