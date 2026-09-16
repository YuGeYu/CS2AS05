<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { AlertTriangle, BookOpen, CheckCircle2, CopyPlus, FileCode2, LoaderCircle, Pencil, Save, Send, Trash2, X } from 'lucide-vue-next'
import { applyBotProfile, createBotProfile, deleteBotProfile, getBotWorkshopState, listBotProfiles, openBotProfile, renameBotProfile, saveBotProfile, type BotProfileDocument, type BotProfileSummary, type BotToolState } from '@/services/tauri/bot-difficulty'
import { dismissBotProfileGuide, isBotProfileGuideSeen } from '@/services/tauri/support'

const props = defineProps<{ open: boolean; rootPath: string }>()
const emit = defineEmits<{ close: [] }>()
const profiles = ref<BotProfileSummary[]>([])
const selected = ref<BotProfileSummary | null>(null)
const document = ref<BotProfileDocument | null>(null)
const editor = ref('')
const customName = ref('')
const renameName = ref('')
const deleteTarget = ref<BotProfileSummary | null>(null)
const toolState = ref<BotToolState | null>(null)
const busy = ref(false)
const action = ref('')
const error = ref('')
const notice = ref('')
const titleRef = ref<HTMLElement | null>(null)
const requestSequence = ref(0)
const guideVisible = ref(false)
const guideReady = ref(false)
const guideRemaining = ref(2)
let guideTimer: ReturnType<typeof setInterval> | undefined
const canWrite = computed(() => toolState.value?.status === 'ready' && !!props.rootPath)
const isCustom = computed(() => selected.value?.source === 'custom')
const changed = computed(() => document.value?.text !== undefined && editor.value !== document.value.text)

function friendlyError(cause: unknown) {
  const raw = String(cause)
  if (raw.includes('BOT_WORKSHOP_DB_INVALID')) return '档案内容格式不完整。请检查最近修改的 key、引号、End 区块和不可见字符，再保存。'
  if (raw.includes('BOT_WORKSHOP_REPACK_FAILED')) return 'VPK 重打包失败，原档案没有被覆盖。请撤销最近修改后重试；若仍失败，请检查安全软件或文件占用。'
  if (raw.includes('BOT_WORKSHOP_VPK_ROUNDTRIP_MISMATCH')) return '回读校验未通过，原档案没有被覆盖。请重新打开档案后再编辑。'
  if (raw.includes('BOT_WORKSHOP_WORKSPACE_UNWRITABLE')) return '临时工作目录无法写入。请关闭 CS2/Steam 相关工具后重试，并检查安全软件拦截与磁盘空间。'
  if (raw.includes('BOT_WORKSHOP_EXTRACT_FAILED')) return 'VPK 内的 botprofile.db 提取失败。请关闭 CS2/Steam 相关工具后重试；连续失败时提交诊断信息。'
  if (raw.includes('BOT_WORKSHOP_SOURCE_CHANGED')) return '档案已在其他操作中发生变化，请重新打开后再保存。'
  return raw
}

async function load(preferred?: string) {
  const sequence = ++requestSequence.value
  busy.value = true; error.value = ''; notice.value = ''
  try {
    const [result, state] = await Promise.all([listBotProfiles(props.rootPath), getBotWorkshopState()])
    if (sequence !== requestSequence.value) return
    profiles.value = result.profiles; toolState.value = state
    const next = result.profiles.find(p => p.id === preferred) ?? result.profiles.find(p => p.active) ?? result.profiles[1] ?? result.profiles[0] ?? null
    if (next) await selectProfile(next, sequence)
  } catch (cause) { if (sequence === requestSequence.value) error.value = friendlyError(cause) }
  finally { if (sequence === requestSequence.value) busy.value = false }
}

async function selectProfile(profile: BotProfileSummary, sequence = ++requestSequence.value) {
  selected.value = profile; document.value = null; editor.value = ''; renameName.value = profile.name; error.value = ''; notice.value = ''; busy.value = true
  try {
    const opened = await openBotProfile(props.rootPath, profile.id)
    if (sequence !== requestSequence.value) return
    document.value = opened; editor.value = opened.text ?? ''
  } catch (cause) { if (sequence === requestSequence.value) error.value = friendlyError(cause) }
  finally { if (sequence === requestSequence.value) busy.value = false }
}

