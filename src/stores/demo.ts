import { ref } from 'vue'
import { defineStore } from 'pinia'
import * as api from '@/services/tauri/demo'
import type { DemoListItem, DemoRecordingSettings, DemoReport, DemoRoot } from '@/types/demo'

export const useDemoStore = defineStore('demo', () => {
  const roots = ref<DemoRoot[]>([]); const items = ref<DemoListItem[]>([]); const total = ref(0); const report = ref<DemoReport | null>(null)
  const query = ref(''); const status = ref('all'); const page = ref(1); const pageSize = ref(25); const busy = ref(''); const error = ref(''); const settings = ref<DemoRecordingSettings | null>(null)
  const normalize = (value: unknown) => typeof value === 'string' ? value : value instanceof Error ? value.message : 'Demo 操作失败。'
  async function refresh() { busy.value = 'list'; error.value = ''; try { const [rootRows, demos] = await Promise.all([api.listDemoRoots(), api.listDemos(query.value, status.value, page.value, pageSize.value)]); roots.value = rootRows; items.value = demos.items; total.value = demos.total } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function addRoot(path: string) { await api.addDemoRoot(path); await refresh() }
  async function updateRoot(root: DemoRoot, enabled = root.enabled, depth = root.scanDepth) { await api.updateDemoRoot(root.id, enabled, depth); await refresh() }
  async function removeRoot(id: number) { await api.removeDemoRoot(id); await refresh() }
  async function scan() { if (busy.value === 'scan') return; busy.value = 'scan'; error.value = ''; try { await api.scanDemoRoots(); await refresh() } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function importFile(path: string) { busy.value = 'import'; error.value = ''; try { const result = await api.importDemoFile(path); await refresh(); if (result.status === 'done') await openReport(result.demoFileId) } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function openReport(id: number) { busy.value = 'report'; error.value = ''; try { report.value = await api.getDemoReport(id) } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function retry(id: number) { busy.value = `retry-${id}`; try { await api.retryDemoParse(id); await refresh(); await openReport(id) } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function loadSettings(rootPath: string) { if (!rootPath) return; try { settings.value = await api.getDemoSettings(rootPath) } catch { settings.value = null } }
  async function setRecording(rootPath: string, enabled: boolean) { busy.value = 'recording'; try { settings.value = await api.setDemoRecordingEnabled(rootPath, enabled) } catch (e) { error.value = normalize(e); throw e } finally { busy.value = '' } }
  return { roots, items, total, report, query, status, page, pageSize, busy, error, settings, refresh, addRoot, updateRoot, removeRoot, scan, importFile, openReport, retry, loadSettings, setRecording }
})
