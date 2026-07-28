import type { ToastMessage } from '@/types/cs2'

export interface DispatchedToastMessage extends ToastMessage { durationMs?: number }

export function dispatchToast(message: DispatchedToastMessage) {
  window.dispatchEvent(new CustomEvent<DispatchedToastMessage>('cs2as:toast', { detail: message }))
}
