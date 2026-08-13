// @vitest-environment jsdom
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { reactive } from 'vue'
import WeaponEditorDialog from '@/features/skin-forge/components/WeaponEditorDialog.vue'
import { createWeapon } from '@/types/skin-forge'

describe('WeaponEditorDialog', () => {
  it('提交响应式武器时 emits a cloneable plain payload', async () => {
    const weapon = reactive({
      ...createWeapon(7),
      stickers: [{ id: 101, schema: 0, offsetX: 0, offsetY: 0, wear: 0, scale: 1, rotation: 0 }],
    })
    const wrapper = mount(WeaponEditorDialog, {
      props: { weapon, weaponName: 'AK-47', team: 'ct' },
      global: { stubs: { CatalogGrid: true } },
    })

    await wrapper.get('[data-testid="weapon-editor-save"]').trigger('click')
    const payload = wrapper.emitted('save')?.[0]?.[0]

    expect(payload).toEqual(weapon)
    expect(payload).not.toBe(weapon)
    expect(() => structuredClone(payload)).not.toThrow()
  })
})
