<script setup lang="ts">
import type { Rule, RuleAction, RuleMatcher, RulePayload, RuleTestResult } from '~/types/api'

type RuleForm = {
  matcher: string
  target_id?: number
  action: RuleAction
  response_template_id?: number
  priority: number
  delay_ms: number
  sequence_mode: boolean
  sequence_steps: number[]
  enabled: boolean
  note: string
}

type TestForm = {
  method: string
  path: string
  content_type: string
  query: string
  headers: string
  body: string
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
const deleteTarget = ref<Rule | null>(null)
const testOpen = ref(false)
const testPending = ref(false)
const testResult = ref<RuleTestResult | null>(null)
const testForm = reactive<TestForm>({
  method: 'GET',
  path: '/example',
  content_type: '',
  query: '{}',
  headers: '{}',
  body: '',
})
const form = reactive<RuleForm>({
  matcher: '{\n  "method": "POST",\n  "path": "/example"\n}',
  target_id: undefined,
  action: 'proxy',
  response_template_id: undefined,
  priority: 0,
  delay_ms: 0,
  sequence_mode: false,
  sequence_steps: [],
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
    delay_ms: 0,
    sequence_mode: false,
    sequence_steps: [],
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
    delay_ms: row.delay_ms,
    sequence_mode: row.sequence_mode,
    sequence_steps: (row.sequence_steps ?? []).map((step) => step.template_id),
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
    delay_ms: form.delay_ms,
    sequence_mode: form.sequence_mode,
    sequence_steps: form.sequence_steps.map((template_id) => ({ template_id })),
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

async function onImportChange(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (file) await importConfig(file, refresh)
}

function askDelete(row: Rule) {
  deleteTarget.value = row
  deleteOpen.value = true
}

async function remove() {
  if (!deleteTarget.value) return
  try {
    await api.deleteRule(deleteTarget.value.id)
    deleteOpen.value = false
    deleteTarget.value = null
    await refresh()
    toast.add({ title: t('app.deleted'), color: 'success' })
  } catch (cause) {
    toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
  }
}

function parseStringMap(text: string): Record<string, string> {
  const parsed = text.trim() ? JSON.parse(text) : {}
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, Array.isArray(value) ? String(value[0]) : String(value)]),
  )
}

function parseStringArrayMap(text: string): Record<string, string[]> {
  const parsed = text.trim() ? JSON.parse(text) : {}
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, Array.isArray(value) ? value.map(String) : [String(value)]]),
  )
}

function resetTestForm() {
  Object.assign(testForm, { method: 'GET', path: '/example', content_type: '', query: '{}', headers: '{}', body: '' })
  testResult.value = null
}

async function runTest() {
  testPending.value = true
  testResult.value = null
  try {
    const query = parseStringArrayMap(testForm.query)
    const headers = parseStringMap(testForm.headers)
    testResult.value = await api.testRule({
      method: testForm.method,
      path: testForm.path,
      query,
      headers,
      content_type: testForm.content_type || undefined,
      body: testForm.body || undefined,
    })
  } catch (cause) {
    toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
  } finally {
    testPending.value = false
  }
}
</script>

