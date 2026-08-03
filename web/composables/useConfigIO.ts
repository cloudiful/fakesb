export function useConfigIO() {
  const { t } = useI18n()
  const api = useApi()
  const toast = useToast()

  async function exportConfig() {
    try {
      const bundle = await api.exportConfig()
      const blob = new Blob([JSON.stringify(bundle, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = `fakesb-config-${new Date().toISOString().slice(0, 10)}.json`
      link.click()
      URL.revokeObjectURL(url)
    } catch (cause) {
      toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
    }
  }

  async function importConfig(file: File, afterImport?: () => Promise<void> | void) {
    try {
      const bundle = JSON.parse(await file.text())
      const summary = await api.importConfig(bundle)
      toast.add({
        title: t('app.imported', {
          targets: summary.targets_imported,
          templates: summary.templates_imported,
          rules: summary.rules_imported,
        }),
        color: 'success',
      })
      for (const warning of summary.warnings ?? []) {
        toast.add({ title: t('app.warning'), description: warning, color: 'warning' })
      }
      await afterImport?.()
    } catch (cause) {
      toast.add({ title: t('app.error'), description: String(cause), color: 'error' })
    }
  }

  return { exportConfig, importConfig }
}
