import { convertFileSrc } from '@tauri-apps/api/core'
import { skinForgeCacheImage } from '@/services/tauri/skinForge'

const MAX_CONCURRENT = 4
const BUSY_DELAYS = [120, 300] as const
const TIMEOUT_MS = 15_000
const resolvedByUrl = new Map<string, string>()
const inFlight = new Map<string, Promise<string>>()
type Task = { url: string; priority: number; cancelled: boolean; run: () => void }
const queue: Task[] = []
let active = 0
function wait(delay: number) { return new Promise<void>(resolve => setTimeout(resolve, delay)) }
function isBusy(error: unknown) { return String(error).includes('[IMAGE_BUSY]') }
async function fetchCachedImage(url: string): Promise<string> {
  for (let attempt = 0; ; attempt += 1) {
    try {
      const result = await Promise.race([
        skinForgeCacheImage(url),
        new Promise<never>((_, reject) => setTimeout(() => reject(new Error('[IMAGE_TIMEOUT]')), TIMEOUT_MS)),
      ])
      return convertFileSrc(result.path)
    } catch (error) {
      const delay = BUSY_DELAYS[attempt]
      if (!isBusy(error) || delay === undefined) throw error
      await wait(delay)
    }
  }
}
function drainQueue() {
  while (active < MAX_CONCURRENT) {
    const index = queue.findIndex(task => !task.cancelled)
    if (index < 0) return
    const task = queue.splice(index, 1)[0]!
    active += 1
    task.run()
  }
}
export function loadForgeImage(url: string, options: { signal?: AbortSignal; priority?: 'eager' | 'lazy' } = {}): Promise<string> {
  const cached = resolvedByUrl.get(url)
  if (cached) return Promise.resolve(cached)
  const existing = inFlight.get(url)
  if (existing) return existing
  const promise = new Promise<string>((resolve, reject) => {
    const task: Task = { url, priority: options.priority === 'eager' ? 0 : 1, cancelled: false, run: () => {
      if (task.cancelled || options.signal?.aborted) { active -= 1; drainQueue(); reject(new DOMException('Aborted', 'AbortError')); return }
      void fetchCachedImage(url).then(value => { resolvedByUrl.set(url, value); resolve(value) }, reject).finally(() => { active -= 1; inFlight.delete(url); drainQueue() })
    } }
    queue.push(task)
    queue.sort((a, b) => a.priority - b.priority)
    options.signal?.addEventListener('abort', () => {
      task.cancelled = true
      if (queue.includes(task)) {
        queue.splice(queue.indexOf(task), 1)
        inFlight.delete(url)
        reject(new DOMException('Aborted', 'AbortError'))
        drainQueue()
      }
    }, { once: true })
    drainQueue()
  })
  inFlight.set(url, promise)
  return promise
}
export function retryForgeImage(url: string, options: { signal?: AbortSignal; priority?: 'eager' | 'lazy' } = {}): Promise<string> {
  return loadForgeImage(url, options)
}
export function resetForgeImageQueueForTests() { queue.splice(0); inFlight.clear(); resolvedByUrl.clear(); active = 0 }
export const forgeImageQueueLimits = { maxConcurrent: MAX_CONCURRENT, busyRetries: BUSY_DELAYS.length, timeoutMs: TIMEOUT_MS }
