<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { RotateCcw, Route } from 'lucide-vue-next'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import { getMapRotationDefault, resetMapRotationDefault, setMapRotationDefault } from '@/services/tauri/mapRotation'
import type { MapRotationDefault } from '@/types/mapRotation'

const props = defineProps<{ rootPath: string; cs2Running: boolean; ready: boolean }>()
const state = ref<MapRotationDefault | null>(null)
const busy = ref(false)
const error = ref('')
const disabled = computed(() => !props.rootPath || !props.ready || props.cs2Running || busy.value || state.value?.writable === false)

async function load() { if (!props.rootPath) { state.value = null; return }; try { state.value = await getMapRotationDefault(props.rootPath); error.value = '' } catch (e) { error.value = String(e) } }
async function change(enabled: boolean) { busy.value = true; error.value = ''; try { state.value = await setMapRotationDefault(props.rootPath, enabled) } catch (e) { error.value = String(e) } finally { busy.value = false } }
async function reset() { busy.value = true; error.value = ''; try { state.value = await resetMapRotationDefault(props.rootPath) } catch (e) { error.value = String(e) } finally { busy.value = false } }
watch(() => [props.rootPath, props.cs2Running], () => void load())
onMounted(() => void load())
</script>

<template>
  <div class="control-group map-rotation-control">
    <div class="map-rotation-heading"><div class="map-rotation-copy"><h2><Route :size="18" aria-hidden="true" />自动换图默认状态</h2><p>{{ state?.enabled ? '下一次插件载入时自动换图' : '下一次插件载入时保持当前地图' }}</p><small>只影响下一次 MapRotation 载入；游戏运行中请使用 lbtv_map_rotation 0|1。</small><code v-if="state" :title="state.configPath">{{ state.configPath }}</code></div><button v-if="state?.source === 'fallback'" class="icon-button" type="button" title="恢复默认开启" aria-label="恢复默认开启" :disabled="disabled" @click="reset"><RotateCcw :size="17" /></button></div>
    <div class="map-rotation-toggle-row"><ToggleSwitch :model-value="state?.enabled ?? true" label="启用自动换图" description="默认开启；修改会在下一次插件载入时生效。" :disabled="disabled" @update:model-value="change" /></div>
    <p v-if="props.cs2Running" class="warning-note">请先退出 CS2，再修改下一次载入默认值。</p>
    <p v-if="state?.warning" class="warning-note" role="status">{{ state.warning }}</p>
    <p v-if="state" class="map-rotation-evidence" role="status">回读 {{ state.readBackEnabled ? (state.enabled ? 'enabled=1' : 'enabled=0') : '未建立配置' }} · SHA-256 {{ state.configSha256 || '--' }} · {{ state.loadSemantics }}</p>
    <p v-if="error" class="inline-error" role="alert">{{ error }}</p>
  </div>
</template>
