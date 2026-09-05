import { invoke } from '@tauri-apps/api/core'

import type { InventorySimulatorInstallResult, InventorySimulatorRemoveResult, InventorySimulatorServiceStatus, InventorySimulatorStatus } from '@/types/inventory-simulator'

export const getInventorySimulatorStatus = (rootPath: string) =>
  invoke<InventorySimulatorStatus>('inventory_simulator_get_status', { rootPath })

export const installInventorySimulator = (rootPath: string) =>
  invoke<InventorySimulatorInstallResult>('inventory_simulator_install', { rootPath })

export const removeInventorySimulator = (rootPath: string) =>
  invoke<InventorySimulatorRemoveResult>('inventory_simulator_remove', { rootPath })

export const openInventoryWorkshop = () =>
  invoke<void>('inventory_simulator_open_workshop')

export const checkInventorySimulatorService = () =>
  invoke<InventorySimulatorServiceStatus>('inventory_simulator_check_service')
