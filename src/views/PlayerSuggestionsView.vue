<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Lightbulb, LogIn, LogOut, Pencil, RefreshCw, Send, ShieldCheck, Trash2 } from 'lucide-vue-next'
import ConfirmDialog from '@/components/ui/ConfirmDialog.vue'
import { dispatchToast } from '@/services/toast'
import {
  createIdea,
  createIdeaSection,
  deleteIdea,
  deleteIdeaSection,
  getAssistantAccount,
  loadIdeas,
  loginAssistant,
  logoutAssistant,
  quickRegisterAssistant,
  replyIdea,
  updateIdea,
  updateIdeaSection,
  type IdeaEntry,
  type IdeaNotice,
  type IdeaSection,
  type IdeaUser,
} from '@/services/tauri/support'

const sections = ref<IdeaSection[]>([])
const notices = ref<IdeaNotice[]>([])
const user = ref<IdeaUser | null>(null)
const accountName = ref<string | null>(null)
const activeSection = ref('')
const content = ref('')
const status = ref('')
const loading = ref(false)
const editing = ref<IdeaEntry | null>(null)
const replyDraft = ref<Record<string, string>>({})
const sectionTitle = ref('')
const editingSectionId = ref('')
const sectionDraft = ref('')
const pendingDelete = ref<{ title: string; description: string; run: () => Promise<void> } | null>(null)
const deleteBusy = ref(false)
const loginName = ref('')
const loginPassword = ref('')
const loginBusy = ref(false)

const isAdmin = computed(() => Boolean(user.value && ['admin', 'owner'].includes(user.value.role)))
const isOwner = computed(() => user.value?.role === 'owner')
const activeSectionData = computed(() => sections.value.find(section => section.id === activeSection.value) ?? null)

function formatDate(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value.slice(0, 10) : date.toLocaleString('zh-CN', { dateStyle: 'medium', timeStyle: 'short' })
}

function faultStatusText(value: IdeaSection['faultStatus']) {
  return ({ pending: '待处理', investigating: '处理中', resolved: '已解决', closed: '已关闭' } as Record<string, string>)[value || ''] || '待处理'
}

function noticeSeverityLabel(value: IdeaNotice['severity']) {
  return ({ success: '成功', warning: '提醒', error: '重要' } as const)[value]
}

async function load() {
  loading.value = true
  status.value = ''
  try {
    const payload = await loadIdeas()
    notices.value = payload.notices ?? (payload.notice ? [payload.notice] : [])
    sections.value = payload.sections ?? []
    user.value = payload.user ?? null
    if (!sections.value.some(section => section.id === activeSection.value)) activeSection.value = sections.value[0]?.id ?? ''
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  } finally {
    loading.value = false
  }
}

async function refreshAccount() {
  const account = await getAssistantAccount().catch(() => ({ username: null, loggedIn: false }))
  accountName.value = account.username
}

async function signIn() {
  if (!loginName.value.trim() || !loginPassword.value) return
  loginBusy.value = true
  status.value = ''
  try {
    await loginAssistant(loginName.value.trim(), loginPassword.value)
    loginPassword.value = ''
    await refreshAccount()
    await load()
    dispatchToast({ tone: 'ready', title: '登录成功', message: '现在可以提交和管理自己的玩家建议了。' })
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  } finally {
    loginBusy.value = false
  }
}

async function signOut() {
  await logoutAssistant().catch(() => undefined)
  user.value = null
  accountName.value = null
  dispatchToast({ tone: 'info', title: '已退出登录', message: '你仍然可以继续查看公开的玩家建议。' })
}

async function quickRegister() {
  loginBusy.value = true
  status.value = ''
  try {
    await quickRegisterAssistant()
    await refreshAccount()
    await load()
    dispatchToast({ tone: 'ready', title: '账号已准备好', message: '已完成一键注册，现在可以提交玩家建议。' })
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  } finally {
    loginBusy.value = false
  }
}

async function submitIdea() {
  if (!activeSection.value || !content.value.trim()) return
  status.value = ''
  try {
    if (editing.value) {
      await updateIdea(editing.value.id, content.value)
      editing.value = null
    } else {
      await createIdea(activeSection.value, content.value)
    }
    content.value = ''
    await load()
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  }
}

function editIdea(idea: IdeaEntry) {
  editing.value = idea
  content.value = idea.content
  activeSection.value = sections.value.find(section => section.ideas.some(item => item.id === idea.id))?.id ?? activeSection.value
}

