import { reactive } from 'vue'

export const quickSupportState = reactive({
  busy: false,
  status: '待命，等待你的问题',
  stop: undefined as (() => void) | undefined,
})

export function updateQuickSupportState(patch: Partial<typeof quickSupportState>) {
  Object.assign(quickSupportState, patch)
}
