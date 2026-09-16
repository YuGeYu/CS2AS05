<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { AlertTriangle, FileUp, Heart, MessagesSquare, Mic, MicOff, Paperclip, Send, Users, WifiOff } from 'lucide-vue-next'
import { getCommunityAuth, launchCommunityConnect, openCommunityDownload } from '@/services/tauri/support'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
type ChatMessage = {
  id: string
  display_name: string
  user_id: string
  content: string
  kind: 'text' | 'file'
  role?: string
  created_at: number
  reactions?: Record<string, number>
}
type Peer = {
  id: string
  name: string
  connection: RTCPeerConnection
  audio?: HTMLAudioElement
  gain?: GainNode
  input?: MediaStreamAudioSourceNode
  context?: AudioContext
}
const messages = ref<ChatMessage[]>([])
const draft = ref('')
const connected = ref(false)
const memberCount = ref(0)
const currentRole = ref('user')
const error = ref('')
const editingId = ref<string | null>(null)
const editDraft = ref('')
const voiceOpen = ref(false)
const inputGain = ref(1)
const outputGain = ref(1)
const micOn = ref(false)
const uploadBusy = ref(false)
const contextMessage = ref<ChatMessage | null>(null)
const contextX = ref(0)
const contextY = ref(0)
const peerGains = ref<Record<string, number>>({})
const loopbackTesting = ref(false)
const cs2 = useCs2Store()
const panel = usePanelStore()
let loopbackContext: AudioContext | undefined
let loopbackSource: MediaStreamAudioSourceNode | undefined
let loopbackGain: GainNode | undefined
let socket: WebSocket | undefined
let socketTimer: ReturnType<typeof setTimeout> | undefined
let authToken = ''
let serviceUrl = ''
let selfId = ''
let media: MediaStream | undefined
const peers = new Map<string, Peer>()
function emit(value: unknown) {
  if (socket?.readyState === WebSocket.OPEN) socket.send(JSON.stringify(value))
}
async function connect() {
  try {
    const auth = await getCommunityAuth()
    authToken = auth.token
    serviceUrl = auth.serviceUrl.replace(/\/$/, '')
    const websocketUrl = `${serviceUrl.replace(/^http:/, 'ws:').replace(/^https:/, 'wss:')}/ws?room=lobby&token=${encodeURIComponent(authToken)}`
    socket = new WebSocket(websocketUrl)
    socketTimer = setTimeout(() => {
      if (!connected.value) {
        socket?.close()
        error.value = '圈子连接超时，请检查上海服务器入口'
      }
    }, 10000)
    socket.onopen = () => {
      if (socketTimer) clearTimeout(socketTimer)
      connected.value = true
      error.value = ''
    }
    socket.onclose = (event) => {
      if (socketTimer) clearTimeout(socketTimer)
      connected.value = false
      if (event.code === 1008) error.value = '圈子令牌无效或已过期，请重新登录官网账号'
      else if (event.code === 1013) error.value = '圈子人数已满，请稍后重试'
      else if (!error.value) error.value = `圈子连接已断开（${event.code}）`
    }
    socket.onerror = () => {
      error.value = '圈子服务暂时不可用，请稍后重试'
    }
    socket.onmessage = (event) => {
      const data = JSON.parse(event.data)
      if (data.type === 'message-deleted') messages.value = messages.value.filter((message) => message.id !== data.id)
      void handleMessage(data)
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : '登录圈子失败'
    window.dispatchEvent(new CustomEvent('cs2as:navigate', { detail: 'install' }))
    window.setTimeout(() => window.dispatchEvent(new CustomEvent('cs2as:focus-account')), 80)
  }
}
async function handleMessage(data: any) {
  if (data.type === 'ready') {
    selfId = data.user.id
    currentRole.value = data.user.role || 'user'
  }
  if (data.type === 'history') messages.value = data.messages
  if (data.type === 'message') messages.value.push(data.message)
  if (data.type === 'message-edited') {
    const item = messages.value.find((message) => message.id === data.id)
    if (item) item.content = data.content
  }
  if (data.type === 'presence') memberCount.value = data.count
  if (data.type === 'reaction') {
    const item = messages.value.find((message) => message.id === data.id)
    if (item)
      item.reactions = {
        ...(item.reactions || {}),
        [data.emoji]: ((item.reactions || {})[data.emoji] || 0) + 1,
      }
  }
  if (data.type === 'peer-joined' && voiceOpen.value && data.id !== selfId) await createPeer(data.id, data.displayName, true)
  if (data.type === 'peer-left') closePeer(data.id)
  if (data.type === 'signal') await receiveSignal(data.from, data.payload)
}
function sendMessage() {
  const content = draft.value.trim()
  if (!content) return
  emit({ type: 'chat', content })
  draft.value = ''
}
function beginEdit(message: ChatMessage) {
  if (message.user_id !== selfId) return
  editingId.value = message.id
  editDraft.value = message.content
}
function saveEdit() {
  if (!editingId.value) return
  emit({ type: 'edit', id: editingId.value, content: editDraft.value })
  editingId.value = null
}
function copy(content: string) {
  void navigator.clipboard?.writeText(content)
}
async function openDownload(message: ChatMessage) {
  const file = resource(message)
  const open = () => openCommunityDownload(`${serviceUrl}${file.url}?token=${encodeURIComponent(authToken)}&download=1`)
  try {
    await open()
  } catch (cause) {
    const text = cause instanceof Error ? cause.message : String(cause)
    if (/expired|401|令牌/i.test(text)) {
      try {
        const auth = await getCommunityAuth()
        authToken = auth.token
        serviceUrl = auth.serviceUrl.replace(/\/$/, '')
        await open()
      } catch (retry) {
        error.value = retry instanceof Error ? retry.message : '资源令牌已刷新，但下载仍失败'
      }
    } else error.value = text || '无法打开默认浏览器下载资源'
  }
  contextMessage.value = null
}
function react(message: ChatMessage, emoji = '❤️') {
  emit({ type: 'reaction', id: message.id, emoji })
}
function canDelete(message: ChatMessage) {
  return message.user_id === selfId || currentRole.value === 'owner' || (currentRole.value === 'admin' && message.role === 'user')
}
function deleteMessage(message: ChatMessage) {
  if (!canDelete(message)) return
  emit({ type: 'delete', id: message.id })
  contextMessage.value = null
}
function showContext(event: MouseEvent, message: ChatMessage) {
  event.preventDefault()
  contextMessage.value = message
  contextX.value = event.clientX
  contextY.value = event.clientY
}
function resource(message: ChatMessage) {
  try {
    return JSON.parse(message.content) as {
      name: string
      size: number
      url: string
      type?: string
    }
  } catch {
    return { name: '资源', size: 0, url: '', type: '' }
  }
}
function connectionInfo(message: ChatMessage) {
  return message.kind === 'text' ? message.content.match(/^connect\s+(\[A:\d+:\d+:\d+\]\s+\(\d+\))$/i)?.[1] || '' : ''
}
async function connectToServer(message: ChatMessage) {
  const connection = connectionInfo(message)
  if (!connection) return
  if (!cs2.selectedRoot) {
    error.value = '请先在概览页选择 CS2 游戏目录。'
    return
  }
  try {
    const snapshot = await launchCommunityConnect(cs2.selectedRoot, connection)
    panel.applySnapshot(cs2.selectedRoot, snapshot)
    contextMessage.value = null
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : '无法启动 CS2'
  }
}
async function download(message: ChatMessage) {
  await openDownload(message)
}
function isImage(message: ChatMessage) {
  return /^image\/(png|jpe?g|gif|webp|bmp|svg\+xml)$/i.test(resource(message).type || '') || /\.(png|jpe?g|gif|webp|bmp|svg)$/i.test(resource(message).name)
}
async function upload(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  if (file.size > 10 * 1024 * 1024) {
    error.value = '资源不能超过 10 MB'
    return
  }
  uploadBusy.value = true
  try {
    const response = await fetch(`${serviceUrl}/upload?token=${encodeURIComponent(authToken)}`, {
      method: 'POST',
      headers: {
        'content-type': file.type || 'application/octet-stream',
        'x-file-name': encodeURIComponent(file.name),
      },
      body: file,
    })
    if (!response.ok) {
      let detail = ''
      try {
        detail = ((await response.json()) as { error?: string }).error || ''
      } catch {}
      throw new Error(detail || `上传失败（HTTP ${response.status}）`)
    }
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : '上传失败'
  } finally {
    uploadBusy.value = false
    ;(event.target as HTMLInputElement).value = ''
  }
}
async function toggleVoice() {
  voiceOpen.value = !voiceOpen.value
  if (!voiceOpen.value) {
    media?.getTracks().forEach((track) => track.stop())
    media = undefined
    for (const id of peers.keys()) closePeer(id)
    micOn.value = false
    return
  }
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true })
    media = stream
    micOn.value = true
    for (const peer of peers.values()) stream.getTracks().forEach((track) => peer.connection.addTrack(track, stream))
  } catch {
    error.value = '无法访问麦克风，请检查系统权限'
  }
}
async function createPeer(id: string, name: string, initiator: boolean) {
  if (peers.has(id)) return
  const connection = new RTCPeerConnection({
    iceServers: [{ urls: 'stun:stun.l.google.com:19302' }],
  })
  const peer: Peer = { id, name, connection }
  peers.set(id, peer)
  const stream = media as any
  if (stream) {
    for (const track of stream.getTracks()) connection.addTrack(track, stream as any)
  }
  connection.onicecandidate = (event) => {
    if (event.candidate) emit({ type: 'signal', to: id, payload: { candidate: event.candidate } })
  }
  connection.ontrack = (event) => {
    const context = new AudioContext()
    const source = context.createMediaStreamSource(event.streams[0] as MediaStream)
    const gain = context.createGain()
    gain.gain.value = outputGain.value * (peerGains.value[id] ?? 1)
    source.connect(gain).connect(context.destination)
    peer.gain = gain
    peer.context = context
  }
  if (initiator) {
    const offer = await connection.createOffer()
    await connection.setLocalDescription(offer)
    emit({ type: 'signal', to: id, payload: { description: connection.localDescription } })
  }
}
async function receiveSignal(id: string, payload: any) {
  let peer = peers.get(id)
  if (!peer) {
    await createPeer(id, '玩家', false)
    peer = peers.get(id)!
  }
  if (payload.description) {
    await peer.connection.setRemoteDescription(payload.description)
    if (payload.description.type === 'offer') {
      const answer = await peer.connection.createAnswer()
      await peer.connection.setLocalDescription(answer)
      emit({ type: 'signal', to: id, payload: { description: peer.connection.localDescription } })
    }
  }
  if (payload.candidate) await peer.connection.addIceCandidate(payload.candidate)
}
function closePeer(id: string) {
  const peer = peers.get(id)
  peer?.connection.close()
  peer?.context?.close()
  peer?.audio?.remove()
  peers.delete(id)
  delete peerGains.value[id]
}
function setOutput() {
  for (const peer of peers.values()) if (peer.gain) peer.gain.gain.value = outputGain.value
}
function setInput() {
  if (!media) return
  const value = inputGain.value
  for (const track of media.getAudioTracks()) {
    const settings = track.getSettings()
    const constraints = { ...settings, volume: value }
    void track.applyConstraints(constraints).catch(() => {})
  }
}
function setPeerGain(id: string) {
  const peer = peers.get(id)
  if (peer?.gain) peer.gain.gain.value = outputGain.value * (peerGains.value[id] ?? 1)
}
async function toggleLoopback() {
  if (loopbackTesting.value) {
    loopbackSource?.disconnect()
    loopbackGain?.disconnect()
    await loopbackContext?.close()
    loopbackContext = undefined
    loopbackTesting.value = false
    return
  }
  try {
    if (!media) media = await navigator.mediaDevices.getUserMedia({ audio: true })
    loopbackContext = new AudioContext()
    loopbackSource = loopbackContext.createMediaStreamSource(media)
    loopbackGain = loopbackContext.createGain()
    loopbackGain.gain.value = outputGain.value
    loopbackSource.connect(loopbackGain).connect(loopbackContext.destination)
    loopbackTesting.value = true
  } catch {
    error.value = '无法开启本地回环，请检查麦克风权限'
  }
}
onMounted(connect)
onBeforeUnmount(() => {
  if (socketTimer) clearTimeout(socketTimer)
  socket?.close()
  void loopbackContext?.close()
  media?.getTracks().forEach((track) => track.stop())
  for (const id of peers.keys()) closePeer(id)
})
</script>
<template>
  <section class="community-view" @click="contextMessage = null">
    <header class="community-hero">
      <div>
        <p class="overline">CS2AS · 玩家连接</p>
        <h1>玩家圈子</h1>
        <p>临时消息、资源与语音房，和正在开黑的人保持同一条线上。</p>
      </div>
      <div class="community-status" :data-connected="connected"><span class="status-dot" />{{ connected ? `${memberCount} 人在线` : '正在连接' }}</div>
    </header>
    <div class="community-layout">
      <aside class="community-rooms">
        <h2>分组</h2>
        <button class="community-room is-active" type="button">
          <MessagesSquare :size="18" /><span>聊天大厅</span><b>{{ messages.length }}</b></button
        ><button class="community-room" type="button" :aria-expanded="voiceOpen" @click="toggleVoice">
          <component :is="micOn ? Mic : MicOff" :size="18" /><span>语音大厅</span><span class="voice-untested-badge" title="语音功能尚未完成真实多人测试">未测试</span><span class="room-state">{{ voiceOpen ? '已进入' : '进入' }}</span>
        </button>
        <div v-if="voiceOpen" class="voice-controls">
          <div class="voice-untested-warning" role="status"><AlertTriangle :size="16" aria-hidden="true" /><span><strong>语音功能未测试</strong><small>当前仅完成代码链路，尚未验证真实多人、跨网络和麦克风设备兼容性。</small></span></div>
          <label>麦克风输入<input v-model.number="inputGain" type="range" min="0" max="4" step="0.01" /></label><label>总接收音量<input v-model.number="outputGain" type="range" min="0" max="4" step="0.01" @input="setOutput" /></label
          ><button v-if="currentRole === 'owner'" type="button" class="voice-test-button" @click="toggleLoopback">
            {{ loopbackTesting ? '停止本地回环' : '本地回环测试' }}</button
          ><small>自由输入 · 语音不录音 · 单人音量随连接自动显示</small>
        </div>
      </aside>
      <main class="community-chat">
        <div v-if="error" class="community-error"><WifiOff :size="16" />{{ error }}</div>
        <div class="community-messages" aria-live="polite">
          <article v-for="message in messages" :key="message.id" class="community-message" @contextmenu="showContext($event, message)">
            <div class="message-meta">
              <strong>{{ message.display_name }}</strong
              ><time>{{
                new Date(message.created_at).toLocaleString('zh-CN', {
                  year: 'numeric',
                  month: '2-digit',
                  day: '2-digit',
                  hour: '2-digit',
                  minute: '2-digit',
                })
              }}</time>
            </div>
            <template v-if="editingId === message.id"><input v-model="editDraft" @keyup.enter="saveEdit" /><button type="button" @click="saveEdit">保存</button></template
            ><template v-else-if="message.kind === 'file'"
              ><template v-if="isImage(message)"><img class="resource-image" :src="`${serviceUrl}${resource(message).url}?token=${encodeURIComponent(authToken)}`" :alt="resource(message).name" loading="lazy" /></template
              ><a v-else class="resource-card" :href="`${serviceUrl}${resource(message).url}?token=${encodeURIComponent(authToken)}`" target="_blank" rel="noreferrer"
                ><FileUp :size="18" /><span>{{ resource(message).name }}</span
                ><small>{{ Math.round(resource(message).size / 1024) }} KB</small></a
              ></template
            >
            <p v-else class="message-content" @dblclick="beginEdit(message)">
              {{ message.content }}
            </p>
            <div class="message-actions">
              <button type="button" @click="copy(message.content)">复制</button><button type="button" @click="react(message)"><Heart :size="13" /> {{ message.reactions?.['❤️'] || 0 }}</button><button v-if="message.user_id === selfId" type="button" @click="beginEdit(message)">编辑</button><button v-if="canDelete(message)" type="button" @click="deleteMessage(message)">撤回</button>
            </div>
          </article>
          <p v-if="!messages.length" class="community-empty">这里还没有消息，先打个招呼吧。</p>
        </div>
        <div v-if="contextMessage" class="community-context-menu" :style="{ left: `${contextX}px`, top: `${contextY}px` }" @click.stop>
          <button v-if="canDelete(contextMessage)" type="button" @click="deleteMessage(contextMessage)">直接删除这条消息</button><button v-if="contextMessage.kind === 'file'" type="button" @click="openDownload(contextMessage)">下载资源</button>
          <button v-if="connectionInfo(contextMessage)" type="button" @click="connectToServer(contextMessage)">连接这台服务器</button>
        </div>
        <form class="community-composer" @submit.prevent="sendMessage">
          <label class="upload-button" title="上传资源（最大 10 MB）"><Paperclip :size="19" /><input type="file" hidden @change="upload" /></label><input v-model="draft" maxlength="4000" placeholder="输入消息，连接信息会自动规范化" aria-label="圈子消息" /><button class="send-button" type="submit" :disabled="!connected || !draft.trim() || uploadBusy" title="发送">
            <Send :size="18" />
          </button>
        </form>
      </main>
      <aside class="community-info">
        <Users :size="18" /><strong>临时社区</strong>
        <p>消息、资源和赞赏图片 40 小时后自动清理；语音大厅可用“本地回环测试”独自验证麦克风与音量。</p>
      </aside>
    </div>
  </section>
</template>