function removeIdea(idea: IdeaEntry) {
  pendingDelete.value = {
    title: '删除这条玩家建议？',
    description: '这条建议和已有管理员回复都会被移除，此操作无法恢复。',
    run: async () => { await deleteIdea(idea.id); await load() },
  }
}

async function saveReply(idea: IdeaEntry) {
  try {
    await replyIdea(idea.id, replyDraft.value[idea.id] || idea.reply?.content || '')
    await load()
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  }
}

async function addSection() {
  if (!sectionTitle.value.trim()) return
  try {
    await createIdeaSection(sectionTitle.value)
    sectionTitle.value = ''
    await load()
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  }
}

function beginRename(section: IdeaSection) {
  editingSectionId.value = section.id
  sectionDraft.value = section.title
}

async function saveSectionTitle(section: IdeaSection) {
  if (!sectionDraft.value.trim()) return
  try {
    await updateIdeaSection(section.id, sectionDraft.value)
    editingSectionId.value = ''
    sectionDraft.value = ''
    await load()
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  }
}

function removeSection(section: IdeaSection) {
  pendingDelete.value = {
    title: `删除“${section.title}”？`,
    description: section.ideas.length ? `该分区下的 ${section.ideas.length} 条建议及管理员回复会一并永久删除。` : '该分区目前没有建议，删除后无法恢复。',
    run: async () => { await deleteIdeaSection(section.id); await load() },
  }
}

async function confirmDelete() {
  if (!pendingDelete.value || deleteBusy.value) return
  deleteBusy.value = true
  try {
    await pendingDelete.value.run()
    pendingDelete.value = null
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error)
  } finally {
    deleteBusy.value = false
  }
}

function canEdit(idea: IdeaEntry) {
  return isAdmin.value || idea.user.id === user.value?.id
}

onMounted(async () => {
  await refreshAccount()
  await load()
})
</script>

