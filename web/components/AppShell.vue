<script setup lang="ts">
const { t, locale, setLocale } = useI18n()

const navigation = computed(() => [
  { label: t('app.dashboard'), to: '/', icon: 'i-mdi-view-dashboard-outline' },
  { label: t('app.targets'), to: '/targets', icon: 'i-mdi-server-network' },
  { label: t('app.rules'), to: '/rules', icon: 'i-mdi-source-branch' },
  { label: t('app.templates'), to: '/templates', icon: 'i-mdi-file-code-outline' },
  { label: t('app.logs'), to: '/logs', icon: 'i-mdi-history' },
])

function toggleLocale() {
  setLocale(locale.value === 'zh-CN' ? 'en' : 'zh-CN')
}
</script>

<template>
  <UHeader :ui="{ root: 'bg-white/80 backdrop-blur dark:bg-gray-950/80' }">
    <template #left>
      <NuxtLink to="/" class="flex items-center gap-2 font-semibold text-gray-900 dark:text-white">
        <UIcon name="i-mdi-router-network" class="size-5 text-primary" />
        <span>{{ t('app.title') }}</span>
      </NuxtLink>
    </template>

    <UNavigationMenu :items="navigation" orientation="horizontal" />

    <template #right>
      <UButton
        icon="i-mdi-translate"
        color="neutral"
        variant="ghost"
        :aria-label="locale"
        @click="toggleLocale"
      />
    </template>
  </UHeader>

  <UMain>
    <UContainer class="py-6 sm:py-8">
      <slot />
    </UContainer>
  </UMain>
</template>
