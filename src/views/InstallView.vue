<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { AlertTriangle, Bug, Database, FolderOpen, LogIn, LogOut, Power, RefreshCw, Send, ShieldCheck, TerminalSquare, Trash2, Wrench, X } from 'lucide-vue-next'

import SupportActions from '@/components/SupportActions.vue'
import AppearanceSettingsDrawer from '@/components/AppearanceSettingsDrawer.vue'
import { appConfig } from '@/config/app'
import { useCs2Store } from '@/stores/cs2'
import { clearAssistantData, getAssistantAccount, getAssistantPreferences, loginAssistant, logoutAssistant, openFaultIdeaPage, setAssistantAutostart, submitFaultReport, type AssistantAccount } from '@/services/tauri/support'

const store = useCs2Store()
const uninstallConfirmOpen = ref(false)
const clearConfirmOpen = ref(false)
const faultOpen = ref(false)
const autostartEnabled = ref(false)
const preferenceBusy = ref(false)
const maintenanceState = ref('')
const faultDetails = ref('')
const faultSubmitting = ref(false)
const faultState = ref('')
const account = ref<AssistantAccount>({ username: null, loggedIn: false })
const accountOpen = ref(false)
const accountUsername = ref('')
const accountPassword = ref('')
const accountState = ref('')
const accountBusy = ref(false)
const lastTicketId = ref('')
const appearanceOpen = ref(false)
const keepBackup = ref(false)

const installBlocked = computed(() => !store.selectedRoot || store.cs2Running || store.busy)
const installLabel = computed(() => {
  if (store.busy) return '处理中...'
  if (store.environment?.baseEnvironmentReady) return '覆盖更新定制插件包'
  return '安装定制插件包'
})
const installHint = computed(() => {
  if (!store.selectedRoot) return '选择 Counter-Strike Global Offensive 目录后才能安装。'
  if (store.cs2Running) return '检测到 CS2 正在运行，请退出游戏后再继续。'
  if (store.environment?.baseEnvironmentReady) return `已检测到插件，可直接覆盖更新到 ${appConfig.appVersion} 定制包。`
  return '将安装基于 CS2-Bot-Improver v1.4.4 的定制资源包。'
})
const packageState = computed(() => store.environment?.baseEnvironmentReady ? '已安装' : '待安装')
const processLabel = computed(() => ({
  checking: '检测中',
  running: '运行中',
  stopped: '未运行',
  unknown: '检测失败',
}[store.cs2ProcessState]))
const processVisualState = computed(() => ({
  checking: 'warn',
  running: 'danger',
  stopped: 'ready',
  unknown: 'warn',
}[store.cs2ProcessState]))

async function browse() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const result = await open({ directory: true, multiple: false, title: '选择 CS2 游戏目录' })
  if (typeof result === 'string') await store.selectRoot(result)
}

async function install() {
  try {
    await store.install(keepBackup.value)
  } catch {
    // The store exposes the actionable failure in the toast and diagnostics.
  } finally {
    keepBackup.value = false
  }
}

async function openPanel() {
  try {
    await store.openPanel()
  } catch {
    // The store exposes the actionable failure in the toast and diagnostics.
  }
}

async function confirmUninstall() {
  uninstallConfirmOpen.value = false
  try {
    await store.uninstall()
  } catch {
    // The store exposes the actionable failure in the toast and diagnostics.
  }
}

async function toggleAutostart(event: Event) {
  const enabled = (event.target as HTMLInputElement).checked
  preferenceBusy.value = true
  maintenanceState.value = ''
  try {
    autostartEnabled.value = (await setAssistantAutostart(enabled)).autostartEnabled
    maintenanceState.value = enabled ? '已开启开机启动。' : '已关闭开机启动。'
  } catch (error) {
    autostartEnabled.value = !enabled
    maintenanceState.value = error instanceof Error ? error.message : String(error)
  } finally {
    preferenceBusy.value = false
  }
}

async function confirmClearData() {
  clearConfirmOpen.value = false
  preferenceBusy.value = true
  try {
    const result = await clearAssistantData()
    localStorage.clear()
    maintenanceState.value = result.message
  } catch (error) {
    maintenanceState.value = error instanceof Error ? error.message : String(error)
  } finally {
    preferenceBusy.value = false
  }
}