<template>
  <UPage>
    <UPageHeader :title="t('app.rules')">
      <template #links>
        <UButton icon="i-mdi-import" color="neutral" variant="ghost" :label="t('app.import')" @click="fileInput?.click()" />
        <UButton icon="i-mdi-export" color="neutral" variant="ghost" :label="t('app.export')" @click="exportConfig" />
        <UButton icon="i-mdi-flask-outline" color="neutral" variant="ghost" :label="t('app.testRule')" @click="resetTestForm(); testOpen = true" />
        <UButton icon="i-mdi-plus" :label="t('app.create')" @click="resetForm(); open = true" />
      </template>
    </UPageHeader>
    <input ref="fileInput" type="file" accept="application/json" class="hidden" @change="onImportChange">
    <UAlert v-if="error" color="error" :title="t('app.error')" class="mt-6" />
    <UTable :data="data?.items ?? []" :columns="columns" :loading="pending" class="mt-6">
      <template #matcher-cell="{ row }"><code class="text-xs">{{ matcherLabel(row.original.matcher) }}</code></template>
      <template #action-cell="{ row }"><UBadge color="neutral" variant="subtle">{{ row.original.action }}</UBadge></template>
      <template #enabled-cell="{ row }"><StatusBadge :enabled="row.original.enabled" /></template>
      <template #actions-cell="{ row }"><UButton icon="i-mdi-pencil" color="neutral" variant="ghost" :aria-label="t('app.edit')" @click="edit(row.original)" /><UButton icon="i-mdi-delete" color="error" variant="ghost" :aria-label="t('app.delete')" @click="askDelete(row.original)" /></template>
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
            <UFormField :label="t('app.fields.delay')" name="delay_ms"><UInput v-model.number="form.delay_ms" type="number" /></UFormField>
            <UFormField :label="t('app.fields.target')" name="target_id"><USelect v-model="form.target_id" :items="targetOptions" /></UFormField>
            <UFormField :label="t('app.fields.responseTemplate')" name="response_template_id"><USelect v-model="form.response_template_id" :items="templateOptions" /></UFormField>
          </div>
          <p v-if="form.action === 'proxy' && form.response_template_id" class="text-xs text-(--ui-text-muted)">{{ t('app.proxyTransformHint') }}</p>
          <UFormField :label="t('app.fields.note')" name="note"><UTextarea v-model="form.note" /></UFormField>
          <USwitch v-model="form.enabled" :label="t('app.enabled')" />
          <div class="flex items-center justify-between rounded-lg border border-(--ui-border) px-4 py-3">
            <USwitch v-model="form.sequence_mode" :label="t('app.sequenceMode')" />
            <span class="text-xs text-(--ui-text-muted)">{{ t('app.sequenceModeHint') }}</span>
          </div>
          <div v-if="form.sequence_mode" class="space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium">{{ t('app.sequenceSteps') }}</span>
              <UButton icon="i-mdi-plus" color="neutral" variant="ghost" size="xs" :label="t('app.addStep')" @click="form.sequence_steps.push(0)" />
            </div>
            <div v-for="(_, index) in form.sequence_steps" :key="index" class="flex items-center gap-2">
              <span class="w-8 text-center text-sm text-(--ui-text-muted)">{{ index + 1 }}</span>
              <USelect v-model="form.sequence_steps[index]" :items="templateOptions" class="flex-1" />
              <UButton icon="i-mdi-delete" color="error" variant="ghost" :aria-label="t('app.delete')" @click="form.sequence_steps.splice(index, 1)" />
            </div>
          </div>
          <div class="flex justify-end gap-2"><UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="open = false" /><UButton type="submit" :label="t('app.save')" /></div>
        </UForm>
      </template>
    </UModal>

    <ConfirmDialog
      v-model:open="deleteOpen"
      :title="t('app.deleteConfirm')"
      :description="deleteTarget ? t('app.deleteRuleConfirm', { matcher: matcherLabel(deleteTarget.matcher) }) : ''"
      @confirm="remove"
    />

    <UModal v-model:open="testOpen" :title="t('app.testRule')" :ui="{ content: 'sm:max-w-4xl' }">
      <template #body>
        <div class="space-y-4">
          <div class="grid gap-4 sm:grid-cols-3">
            <UFormField :label="t('app.fields.method')"><USelect v-model="testForm.method" :items="['GET', 'POST', 'PUT', 'DELETE', 'PATCH']" /></UFormField>
            <UFormField :label="t('app.fields.path')"><UInput v-model="testForm.path" /></UFormField>
            <UFormField :label="t('app.fields.contentType')"><UInput v-model="testForm.content_type" placeholder="application/json" /></UFormField>
          </div>
          <div class="grid gap-4 sm:grid-cols-2">
            <UFormField :label="t('app.fields.query')"><UTextarea v-model="testForm.query" :rows="3" class="font-mono text-xs" /></UFormField>
            <UFormField :label="t('app.fields.headers')"><UTextarea v-model="testForm.headers" :rows="3" class="font-mono text-xs" /></UFormField>
          </div>
          <UFormField :label="t('app.fields.request')"><UTextarea v-model="testForm.body" :rows="6" class="font-mono text-xs" /></UFormField>

          <UAlert v-if="testResult && !testResult.matched" color="warning" :title="t('app.testNoMatch')" class="mt-4" />
          <template v-else-if="testResult">
            <UCard class="mt-4" :ui="{ body: 'p-4' }">
              <div class="flex flex-wrap items-center gap-2">
                <UBadge color="primary" variant="subtle">#{{ testResult.rule_id }}</UBadge>
                <UBadge color="neutral" variant="subtle">{{ testResult.action }}</UBadge>
                <span class="text-sm text-(--ui-text-muted)">{{ t('app.fields.priority') }}: {{ testResult.priority }}</span>
                <span v-if="testResult.target_name" class="text-sm">{{ t('app.testWillProxy', { name: testResult.target_name }) }}</span>
              </div>
              <template v-if="testResult.rendered">
                <div class="mt-4 flex flex-wrap items-center gap-2 text-sm">
                  <span class="font-medium">{{ testResult.rendered.status_code }}</span>
                  <span class="text-(--ui-text-muted)">{{ testResult.rendered.content_type }}</span>
                </div>
                <pre class="mt-2 max-h-72 overflow-auto rounded-lg bg-(--ui-bg-elevated) p-3 font-mono text-xs">{{ testResult.rendered.raw_body }}</pre>
              </template>
            </UCard>
          </template>

          <div class="flex justify-end gap-2">
            <UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="testOpen = false" />
            <UButton icon="i-mdi-flask-outline" :loading="testPending" :label="t('app.runTest')" @click="runTest" />
          </div>
        </div>
      </template>
    </UModal>
  </UPage>
</template>