async function createFromBuiltin() {
  if (!selected.value || selected.value.source !== 'builtin') return
  action.value = '正在创建自定义档案…'; error.value = ''
  try { const result = await createBotProfile(props.rootPath, { baseProfileId: selected.value.id, name: customName.value.trim() || `${selected.value.name} 自定义` }); customName.value = ''; notice.value = result.message; await load(result.profile.id) }
  catch (cause) { error.value = friendlyError(cause) } finally { action.value = '' }
}

async function save() {
  if (!document.value || !isCustom.value || !changed.value || !selected.value) return
  action.value = '正在回写 VPK 并进行回读校验…'; error.value = ''
  try { const result = await saveBotProfile(props.rootPath, { profileId: selected.value.id, text: editor.value, expectedDbSha256: document.value.profile.dbSha256 ?? '' }); notice.value = result.message + (result.backupPath ? ` 已备份：${result.backupPath}` : ''); await load(result.profile.id) }
  catch (cause) { error.value = friendlyError(cause) } finally { action.value = '' }
}

async function apply() {
  if (!selected.value || !isCustom.value || changed.value) return
  action.value = '正在应用到当前 BOT 模式…'; error.value = ''
  try { const result = await applyBotProfile(props.rootPath, selected.value.id); notice.value = result.message + (result.backupPath ? ` 已备份当前档案：${result.backupPath}` : ''); await load(result.profile.id) }
  catch (cause) { error.value = friendlyError(cause) } finally { action.value = '' }
}

async function renameProfile() {
  if (!selected.value || !isCustom.value || changed.value || !renameName.value.trim()) return
  action.value = '正在更新档案名称…'; error.value = ''
  try { const result = await renameBotProfile({ profileId: selected.value.id, name: renameName.value }); notice.value = result.message; await load(result.profile.id) }
  catch (cause) { error.value = friendlyError(cause) } finally { action.value = '' }
}

function requestDelete(profile: BotProfileSummary) {
  if (action.value || (selected.value?.id === profile.id && changed.value)) { error.value = '当前档案有未保存修改，请先保存或放弃修改。'; return }
  deleteTarget.value = profile
}

async function confirmDelete() {
  const target = deleteTarget.value
  if (!target) return
  deleteTarget.value = null; action.value = '正在备份并归档自定义档案…'; error.value = ''
  try { const result = await deleteBotProfile(props.rootPath, target.id); notice.value = result.message + (result.backupPath ? ` 已备份活动档案：${result.backupPath}` : ''); await load() }
  catch (cause) { error.value = friendlyError(cause) } finally { action.value = '' }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && !action.value) {
    if (guideVisible.value) return
    if (deleteTarget.value) deleteTarget.value = null
    else emit('close')
  }
}

async function openGuide(force = false) {
  guideVisible.value = true; guideReady.value = false; guideRemaining.value = 2
  if (guideTimer) clearInterval(guideTimer)
  try {
    if (!force && await isBotProfileGuideSeen()) { guideVisible.value = false; return }
    const startedAt = Date.now()
    guideTimer = setInterval(() => {
      const remaining = Math.max(0, 2 - Math.ceil((Date.now() - startedAt) / 1000))
      guideRemaining.value = remaining
      if (remaining === 0) { guideReady.value = true; if (guideTimer) clearInterval(guideTimer) }
    }, 100)
  } catch (cause) { error.value = cause instanceof Error ? cause.message : '无法读取说明页状态' }
}

async function finishGuide() {
  if (!guideReady.value) return
  try { await dismissBotProfileGuide(); guideVisible.value = false }
  catch (cause) { error.value = cause instanceof Error ? cause.message : '无法保存说明页阅读状态' }
}

watch(() => props.open, async open => { if (!open) { if (guideTimer) clearInterval(guideTimer); return }; await nextTick(); titleRef.value?.focus(); void openGuide(); void load() })
</script>

