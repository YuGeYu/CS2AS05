import { invoke } from '@tauri-apps/api/core'
export const getBotChatConfig = (rootPath: string) => invoke<string>('get_bot_chat_config', { rootPath })
export const setBotChatConfig = (rootPath: string, content: string) => invoke<string>('set_bot_chat_config', { rootPath, content })
