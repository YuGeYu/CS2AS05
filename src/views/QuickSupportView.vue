<script setup lang="ts">
import { computed, nextTick, onMounted, onBeforeUnmount, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { Bot, Link2, LoaderCircle, Plus, RefreshCw, Send, Settings2, ShoppingBag, Sparkles, Trash2 } from 'lucide-vue-next'
import { chatAi, getAiChatSessions, getAiConnection, getAiModels, openApiPurchase, runAiPowerShell, saveAiChatSessions, saveAiConnection, type AiChatMessage, type AiConnectionSummary } from '@/services/tauri/support'
import { quickSupportState, updateQuickSupportState } from '@/services/quick-support-state'

type SavedSession = { id: string; title: string; messages: AiChatMessage[]; updatedAt: number }
const messages = ref<AiChatMessage[]>([]); const sessions = ref<SavedSession[]>([]); const activeSessionId = ref(''); const draft = ref(''); const busy = ref(false); const status = ref(''); const connection = ref<AiConnectionSummary | null>(null)
const settingsOpen = ref(false); const settingsUrl = ref('https://api.600318.xyz'); const settingsKey = ref(''); const settingsModel = ref('gpt-5.6-sol'); const settingsBusy = ref(false); const settingsState = ref(''); const availableModels = ref<string[]>([]); const modelsBusy = ref(false); const messagesEnd = ref<HTMLElement | null>(null); let stopRequested = false
let saveQueue: Promise<void> = Promise.resolve()
const canSend = computed(() => Boolean(draft.value.trim()) && !busy.value)
const welcome = computed(() => connection.value?.usingDefault ? '默认连接每天有使用限额。遇到额度不足属于正常情况，可在“连接信息”中配置自己的服务。' : '连接已准备好。可以问我关于 CS2、助手设置、Demo 复盘和上游项目的问题。')
function snapshotSessions() { return sessions.value.map(session => ({ ...session, messages: session.messages.map(message => ({ ...message })) })) }
function persist() {
  if (!isTauri()) return
  const snapshot = snapshotSessions()
  saveQueue = saveQueue.catch(() => undefined).then(() => saveAiChatSessions(snapshot))
  void saveQueue.catch(error => { status.value = error instanceof Error ? error.message : '客服会话暂未保存。' })
}
async function loadSessions() {
  if (!isTauri()) return
  try { sessions.value = await getAiChatSessions() } catch (error) { status.value = error instanceof Error ? error.message : '客服历史会话暂不可用。'; sessions.value = [] }
}
function currentContext() {
  const view = document.querySelector<HTMLElement>('.view-container')
  if (!view) return '当前界面没有可读取的状态。'
  const buttons = [...document.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')]
    .filter(button => button.getClientRects().length > 0 && !button.classList.contains('quick-floating-orb'))
    .map((button, index) => {
      const id = `btn-${index}`
      button.dataset.agentButtonId = id
      const label = button.getAttribute('aria-label') || button.title || button.textContent?.replace(/\s+/g, ' ').trim() || '无文字按钮'
      return `${id}: ${label.slice(0, 100)}`
    })
  const page = view.getAttribute('data-current-view') || 'unknown'
  const text = view.textContent?.replace(/\s+/g, ' ').trim().slice(0, 10000) || '无可见文字'
  return `当前页面=${page}\n可点击按钮：\n${buttons.length ? buttons.join('\n') : '无'}\n页面文字：\n${text}`
}
async function scrollBottom() { await nextTick(); messagesEnd.value?.scrollIntoView({ behavior: 'smooth', block: 'end' }) }
function appendToSession(sessionId: string, message: AiChatMessage) { const session = sessions.value.find(item => item.id === sessionId); if (!session) return; session.messages.push(message); session.updatedAt = Date.now(); if (activeSessionId.value === sessionId) messages.value = session.messages; persist() }
function newSession() { const id = crypto.randomUUID(); const session: SavedSession = { id, title: '新的对话', messages: [{ role: 'assistant', content: `你好，我是快快客服。${welcome.value}` }], updatedAt: Date.now() }; sessions.value.unshift(session); activeSessionId.value = id; messages.value = session.messages; status.value = '新会话已开启'; persist(); void scrollBottom() }
function selectSession(session: SavedSession) { if (busy.value) return; activeSessionId.value = session.id; messages.value = session.messages; status.value = '已切换会话'; void scrollBottom() }
function deleteSession(sessionId: string) { if (busy.value) return; sessions.value = sessions.value.filter(item => item.id !== sessionId); if (activeSessionId.value === sessionId) { const next = sessions.value[0]; if (next) { activeSessionId.value = next.id; messages.value = next.messages } else newSession() } persist(); status.value = '旧会话已删除' }
async function waitForAgentSurface() { let previous = ''; let stable = 0; for (let attempt = 0; attempt < 24; attempt += 1) { await new Promise(resolve => setTimeout(resolve, 100)); const view = document.querySelector<HTMLElement>('.view-container'); const signature = `${view?.dataset.currentView}|${view?.querySelectorAll('button').length}|${view?.textContent?.length}`; if (signature === previous) stable += 1; else stable = 0; previous = signature; if (stable >= 3) return } }
async function executeSafeAction(reply: string) {
  const match = reply.match(/\[助手操作：导航=(overview|presets|items|knives|inventory|commands|demoReview|quickSupport|install)\]/)
  if (match) { window.dispatchEvent(new CustomEvent('cs2as:navigate', { detail: match[1] })); return `已执行页面导航：${match[1]}。` }
  const click = reply.match(/\[助手操作：点击=(btn-\d+)\]/)
  if (!click) {
    const powershell = reply.match(/\[助手操作：PowerShell\]\s*```(?:powershell|ps1)?\s*([\s\S]*?)```/i)
    const command = powershell?.[1]?.trim()
    if (!command) return null
    status.value = '正在使用 PowerShell 查询项目与线上资源…'
    updateQuickSupportState({ status: status.value })
    const output = await runAiPowerShell(command)
    return `PowerShell 已执行。\n${output}`
  }
  const buttonId = click[1] || 'btn-unknown'
  const button = document.querySelector<HTMLButtonElement>(`[data-agent-button-id="${buttonId}"]`)
  if (!button || button.disabled || button.getClientRects().length === 0) return `按钮 ${buttonId} 当前不可点击，请重新读取页面。`
  const label = button.getAttribute('aria-label') || button.title || button.textContent?.replace(/\s+/g, ' ').trim() || buttonId
  button.click()
  return `已点击按钮：${label.slice(0, 120)}。`
}
function stop() { stopRequested = true; busy.value = false; updateQuickSupportState({ busy: false, status: '已停止本轮操作', stop: undefined }); status.value = '已停止本轮操作' }
async function send() {
  if (!canSend.value) return
  const content = draft.value.trim(); draft.value = ''; const targetSessionId = activeSessionId.value
  appendToSession(targetSessionId, { role: 'user', content }); busy.value = true; stopRequested = false
  status.value = '正在整理问题并读取当前界面…'; updateQuickSupportState({ busy: true, status: '正在读取当前界面并整理回答…', stop }); await scrollBottom()
  try {
    const target = sessions.value.find(item => item.id === targetSessionId)
    if (!target) return
    const workingMessages = [...target.messages]
    for (let step = 0; step < 6; step += 1) {
      if (stopRequested) return
      status.value = step === 0 ? '正在整理回答…' : `已完成第 ${step} 步，正在回读界面…`
      updateQuickSupportState({ status: status.value })
      const reply = await chatAi(workingMessages, currentContext())
      if (stopRequested) return
      const displayReply = reply.replace(/\n?\[助手操作：(导航=[^\]]+|点击=[^\]]+)\]/g, '').replace(/\n?\[助手操作：PowerShell\]\s*```(?:powershell|ps1)?\s*[\s\S]*?```/gi, '').trim()
      if (displayReply) appendToSession(targetSessionId, { role: 'assistant', content: displayReply })
      workingMessages.push({ role: 'assistant', content: reply }); await scrollBottom()
      const actionResult = await executeSafeAction(reply)
      if (!actionResult) break
      workingMessages.push({ role: 'user', content: `[客户端执行结果] ${actionResult}` })
      await waitForAgentSurface()
    }
    status.value = '回答完成'; updateQuickSupportState({ busy: false, status: '已完成本轮回答', stop: undefined })
  } catch (error) { if (!stopRequested) { status.value = error instanceof Error ? error.message : String(error); updateQuickSupportState({ busy: false, status: '本轮请求失败', stop: undefined }) } }
  finally { busy.value = false }
}
async function refreshModels() { if (modelsBusy.value) return; modelsBusy.value = true; settingsState.value = '正在读取这个连接的可用模型…'; try { availableModels.value = await getAiModels(settingsUrl.value, settingsKey.value); if (!availableModels.value.includes(settingsModel.value)) settingsModel.value = availableModels.value[0] || ''; settingsState.value = `已读取 ${availableModels.value.length} 个可用模型。` } catch (error) { availableModels.value = []; settingsState.value = error instanceof Error ? error.message : String(error) } finally { modelsBusy.value = false } }
async function openSettings() { settingsState.value = ''; const current = connection.value || await getAiConnection(); connection.value = current; settingsUrl.value = current.url; settingsModel.value = current.model; settingsKey.value = ''; settingsOpen.value = true; await refreshModels() }
async function saveSettings() { if (settingsBusy.value) return; settingsBusy.value = true; settingsState.value = ''; try { connection.value = await saveAiConnection(settingsUrl.value, settingsKey.value, settingsModel.value); settingsState.value = '连接信息已保存。密钥只保存在本机，不会显示在页面或日志中。'; settingsKey.value = '' } catch (error) { settingsState.value = error instanceof Error ? error.message : String(error) } finally { settingsBusy.value = false } }
async function buyConnection() { await openApiPurchase().catch(error => { settingsState.value = String(error) }) }
onMounted(async () => { await loadSessions(); connection.value = await getAiConnection().catch(() => null); const first = sessions.value[0]; if (!first) newSession(); else { activeSessionId.value = first.id; messages.value = first.messages } updateQuickSupportState({ status: '待命，等待你的问题' }) })
onBeforeUnmount(() => { if (busy.value) updateQuickSupportState({ busy: true, status: '快快正在后台整理回答，点击浮动圆钮可停止', stop }); else updateQuickSupportState({ busy: false, stop: undefined }) })
</script>
<template>
  <section class="quick-support-view" aria-labelledby="quick-support-title">
    <header class="quick-support-header"><div><p class="overline">QUICK SUPPORT</p><h1 id="quick-support-title"><Sparkles :size="24" />快快客服</h1><p>一个会记住上下文、理解助手界面的 AI 搭档。</p></div><div class="quick-support-actions"><button class="secondary-button" type="button" @click="newSession"><Plus :size="17" />开启新会话</button><button class="secondary-button" type="button" @click="openSettings"><Settings2 :size="17" />连接信息</button><button class="secondary-button" type="button" @click="buyConnection"><ShoppingBag :size="17" />购买连接信息</button></div></header>
    <div class="quick-support-meta"><span><Bot :size="15" />模型：{{ connection?.model || 'gpt-5.6-sol' }}</span><span><Link2 :size="15" />{{ connection?.url || 'https://api.600318.xyz' }}</span><span v-if="connection?.usingDefault" class="quick-support-limit">默认配置每日限额</span></div>
    <div class="quick-support-layout"><aside class="quick-session-list" aria-label="历史会话"><h2>会话</h2><div v-for="session in sessions" :key="session.id" class="quick-session-row" :class="{ active: session.id === activeSessionId }"><button class="quick-session-open" type="button" @click="selectSession(session)"><strong>{{ session.title }}</strong><small>{{ new Date(session.updatedAt).toLocaleString() }}</small></button><button class="quick-session-delete" type="button" title="删除旧会话" aria-label="删除旧会话" :disabled="busy" @click="deleteSession(session.id)"><Trash2 :size="14" /></button></div></aside><section class="quick-chat-panel" aria-label="快快客服聊天区"><div class="quick-chat-messages"><article v-for="(message, index) in messages" :key="index" class="quick-message" :class="`quick-message-${message.role}`"><div class="quick-message-role">{{ message.role === 'assistant' ? '快快客服' : '你' }}</div><p>{{ message.content }}</p></article><div v-if="busy" class="quick-message quick-message-assistant"><LoaderCircle :size="18" class="spin" /><span>{{ quickSupportState.status }}</span></div><div ref="messagesEnd" /></div><p v-if="status" class="quick-support-status" role="status">{{ status }}</p><form class="quick-chat-composer" @submit.prevent="send"><textarea v-model="draft" rows="2" placeholder="描述你的问题…" aria-label="输入消息" @keydown.enter.exact.prevent="send" /><button class="primary-button" type="submit" :disabled="!canSend"><LoaderCircle v-if="busy" :size="18" class="spin" /><Send v-else :size="18" />发送</button></form></section><aside class="quick-support-aside"><div class="quick-aside-icon"><Sparkles :size="22" /></div><h2>工作状态</h2><p class="quick-agent-status">{{ quickSupportState.status }}</p><ul><li>每次回复固定写回发起请求的会话</li><li>等待新页面稳定后读取全部可见按钮</li><li>可执行导航和页面按钮点击</li><li>可检索公开的上游项目与源码资料</li></ul><p class="quick-support-note">页面操作始终开启。浮动圆钮可随时停止本轮操作；快快每次点击后会重新读取页面，再决定下一步。</p></aside></div>
  </section>
  <Teleport to="body"><div v-if="settingsOpen" class="modal-backdrop"><section class="confirm-dialog quick-settings-dialog" role="dialog" aria-modal="true" aria-labelledby="quick-settings-title"><div class="modal-heading"><Link2 :size="22" /><div><p class="overline">CONNECTION</p><h2 id="quick-settings-title">连接信息</h2><p>自定义中转站地址、密钥与模型。密钥仅保存在本机。</p></div></div><label>URL<input v-model="settingsUrl" type="url" placeholder="https://api.600318.xyz" autocomplete="url" /></label><label>Key<input v-model="settingsKey" type="password" placeholder="留空则使用已保存的密钥" autocomplete="off" /></label><div class="quick-model-field"><label>模型<select v-model="settingsModel" :disabled="modelsBusy || !availableModels.length"><option v-for="model in availableModels" :key="model" :value="model">{{ model }}</option></select></label><button class="secondary-button" type="button" :disabled="modelsBusy" @click="refreshModels"><LoaderCircle v-if="modelsBusy" :size="16" class="spin" /><RefreshCw v-else :size="16" />重新读取</button></div><p v-if="settingsState" class="quick-support-status" role="status">{{ settingsState }}</p><div class="dialog-actions"><button class="secondary-button" type="button" @click="settingsOpen = false">关闭</button><button class="primary-button" type="button" :disabled="settingsBusy || !settingsModel" @click="saveSettings"><LoaderCircle v-if="settingsBusy" :size="17" class="spin" /><span v-else>保存连接</span></button></div></section></div></Teleport>
</template>