<!-- 只影响 BOT 模式，在线模式不会使用这些文件。 -->
<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop bot-workbench-backdrop" @click.self="!guideVisible && emit('close')" @keydown="onKeydown">
      <section class="bot-workbench" role="dialog" aria-modal="true" aria-labelledby="bot-workbench-title">
        <header class="bot-workbench-header"><div><p class="overline">BOT / LOCAL ONLY</p><h2 id="bot-workbench-title" ref="titleRef" tabindex="-1">人机强度工坊</h2><p>从 VPK 提取并编辑 botprofile.db。在线模式和 gameinfo.gi 不受影响。</p></div><div class="bot-workbench-header-actions"><button class="secondary-button" type="button" :disabled="!!action || guideVisible" @click="openGuide(true)"><BookOpen :size="16" />查看强度教程</button><button class="icon-button" type="button" aria-label="关闭人机强度工坊" title="关闭" :disabled="!!action || guideVisible" @click="emit('close')"><X :size="18" /></button></div></header>
        <div class="bot-workbench-body">
          <aside class="bot-profile-list" aria-label="BOT 强度档案">
            <div class="bot-tool-note"><FileCode2 :size="16" /><span>{{ toolState?.status === 'ready' ? `VPKEdit 已就绪 · ${toolState.version}` : toolState?.detail ?? '正在检查 VPKEdit' }}</span></div>
            <div v-for="profile in profiles" :key="profile.id" class="bot-profile-row">
              <button class="bot-profile-item" :class="{ active: selected?.id === profile.id }" type="button" :disabled="!!action" @click="selectProfile(profile)"><span><strong>{{ profile.name }}</strong><small>{{ profile.source === 'builtin' ? `内置 ${profile.baseDifficulty} · 只读` : `自定义 · 源自 ${profile.baseDifficulty}` }}</small></span><CheckCircle2 v-if="profile.active" :size="17" aria-label="当前使用" /></button>
              <div v-if="profile.source === 'custom'" class="bot-profile-tools"><button class="icon-button" type="button" :disabled="!!action" :aria-label="`重命名${profile.name}`" title="重命名" @click="selectProfile(profile)"><Pencil :size="15" /></button><button class="icon-button bot-profile-delete" type="button" :disabled="!!action" :aria-label="`删除${profile.name}`" title="删除" @click="requestDelete(profile)"><Trash2 :size="15" /></button></div>
            </div>
          </aside>
          <main class="bot-editor">
            <div v-if="busy" class="bot-state"><LoaderCircle class="spin" :size="22" />正在读取档案…</div>
            <div v-else-if="error" class="bot-state bot-error" role="alert"><AlertTriangle :size="22" /><div><strong>操作未完成</strong><p>{{ error }}</p></div></div>
            <template v-else-if="document && selected">
              <div class="bot-editor-heading"><div><strong>{{ selected.name }}</strong><span>{{ selected.readOnly ? '内置基线只能阅读，请复制为自定义档案后编辑。' : changed ? '有未保存改动' : '已与当前自定义 VPK 同步' }}</span></div><span class="bot-hash">DB {{ document.profile.dbSha256?.slice(0, 12) ?? '读取中' }}</span></div>
              <textarea v-model="editor" class="bot-db-editor" :readonly="selected.readOnly || !!action" aria-label="botprofile.db 编辑器" spellcheck="false" />
              <div v-if="selected.source === 'builtin'" class="bot-create-row"><label><span>自定义档案名称</span><input v-model="customName" maxlength="48" placeholder="例如：职业反应训练" :disabled="!canWrite || !!action" /></label><button class="primary-button" type="button" :disabled="!canWrite || !!action" @click="createFromBuiltin"><CopyPlus :size="17" />复制为自定义档案</button></div>
              <div v-else class="bot-custom-actions"><div class="bot-rename-row"><label><span>档案名称</span><input v-model="renameName" maxlength="48" :disabled="!!action || changed" /><button class="secondary-button" type="button" :disabled="!!action || changed || !renameName.trim() || renameName.trim() === selected.name" @click="renameProfile"><Pencil :size="16" />重命名</button></label></div><div class="bot-action-row"><button class="secondary-button" type="button" :disabled="!changed || !!action" @click="editor = document?.text ?? ''">放弃未保存改动</button><button class="primary-button" type="button" :disabled="!changed || !!action" @click="save"><Save :size="17" />保存到 VPK</button><button class="primary-button" type="button" :disabled="changed || !!action" @click="apply"><Send :size="17" />应用到 BOT 模式</button></div></div>
            </template>
            <div v-else class="bot-state bot-empty">请选择一个强度档案。</div>
          </main>
        </div>
        <footer class="bot-workbench-footer"><span aria-live="polite">{{ action || notice || '保存会创建自定义 VPK 备份；应用前请退出 CS2。' }}</span><button class="secondary-button" type="button" :disabled="!!action || guideVisible" @click="emit('close')">关闭</button></footer>
      </section>
      <section v-if="guideVisible" class="bot-profile-guide" role="dialog" aria-modal="true" aria-labelledby="bot-profile-guide-title">
        <div class="bot-profile-guide-kicker"><FileCode2 :size="16" />首次进入 · 阅读说明</div>
        <h3 id="bot-profile-guide-title">让 CS2 BOT 更强：botprofile.db 实战教程</h3>
        <p class="bot-profile-guide-lead">botprofile.db 可以理解成 BOT 的“基础人格与能力表”。它影响瞄准、反应、攻击倾向和团队协作，但不是唯一开关；CS2 的行为文件、武器偏好、地图导航和控制台设置会一起决定最终表现。</p>
        <div class="bot-profile-guide-grid"><article><strong>01 · 先做安全备份</strong><p>只在助手启动的本地 BOT / <code>-insecure</code> 对局中使用。关闭 CS2 后再保存；工坊会在回写 VPK 前保留备份，出现异常时可恢复。</p></article><article><strong>02 · 从关键参数开始</strong><p><code>Skill</code> 控制基础水平；<code>ReactionTime</code>、<code>AttackDelay</code> 越小通常反应越快；<code>Aggression</code> 影响主动找人和压制；<code>Teamwork</code> 影响跟随、补枪与协同。一次只改少量参数，方便比较。</p></article><article><strong>03 · 别把数值拉满</strong><p>极端值可能造成“锁头感”、不自然的瞬间反应或行为异常。建议从小幅调整开始，打一两回合观察，再逐步迭代。</p></article><article><strong>04 · 还要检查行为文件</strong><p>武器购买偏好、地图 <code>.nav</code>、<code>bt_config.kv3</code> / <code>bt_default.kv3</code> 和服务器 cvar 都可能覆盖或削弱 botprofile.db 的效果。</p></article><article><strong>05 · 正确的测试方法</strong><p>应用档案后重启本地 BOT 对局；固定地图、人数和难度，分别观察枪法、转身、进攻路线、补枪和残局决策，避免只凭一局下结论。</p></article><article><strong>06 · 当前版本说明</strong><p>不同 CS2 更新可能改变参数含义。教程经验来自小黑盒玩家整理，具体效果以当前版本实测为准；在线模式不会使用这些本地修改。</p></article></div>
        <div class="bot-profile-guide-bug"><strong>已知问题，请先知悉</strong><p>BOT 强度功能目前仍有 BUG，部分异常暂时无法稳定定位，无法承诺一次修复完整。我们会根据日志和实测持续缩小范围、逐步修复，给你带来不便敬请见谅。</p></div>
        <p class="bot-profile-guide-source">说明参考：小黑盒《如何让CSGO的BOT更加强力（保姆级教程）》；其中关于 botprofile.db、Skill、Teamwork、武器偏好与行为文件的经验，作为玩家向阅读材料整理，具体效果以当前 CS2 版本实测为准。</p>
        <button class="primary-button bot-profile-guide-confirm" type="button" :disabled="!guideReady" @click="finishGuide">{{ guideReady ? '我已阅读，进入工坊' : `请阅读说明（${guideRemaining} 秒）` }}</button>
      </section>
      <div v-if="deleteTarget" class="bot-delete-confirm" role="alertdialog" aria-modal="true" aria-labelledby="bot-delete-title"><h3 id="bot-delete-title">删除自定义档案？</h3><p>“{{ deleteTarget.name }}”{{ deleteTarget.active ? ` 正在使用，删除后将恢复 ${deleteTarget.baseDifficulty} 内置 BOT 难度。` : ' 将移入可恢复归档。' }}</p><p>此操作需要 CS2 已退出，且不会删除其它档案。</p><div><button class="secondary-button" type="button" @click="deleteTarget = null">取消</button><button class="danger-button" type="button" @click="confirmDelete"><Trash2 :size="16" />确认删除</button></div></div>
    </div>
  </Teleport>
</template>
