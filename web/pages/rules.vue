<script setup lang="ts">
import type { Rule, RuleAction, RuleMatcher, RulePayload } from '~/types/api'

type RuleForm = {
  matcher: string
  target_id?: number
  action: RuleAction
  response_template_id?: number
  priority: number
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
const form = reactive<RuleForm>({
  matcher: '{\n  "method": "POST",\n  "path": "/example"\n}',
  target_id: undefined,
  action: 'proxy',
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
  { accessorKey: 'matcher', header: t('app.fields.matcher') },
  { accessorKey: 'action', header: t('app.fields.action') },
  { accessorKey: 'priority', header: t('app.fields.priority') },
  { accessorKey: 'enabled', header: t('app.fields.status') },
  { id: 'actions', header: '' },
])
const targetOptions = computed(() => (targets.value?.items ?? []).map((item) => ({ label: item.name, value: item.id })))
const templateOptions = computed(() => (templates.value?.items ?? []).map((item) => ({ label: item.name, value: item.id })))
const actionOptions = computed(() => [
  { label: t('app.actions.proxy'), value: 'proxy' },
  { label: t('app.actions.static'), value: 'static' },
])

function resetForm() {
  Object.assign(form, {
    matcher: '{\n  "method": "POST",\n  "path": "/example"\n}',
    target_id: undefined,
    action: 'proxy',
    response_template_id: undefined,
    priority: 0,
    enabled: true,
    note: '',
  })
  editingId.value = null
}

function edit(row: Rule) {
  Object.assign(form, {
    matcher: JSON.stringify(row.matcher, null, 2),
    target_id: row.target_id ?? undefined,
    action: row.action,
    response_template_id: row.response_template_id ?? undefined,
    priority: row.priority,
    enabled: row.enabled,
    note: row.note ?? '',
  })
  editingId.value = row.id
  open.value = true
}

function payload(): RulePayload {
  return {
    matcher: JSON.parse(form.matcher) as RuleMatcher,
    target_id: form.target_id,
    action: form.action,
    response_template_id: form.response_template_id,
    priority: form.priority,
    enabled: form.enabled,
    note: form.note,
  }
}

function matcherLabel(matcher: RuleMatcher) {
  return [matcher.method, matcher.path ?? matcher.path_pattern].filter(Boolean).join(' ') || JSON.stringify(matcher)
}

async function save() {
  try {
    const body = payload()
    if (editingId.value) await api.updateRule(editingId.value, body)
    else await api.createRule(body)
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
      <template #matcher-cell="{ row }"><code class="text-xs">{{ matcherLabel(row.original.matcher) }}</code></template>
      <template #action-cell="{ row }"><UBadge color="neutral" variant="subtle">{{ row.original.action }}</UBadge></template>
      <template #enabled-cell="{ row }"><StatusBadge :enabled="row.original.enabled" /></template>
      <template #actions-cell="{ row }"><UButton icon="i-mdi-pencil" color="neutral" variant="ghost" :aria-label="t('app.edit')" @click="edit(row.original)" /></template>
    </UTable>
    <div class="mt-4 flex justify-end"><UPagination v-model:page="page" :page-count="pageSize" :total="data?.total ?? 0" /></div>

    <UModal v-model:open="open" :title="editingId ? t('app.edit') : t('app.create')" :ui="{ content: 'sm:max-w-3xl' }">
      <template #body>
        <UForm :state="form" class="space-y-4" @submit="save">
          <UFormField :label="t('app.fields.matcher')" name="matcher" required>
            <UTextarea v-model="form.matcher" :rows="12" class="font-mono" />
          </UFormField>
          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField :label="t('app.fields.action')" name="action" required><USelect v-model="form.action" :items="actionOptions" /></UFormField>
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