function openFaultDialog() {
  faultState.value = ''
  faultOpen.value = true
}

function openAccountDialog() {
  accountState.value = ''
  accountPassword.value = ''
  accountUsername.value = account.value.username || ''
  accountOpen.value = true
}

async function submitAccountLogin() {
  if (!accountUsername.value.trim() || !accountPassword.value || accountBusy.value) return
  accountBusy.value = true
  accountState.value = ''
  try {
    account.value = await loginAssistant(accountUsername.value.trim(), accountPassword.value)
    accountPassword.value = ''
    accountState.value = '官网登录成功，之后提交故障会沿用这个账号。'
  } catch (error) {
    accountState.value = error instanceof Error ? error.message : String(error)
  } finally {
    accountBusy.value = false
  }
}

async function signOutAccount() {
  accountBusy.value = true
  try {
    account.value = await logoutAssistant()
    accountState.value = '已退出助手账号。下次提交故障时会重新一键注册。'
  } catch (error) {
    accountState.value = error instanceof Error ? error.message : String(error)
  } finally {
    accountBusy.value = false
  }
}

async function submitFault() {
  if (faultDetails.value.trim().length < 10 || faultSubmitting.value) return
  faultSubmitting.value = true
  faultState.value = ''
  try {
    const result = await submitFaultReport(faultDetails.value.trim(), store.selectedRoot || undefined)
    faultState.value = result.message
    lastTicketId.value = result.ticketId
    account.value = { username: result.accountUsername, loggedIn: true }
    faultDetails.value = ''
  } catch (error) {
    faultState.value = error instanceof Error ? error.message : String(error)
  } finally {
    faultSubmitting.value = false
  }
}

onMounted(async () => {
  try {
    autostartEnabled.value = (await getAssistantPreferences()).autostartEnabled
    account.value = await getAssistantAccount()
  } catch {
    maintenanceState.value = '开机启动状态暂时无法读取。'
  }
})

</script>

