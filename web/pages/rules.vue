<script setup lang="ts">
import type { Rule, RulePayload } from '~/types/api'

type RuleForm = Omit<RulePayload, 'note' | 'target_id' | 'response_template_id'> & {
  note: string
  target_id?: number
  response_template_id?: number
}

const { t } = useI18n()
const api = useApi()
const toast = useToast()
const page = ref(1)
const pageSize = 20
const open = ref(false)
const editingId = ref<number | null>(null)
const form = reactive<RuleForm>({
  service_code: '',
  message_type: '',
  message_code: '',
  target_id: undefined,
  mode: 'passthrough',
  response_template_id: undefined,
  priority: 0,
  enabled: true,
  note: '',
})

const [{ data, pending, error, refresh }, { data: targets }, { data: templates }] = await Promise.all([
  useAsyncData('rules', () => api.listRules({ offset: (page.value - 1) * pageSize, limit: pageSize }), { watch: [page] }),
  useAsyncData('rule-target-options', () => api.listTargets({ limit: 100 })),
  useAsyncData('rule-template-options', () => api.listTemplates({ limit: 100 })),
])

const columns = computed(() => [
  { accessorKey: 'service_code', header: t('app.fields.service') },
  { accessorKey: 'message_type', header: t('app.fields.messageType') },
  { accessorKey: 'message_code', header: t('app.fields.messageCode') },
  { accessorKey: 'mode', header: t('app.fields.mode') },
  { accessorKey: 'priority', header: t('app.fields.priority') },
  { accessorKey: 'enabled', header: t('app.fields.status') },
  { id: 'actions', header: '' },
])
const targetOptions = computed(() => (targets.value?.items ?? []).map((item) => ({ label: item.name, value: item.id })))
const templateOptions = computed(() => (templates.value?.items ?? []).map((item) => ({ label: item.name, value: item.id })))
const modeOptions = computed(() => [
  { label: t('app.modes.passthrough'), value: 'passthrough' },
  { label: t('app.modes.mock'), value: 'mock' },
])

function resetForm() {
  Object.assign(form, {
    service_code: '', message_type: '', message_code: '', target_id: undefined,
    mode: 'passthrough', response_template_id: undefined, priority: 0, enabled: true, note: '',
  })
  editingId.value = null
}

function edit(row: Rule) {
  Object.assign(form, {
    service_code: row.service_code,
    message_type: row.message_type,
    message_code: row.message_code,
    target_id: row.target_id ?? undefined,
    mode: row.mode,
    response_template_id: row.response_template_id ?? undefined,
    priority: row.priority,
    enabled: row.enabled,
    note: row.note ?? '',
  })
  editingId.value = row.id
  open.value = true
}

async function save() {
  try {
    if (editingId.value) await api.updateRule(editingId.value, form)
    else await api.createRule(form)
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
    <UPageHeader :title="t('app.rules')">
      <template #right><UButton icon="i-mdi-plus" :label="t('app.create')" @click="resetForm(); open = true" /></template>
    </UPageHeader>
    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <UTable :data="data?.items ?? []" :columns="columns" :loading="pending" class="mt-6">
      <template #mode-cell="{ row }"><UBadge color="neutral" variant="subtle">{{ row.original.mode }}</UBadge></template>
      <template #enabled-cell="{ row }"><StatusBadge :enabled="row.original.enabled" /></template>
      <template #actions-cell="{ row }"><UButton icon="i-mdi-pencil" color="neutral" variant="ghost" :aria-label="t('app.edit')" @click="edit(row.original)" /></template>
    </UTable>
    <div class="mt-4 flex justify-end"><UPagination v-model:page="page" :page-count="pageSize" :total="data?.total ?? 0" /></div>

    <UModal v-model:open="open" :title="editingId ? t('app.edit') : t('app.create')">
      <template #body>
        <UForm :state="form" class="space-y-4" @submit="save">
          <div class="grid gap-4 sm:grid-cols-3">
            <UFormField :label="t('app.fields.service')" name="service_code" required><UInput v-model="form.service_code" /></UFormField>
            <UFormField :label="t('app.fields.messageType')" name="message_type" required><UInput v-model="form.message_type" /></UFormField>
            <UFormField :label="t('app.fields.messageCode')" name="message_code" required><UInput v-model="form.message_code" /></UFormField>
          </div>
          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField :label="t('app.fields.mode')" name="mode" required><USelect v-model="form.mode" :items="modeOptions" /></UFormField>
            <UFormField :label="t('app.fields.priority')" name="priority"><UInput v-model.number="form.priority" type="number" /></UFormField>
            <UFormField :label="t('app.fields.target')" name="target_id"><USelect v-model="form.target_id" :items="targetOptions" /></UFormField>
            <UFormField :label="t('app.fields.responseTemplate')" name="response_template_id"><USelect v-model="form.response_template_id" :items="templateOptions" /></UFormField>
          </div>
          <UFormField :label="t('app.fields.note')" name="note"><UTextarea v-model="form.note" /></UFormField>
          <USwitch v-model="form.enabled" :label="t('app.enabled')" />
          <div class="flex justify-end gap-2"><UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="open = false" /><UButton type="submit" :label="t('app.save')" /></div>
        </UForm>
      </template>
    </UModal>
  </UPage>
</template>