<template>
  <section class="tool-view player-suggestions-view" aria-labelledby="player-suggestions-title">
    <header class="player-suggestions-heading">
      <div><p class="overline">社区共创 / PLAYER VOICE</p><h1 id="player-suggestions-title">玩家建议</h1><p>把每一条想法留在版本迭代里，公开查看进展，也能收到管理员回复。</p></div>
      <button class="icon-button" type="button" title="刷新玩家建议" aria-label="刷新玩家建议" :disabled="loading" @click="load"><RefreshCw :size="18" :class="{ spinning: loading }" /></button>
    </header>

    <section class="player-suggestions-account" aria-label="玩家建议账号状态">
      <div class="player-suggestions-account-icon"><ShieldCheck :size="20" /></div>
      <div><strong>{{ accountName ? `已登录：${accountName}` : '登录后参与版本共创' }}</strong><p>{{ accountName ? '建议会同步到官网意见页，并保留你的编辑权限。' : '未登录也可以查看公开内容；提交、修改和删除建议需要官网账号。' }}</p></div>
      <button v-if="accountName" class="secondary-button" type="button" @click="signOut"><LogOut :size="16" />退出登录</button>
    </section>

    <form v-if="!accountName" class="player-suggestions-login" @submit.prevent="signIn">
      <div><label for="suggestions-login-name">账号</label><input id="suggestions-login-name" v-model="loginName" autocomplete="username" placeholder="官网账号" /></div>
      <div><label for="suggestions-login-password">密码</label><input id="suggestions-login-password" v-model="loginPassword" type="password" autocomplete="current-password" placeholder="官网密码" /></div>
      <div class="player-suggestions-login-actions"><button class="primary-button" type="submit" :disabled="loginBusy || !loginName.trim() || !loginPassword"><LogIn :size="17" />{{ loginBusy ? '登录中…' : '登录并参与' }}</button><button class="secondary-button" type="button" :disabled="loginBusy" @click="quickRegister"><Lightbulb :size="17" />一键注册</button></div>
    </form>

    <section v-if="notices.length" class="player-suggestions-notices" aria-label="官网公告">
      <article v-for="notice in notices" :key="notice.id" class="player-suggestion-notice" :data-severity="notice.severity"><header><strong>{{ notice.title }}</strong><span>{{ noticeSeverityLabel(notice.severity) }}</span><time>{{ formatDate(notice.publishedAt) }}</time></header><p>{{ notice.content }}</p></article>
    </section>

    <form class="player-suggestions-composer" @submit.prevent="submitIdea">
      <div class="player-suggestions-composer-heading"><div><span class="overview-kicker">WRITE IT DOWN</span><h2>{{ editing ? '修改玩家建议' : '提交一条玩家建议' }}</h2></div><span class="player-suggestions-counter">{{ content.length }} / 600</span></div>
      <label>归属分区<select v-model="activeSection" :disabled="!accountName || !sections.length"><option v-for="section in sections" :key="section.id" :value="section.id">{{ section.title }}</option></select></label>
      <label>建议内容<textarea v-model="content" maxlength="600" :disabled="!accountName || !activeSectionData || activeSectionData.locked" placeholder="告诉我们下个版本最值得改进的地方…" /></label>
      <div class="player-suggestions-form-actions"><button class="primary-button" type="submit" :disabled="!accountName || !content.trim() || !activeSection || activeSectionData?.locked"><Send :size="17" />{{ editing ? '保存修改' : '提交建议' }}</button><button v-if="editing" class="text-button" type="button" @click="editing = null; content = ''">取消编辑</button></div>
    </form>
    <p v-if="!accountName" class="hint">登录后即可提交建议。每位普通用户每天最多新增 10 条，管理员账号每天最多 20 条。</p>
    <p v-if="status" class="inline-error" role="alert">{{ status }}</p>

    <form v-if="isAdmin" class="player-suggestions-admin" @submit.prevent="addSection"><input v-model="sectionTitle" maxlength="32" placeholder="新增分区，例如 下个版本意见" /><button class="secondary-button" type="submit"><Lightbulb :size="16" />新增分区</button></form>

    <section v-for="section in sections" :id="section.kind === 'fault' ? `fault-section-${section.id}` : undefined" :key="section.id" class="player-suggestion-section" :data-kind="section.kind">
      <header class="player-suggestion-section-heading">
        <form v-if="editingSectionId === section.id" class="player-suggestion-rename" @submit.prevent="saveSectionTitle(section)"><input v-model="sectionDraft" maxlength="32" :aria-label="`编辑${section.title}标题`" /><button class="secondary-button" type="submit">保存</button><button class="text-button" type="button" @click="editingSectionId = ''">取消</button></form>
        <div v-else><span class="overview-kicker">SECTION {{ String(section.sortOrder).padStart(2, '0') }}</span><h2>{{ section.title }}</h2></div>
        <div v-if="section.kind === 'fault'" class="player-suggestion-fault-status"><span>故障状态</span><strong>{{ faultStatusText(section.faultStatus) }}</strong></div>
        <div v-else-if="isAdmin" class="player-suggestion-inline-actions"><button class="text-button" type="button" @click="beginRename(section)"><Pencil :size="14" />改标题</button><button class="text-button danger" type="button" @click="removeSection(section)"><Trash2 :size="14" />删除</button></div>
      </header>
      <ol v-if="section.ideas.length" class="player-suggestion-list"><li v-for="idea in section.ideas" :key="idea.id"><p>{{ idea.content }}</p><div class="player-suggestion-meta"><span class="player-suggestion-avatar">{{ idea.user.displayName.slice(0, 1).toUpperCase() }}</span><span>{{ idea.user.displayName }}</span><time>{{ formatDate(idea.createdAt) }}</time><button v-if="!section.locked && canEdit(idea)" class="text-button" type="button" @click="editIdea(idea)"><Pencil :size="14" />修改</button><button v-if="!section.locked && (idea.user.id === user?.id || isOwner)" class="text-button danger" type="button" @click="removeIdea(idea)"><Trash2 :size="14" />删除</button></div><blockquote v-if="idea.reply"><strong>管理员回复：</strong>{{ idea.reply.content }}</blockquote><form v-if="isAdmin" class="player-suggestion-reply" @submit.prevent="saveReply(idea)"><input v-model="replyDraft[idea.id]" :placeholder="idea.reply?.content || '回复这条建议（每条建议仅一条回复，可修改）'" /><button class="secondary-button" type="submit">{{ idea.reply ? '更新回复' : '回复' }}</button></form></li></ol>
      <p v-else class="player-suggestion-empty">这个分区还没有建议，欢迎留下第一条。</p>
    </section>

    <ConfirmDialog :open="Boolean(pendingDelete)" :title="pendingDelete?.title || ''" :description="pendingDelete?.description || ''" confirm-label="确认删除" :busy="deleteBusy" @cancel="pendingDelete = null" @confirm="confirmDelete" />
  </section>
</template>