<template>
  <main class="installer-page">
    <section class="installer-shell" aria-labelledby="page-title">
      <header class="installer-header">
        <div class="brand-mark" aria-hidden="true"><Wrench :size="24" /></div>
        <div>
          <p class="overline">CS2-BOT-IMPROVER</p>
          <h1 id="page-title">CS2 人机增强助手</h1>
          <p class="subtitle">从环境确认到故障反馈，把每一次运行准备妥当</p>
        </div>
        <div class="installer-header-actions"><span class="version-label">{{ appConfig.appVersion }}</span><button class="secondary-button appearance-trigger" type="button" aria-label="打开主题设置" @click="appearanceOpen = true"><Wrench :size="17" /><span>主题设置</span></button></div>
      </header>

      <SupportActions />

      <section class="directory-section" aria-labelledby="directory-title">
        <div class="section-heading">
          <div>
            <p class="overline">安装位置</p>
            <h2 id="directory-title">CS2 游戏目录</h2>
          </div>
          <button class="icon-button" type="button" title="重新扫描 CS2 目录" aria-label="重新扫描 CS2 目录" :disabled="store.busy" @click="store.scanRoots">
            <RefreshCw :size="18" :class="{ spinning: store.busy }" />
          </button>
        </div>

        <div class="directory-control">
          <label class="sr-only" for="selected-directory">CS2 游戏目录</label>
          <input id="selected-directory" :value="store.selectedRoot" readonly placeholder="尚未选择目录" />
          <button class="secondary-button" type="button" :disabled="store.busy" @click="browse">
            <FolderOpen :size="18" />
            <span>选择目录</span>
          </button>
        </div>
        <p v-if="store.candidates.length" class="candidate-note">已找到 {{ store.candidates.length }} 个候选目录。</p>
      </section>

      <section class="status-grid" aria-label="当前状态">
        <div class="status-item" :data-state="store.selectedRoot ? 'ready' : 'warn'">
          <span>目录</span><strong>{{ store.selectedRoot ? '已选择' : '未选择' }}</strong>
        </div>
        <div class="status-item" :data-state="processVisualState">
          <span>CS2</span><strong>{{ processLabel }}</strong>
        </div>
        <div class="status-item" :data-state="store.environment?.baseEnvironmentReady ? 'ready' : 'warn'">
          <span>插件</span><strong>{{ packageState }}</strong>
        </div>
      </section>

      <section class="assistant-settings" aria-labelledby="assistant-settings-title">
        <div class="section-heading">
          <div>
            <p class="overline">助手设置</p>
            <h2 id="assistant-settings-title">随时待命，保持轻盈</h2>
          </div>
        </div>
        <div class="setting-row">
          <span class="setting-icon" aria-hidden="true"><Power :size="20" /></span>
          <div class="setting-copy">
            <strong>开机启动</strong>
            <span>登录 Windows 后自动启动助手，不需要管理员权限。</span>
          </div>
          <label class="toggle-switch" :aria-label="autostartEnabled ? '关闭开机启动' : '开启开机启动'">
            <input type="checkbox" :checked="autostartEnabled" :disabled="preferenceBusy" @change="toggleAutostart" />
            <span aria-hidden="true"></span>
          </label>
        </div>
        <p v-if="maintenanceState" class="maintenance-state" aria-live="polite">{{ maintenanceState }}</p>
      </section>

      <section class="install-section" aria-labelledby="install-title">
        <div>
          <p class="overline">定制资源包</p>
          <h2 id="install-title">{{ appConfig.appVersion }} 定制资源包</h2>
          <p>基于上游 CS2-Bot-Improver v1.4.4，保留上游完整能力并叠加本项目定制。</p>
          <p>{{ installHint }}</p>
        </div>
        <label class="backup-choice"><input v-model="keepBackup" type="checkbox" :disabled="installBlocked" />本次操作保留写前备份</label>
        <button class="primary-button" type="button" :disabled="installBlocked" @click="install">
          <ShieldCheck :size="20" />
          <span>{{ installLabel }}</span>
        </button>
      </section>

      <section class="panel-section" aria-labelledby="panel-title">
        <div>
          <p class="overline">高级兼容入口</p>
          <h2 id="panel-title">原版 Panel v1.4.4</h2>
          <p>融合功能异常时可临时打开原版工具；请勿让两个面板同时写入同一目录。</p>
        </div>
        <button class="secondary-button panel-button" type="button" :disabled="store.busy" @click="openPanel">
          <TerminalSquare :size="19" />
          <span>打开原版 Panel</span>
        </button>
      </section>

      <section class="utility-section">
        <div class="utility-heading">
          <div><p class="overline">维护与求助</p><h2>问题留在这里，信息一次带齐</h2></div>
        </div>
          <div class="maintenance-actions">
          <button class="secondary-button" type="button" :disabled="preferenceBusy" @click="clearConfirmOpen = true"><Database :size="18" /><span>清除数据</span></button>
          <button class="primary-button" type="button" @click="openFaultDialog"><Bug :size="18" /><span>提交故障</span></button>
          <button class="secondary-button" type="button" @click="openAccountDialog"><LogIn :size="18" /><span>{{ account.loggedIn ? `已登录：${account.username}` : '登录官网账号' }}</span></button>
          <button class="danger-button" type="button" :disabled="!store.selectedRoot || store.busy" @click="uninstallConfirmOpen = true"><Trash2 :size="17" /><span>卸载插件</span></button>
        </div>
      </section>
    </section>

    <Teleport to="body">
      <div v-if="uninstallConfirmOpen" class="modal-backdrop" role="presentation">
        <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="uninstall-title">
          <h2 id="uninstall-title">卸载定制插件包？</h2>
          <p>将删除由定制资源包安装的插件目录和配置文件。CS2 核心文件与 gameinfo.gi 不会删除。</p>
          <div class="dialog-actions">
            <button class="secondary-button" type="button" @click="uninstallConfirmOpen = false">取消</button>
            <button class="danger-button" type="button" @click="confirmUninstall"><Trash2 :size="17" /><span>确认卸载</span></button>
          </div>
        </section>
      </div>
      <div v-if="clearConfirmOpen" class="modal-backdrop" role="presentation" @click.self="clearConfirmOpen = false">
        <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="clear-data-title">
          <div class="dialog-icon danger" aria-hidden="true"><AlertTriangle :size="22" /></div>
          <h2 id="clear-data-title">清除助手数据？</h2>
          <p>将清除助手缓存、运行日志和界面偏好。CS2 文件、已安装插件、Demo 与复盘数据库不会删除。完成后部分设置需要重新选择。</p>
          <div class="dialog-actions"><button class="secondary-button" type="button" @click="clearConfirmOpen = false">取消</button><button class="danger-button" type="button" @click="confirmClearData"><Database :size="17" /><span>确认清除</span></button></div>
        </section>
      </div>
      <div v-if="faultOpen" class="modal-backdrop" role="presentation" @click.self="faultOpen = false">
        <form class="fault-dialog" role="dialog" aria-modal="true" aria-labelledby="fault-title" @submit.prevent="submitFault">
          <header><div><p class="overline">故障工单</p><h2 id="fault-title">告诉我们发生了什么</h2></div><button class="icon-button" type="button" aria-label="关闭故障提交" @click="faultOpen = false"><X :size="19" /></button></header>
          <p>请描述出现问题前后的操作和你看到的现象。助手会附带经过脱敏与裁剪的诊断日志，不上传 Demo、游戏文件或皮肤配置。</p>
          <label for="fault-details">故障详情</label>
          <textarea id="fault-details" v-model="faultDetails" minlength="10" maxlength="4000" rows="8" placeholder="例如：我在安装完成后启动 CS2，进入本地地图时……" autofocus />
          <div class="fault-meta"><span><ShieldCheck :size="15" />诊断日志将随工单提交</span><span>{{ faultDetails.length }}/4000</span></div>
          <p v-if="faultState" class="fault-state" aria-live="polite">{{ faultState }}</p>
          <button v-if="lastTicketId" class="text-button" type="button" @click="openFaultIdeaPage(lastTicketId)">查看已提交故障</button>
          <div class="dialog-actions"><button class="secondary-button" type="button" @click="faultOpen = false">稍后再说</button><button class="primary-button" type="submit" :disabled="faultDetails.trim().length < 10 || faultSubmitting"><Send :size="17" /><span>{{ faultSubmitting ? '正在提交…' : '提交故障单' }}</span></button></div>
        </form>
      </div>
      <div v-if="accountOpen" class="modal-backdrop" role="presentation" @click.self="accountOpen = false">
        <form class="confirm-dialog account-dialog" role="dialog" aria-modal="true" aria-labelledby="account-title" @submit.prevent="submitAccountLogin">
          <header><div><p class="overline">官网账号</p><h2 id="account-title">登录 CS2AS</h2></div><button class="icon-button" type="button" aria-label="关闭账号登录" @click="accountOpen = false"><X :size="19" /></button></header>
          <p>登录后提交故障会归档到你的账号名下；没有账号的玩家，首次提交时会自动生成一个好记的账号。</p>
          <label for="account-username">用户名</label>
          <input id="account-username" v-model="accountUsername" autocomplete="username" maxlength="20" placeholder="输入官网用户名" />
          <label for="account-password">密码</label>
          <input id="account-password" v-model="accountPassword" type="password" autocomplete="current-password" maxlength="80" placeholder="输入官网密码" />
          <p v-if="accountState" class="fault-state" aria-live="polite">{{ accountState }}</p>
          <div class="dialog-actions">
            <button v-if="account.loggedIn" class="danger-button" type="button" :disabled="accountBusy" @click="signOutAccount"><LogOut :size="17" /><span>退出登录</span></button>
            <button class="primary-button" type="submit" :disabled="accountBusy || !accountUsername.trim() || !accountPassword">{{ accountBusy ? '正在登录…' : '登录官网账号' }}</button>
          </div>
        </form>
      </div>
    </Teleport>
  </main>
  <AppearanceSettingsDrawer :open="appearanceOpen" @close="appearanceOpen = false" />
</template>
