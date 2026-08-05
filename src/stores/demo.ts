import { ref } from 'vue'
import { defineStore } from 'pinia'
import * as api from '@/services/tauri/demo'
import { dispatchToast } from '@/services/toast'
import type { DemoAnalysisJob, DemoListItem, DemoRecordingSettings, DemoReport, DemoRoot, DemoScanResult } from '@/types/demo'

export const useDemoStore = defineStore('demo', () => {
  const roots = ref<DemoRoot[]>([]); const items = ref<DemoListItem[]>([]); const total = ref(0); const report = ref<DemoReport | null>(null)
  const query = ref(''); const status = ref('all'); const page = ref(1); const pageSize = ref(25); const busy = ref(''); const error = ref(''); const settings = ref<DemoRecordingSettings | null>(null)
  const scanResult = ref<DemoScanResult | null>(null)
  const jobs = ref<DemoAnalysisJob[]>([])
  const rowBusy = ref<Record<number, 'play' | 'reveal'>>({})
  const normalize = (value: unknown) => typeof value === 'string' ? value : value instanceof Error ? value.message : 'Demo 操作失败。'
  async function refresh() { busy.value = 'list'; error.value = ''; try { const [rootRows, demos, jobRows] = await Promise.all([api.listDemoRoots(), api.listDemos(query.value, status.value, page.value, pageSize.value), api.listAnalysisJobs(false)]); roots.value = rootRows; items.value = demos.items; total.value = demos.total; jobs.value = jobRows } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function addRoot(path: string) { const root = await api.addDemoRoot(path); scanResult.value = await api.scanDemoRoots(root.id); await refresh(); announceScan(scanResult.value, true) }
  async function updateRoot(root: DemoRoot, enabled = root.enabled, depth = root.scanDepth) { await api.updateDemoRoot(root.id, enabled, depth); await refresh() }
  async function removeRoot(id: number) { await api.removeDemoRoot(id); await refresh() }
  function announceScan(result: DemoScanResult, added = false) { const prefix = added ? '目录已添加，' : '扫描完成，'; if (result.permissionErrors) dispatchToast({ tone: 'warn', title: added ? '目录已添加，但读取不完整' : '扫描读取不完整', message: `${prefix}发现 ${result.discoveredDemFiles} 个 Demo；${result.permissionErrors} 个目录无法读取。` }); else if (!result.discoveredDemFiles) dispatchToast({ tone: 'warn', title: added ? '目录已添加' : '未发现 Demo', message: `${prefix}未发现 .dem；自动录像通常位于 game\\csgo\\replays。` }); else dispatchToast({ tone: result.failed ? 'warn' : 'ready', title: added ? 'Demo 目录已就绪' : 'Demo 扫描完成', message: `${prefix}发现 ${result.discoveredDemFiles} 个，解析 ${result.parsed} 个，缓存 ${result.cacheHits} 个，失败 ${result.failed} 个。` }) }
  async function scan() { if (busy.value === 'scan') return; busy.value = 'scan'; error.value = ''; try { scanResult.value = await api.scanDemoRoots(); await refresh(); announceScan(scanResult.value) } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function importFile(path: string) { busy.value = 'import'; error.value = ''; try { const result = await api.importDemoFile(path); await refresh(); if (result.status === 'done') await openReport(result.demoFileId) } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function openReport(id: number) { busy.value = 'report'; error.value = ''; try { report.value = await api.getDemoReport(id) } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function retry(id: number) { busy.value = `retry-${id}`; try { await api.retryDemoParse(id); await refresh(); await openReport(id) } catch (e) { error.value = normalize(e) } finally { busy.value = '' } }
  async function play(id: number, rootPath: string) { if (rowBusy.value[id]) return; rowBusy.value = { ...rowBusy.value, [id]: 'play' }; try { await api.playDemo(id, rootPath); dispatchToast({ tone: 'ready', title: 'Demo 播放已启动', message: 'CS2 将以离线模式载入所选 Demo。' }) } catch (e) { dispatchToast({ tone: 'danger', title: '无法播放 Demo', message: normalize(e) }) } finally { const next = { ...rowBusy.value }; delete next[id]; rowBusy.value = next } }
  async function reveal(id: number) { if (rowBusy.value[id]) return; rowBusy.value = { ...rowBusy.value, [id]: 'reveal' }; try { await api.revealDemoFile(id) } catch (e) { dispatchToast({ tone: 'danger', title: '无法定位 Demo', message: normalize(e) }) } finally { const next = { ...rowBusy.value }; delete next[id]; rowBusy.value = next } }
  async function loadSettings(rootPath: string) { if (!rootPath) return; try { settings.value = await api.getDemoSettings(rootPath) } catch { settings.value = null } }
  async function setRecording(rootPath: string, enabled: boolean) { busy.value = 'recording'; try { settings.value = await api.setDemoRecordingEnabled(rootPath, enabled) } catch (e) { error.value = normalize(e); throw e } finally { busy.value = '' } }
  return { roots, items, total, report, jobs, query, status, page, pageSize, busy, rowBusy, error, settings, scanResult, refresh, addRoot, updateRoot, removeRoot, scan, importFile, openReport, retry, play, reveal, loadSettings, setRecording }
})
