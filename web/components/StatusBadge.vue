<script setup lang="ts">
defineProps<{ enabled: boolean }>()
const { t } = useI18n()
</script>

<template>
  <UBadge :color="enabled ? 'success' : 'neutral'" variant="subtle">
    {{ enabled ? t('app.enabled') : t('app.disabled') }}
  </UBadge>
</template>
