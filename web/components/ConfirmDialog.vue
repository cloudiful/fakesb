<script setup lang="ts">
const { t } = useI18n()

const props = defineProps<{ open: boolean; title: string; description?: string }>()
const emit = defineEmits<{ 'update:open': [value: boolean]; confirm: [] }>()

function confirm() {
  emit('update:open', false)
  emit('confirm')
}
</script>

<template>
  <UModal :open="props.open" :title="title" @update:open="emit('update:open', $event)">
    <template #body>
      <p class="text-sm text-(--ui-text-muted)">{{ description }}</p>
    </template>
    <template #footer>
      <div class="flex justify-end gap-2">
        <UButton color="neutral" variant="ghost" :label="t('app.cancel')" @click="emit('update:open', false)" />
        <UButton color="error" :label="t('app.delete')" @click="confirm" />
      </div>
    </template>
  </UModal>
</template>
