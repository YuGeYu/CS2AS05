import type { ThreeStageController, StageSize } from '@/components/three/stage'
import type { IntroAcknowledgementCard } from './types'
import { buildPlaqueLayout, type PlaqueLayoutSlot } from '@/features/acknowledgement/layout'

export interface IntroSceneState {
  elapsedMs: number
  durationMs: number
  supporterLockMs: number
  loading: boolean
  cards: IntroAcknowledgementCard[]
  mode?: 'cinematic' | 'free-roam'
  selectedIndex?: number
}

export function introCardSignature(cards: IntroAcknowledgementCard[]) {
  return cards.map(item => `${item.id}:${item.updatedAt}`).join('|')
}

function fittedText(context: CanvasRenderingContext2D, value: string, maxWidth: number) {
  if (context.measureText(value).width <= maxWidth) return value
  let output = value
  while (output.length > 1 && context.measureText(`${output}...`).width > maxWidth) output = output.slice(0, -1)
  return `${output}...`
}

function easeInOut(value: number) {
  const t = Math.min(1, Math.max(0, value))
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2
}

export async function createIntroScene(canvas: HTMLCanvasElement, initialSize: StageSize, getState: () => IntroSceneState): Promise<ThreeStageController> {
  const THREE = await import('three')
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false, powerPreference: 'high-performance' })
  renderer.setClearColor(0xe8e0d0, 1)
  renderer.outputColorSpace = THREE.SRGBColorSpace
  renderer.shadowMap.enabled = true
  renderer.shadowMap.type = THREE.PCFShadowMap

  const scene = new THREE.Scene()
  scene.background = new THREE.Color(0xe8e0d0)
  scene.fog = new THREE.FogExp2(0x9aafa7, 0.022)
  const camera = new THREE.PerspectiveCamera(46, 1, 0.1, 100)
  const target = new THREE.Vector3()
  const geometries = new Set<InstanceType<typeof THREE.BufferGeometry>>()
  const materials = new Set<InstanceType<typeof THREE.Material>>()
  const textures = new Set<InstanceType<typeof THREE.Texture>>()
  const geometry = <T extends InstanceType<typeof THREE.BufferGeometry>>(value: T) => { geometries.add(value); return value }
  const material = <T extends InstanceType<typeof THREE.Material>>(value: T) => { materials.add(value); return value }

  scene.add(new THREE.HemisphereLight(0xfffbf0, 0x3b5653, 2.45))
  const moon = new THREE.DirectionalLight(0xfff3dc, 4.8)
  moon.position.set(7, 9, 7)
  moon.castShadow = true
  moon.shadow.mapSize.set(1024, 1024)
  scene.add(moon)
  const warm = new THREE.PointLight(0xe9a45f, 46, 32, 1.45)
  warm.position.set(0, 4.2, -9)
  scene.add(warm)

  const stone = material(new THREE.MeshStandardMaterial({ color: 0x9caea4, roughness: 0.84, metalness: 0.02 }))
  const darkStone = material(new THREE.MeshStandardMaterial({ color: 0x4b6867, roughness: 0.9 }))
  const bronze = material(new THREE.MeshStandardMaterial({ color: 0xb7834d, emissive: 0x3a2110, metalness: 0.35, roughness: 0.42 }))
  const jade = material(new THREE.MeshStandardMaterial({ color: 0x456a67, emissive: 0x102b2a, metalness: 0.08, roughness: 0.58 }))

  const floor = new THREE.Mesh(geometry(new THREE.CircleGeometry(18, 64)), darkStone)
  floor.rotation.x = -Math.PI / 2
  floor.receiveShadow = true
  scene.add(floor)
  const dais = new THREE.Mesh(geometry(new THREE.CylinderGeometry(8.2, 9.2, 0.32, 64)), stone)
  dais.position.y = 0.05
  dais.receiveShadow = true
  scene.add(dais)
  const innerRing = new THREE.Mesh(geometry(new THREE.TorusGeometry(6.4, 0.055, 10, 96)), bronze)
  innerRing.rotation.x = Math.PI / 2
  innerRing.position.y = 0.24
  scene.add(innerRing)
  const lightPool = new THREE.Mesh(geometry(new THREE.CircleGeometry(3.2, 64)), material(new THREE.MeshBasicMaterial({ color: 0x9bb7ad, transparent: true, opacity: 0.22 })))
  lightPool.rotation.x = -Math.PI / 2
  lightPool.position.y = 0.23
  scene.add(lightPool)

  const pillarGeometry = geometry(new THREE.CylinderGeometry(0.24, 0.34, 6.8, 10))
  for (let index = 0; index < 12; index += 1) {
    const angle = index / 12 * Math.PI * 2
    const pillar = new THREE.Mesh(pillarGeometry, stone)
    pillar.position.set(Math.sin(angle) * 13.8, 3.4, Math.cos(angle) * 13.8)
    pillar.castShadow = true
    scene.add(pillar)
  }
  const roofRing = new THREE.Mesh(geometry(new THREE.TorusGeometry(13.8, 0.2, 10, 96)), bronze)
  roofRing.rotation.x = Math.PI / 2
  roofRing.position.y = 6.75
  scene.add(roofRing)

  const pedestalBase = geometry(new THREE.CylinderGeometry(1.25, 1.5, 0.72, 10))
  const pedestalTop = geometry(new THREE.CylinderGeometry(1.06, 1.18, 0.18, 10))
  const cardBack = geometry(new THREE.BoxGeometry(3.7, 2.25, 0.16))
  const faceGeometry = geometry(new THREE.PlaneGeometry(3.5, 2.05))
  const exhibits: Array<{ id: string; slot: PlaqueLayoutSlot; group: InstanceType<typeof THREE.Group>; texture: InstanceType<typeof THREE.CanvasTexture>; faceMaterial: InstanceType<typeof THREE.MeshBasicMaterial>; light: InstanceType<typeof THREE.PointLight> }> = []
  let signature = ''

  function cardCanvas(card: IntroAcknowledgementCard) {
    const surface = document.createElement('canvas')
    surface.width = 640
    surface.height = 400
    const context = surface.getContext('2d')
    if (!context) throw new Error('无法创建鸣谢展牌纹理')
    const gradient = context.createLinearGradient(0, 0, 640, 400)
    const isUpstream = card.kind === 'upstream'
    const isAcknowledgement = card.eyebrow.includes('鸣谢')
    gradient.addColorStop(0, isUpstream ? '#405e5b' : isAcknowledgement ? '#69494b' : '#66583f')
    gradient.addColorStop(1, isUpstream ? '#1f3938' : isAcknowledgement ? '#35272d' : '#3d3427')
    context.fillStyle = gradient
    context.fillRect(0, 0, 640, 400)
    context.strokeStyle = isUpstream ? '#b6cfc4' : isAcknowledgement ? '#d38b78' : '#d5b46d'
    context.lineWidth = 8
    context.strokeRect(18, 18, 604, 364)
    context.fillStyle = isUpstream ? '#d8eee2' : isAcknowledgement ? '#f0b39b' : '#f0d28e'
    context.font = '700 25px "Microsoft YaHei UI", sans-serif'
    context.fillText(card.eyebrow, 50, 72)
    context.fillStyle = '#f5f2e9'
    context.font = '700 42px "Microsoft YaHei UI", sans-serif'
    context.fillText(fittedText(context, card.title, 540), 50, 145)
    context.fillStyle = 'rgba(235,238,232,.78)'
    context.font = '400 23px "Microsoft YaHei UI", sans-serif'
    context.fillText(fittedText(context, card.message, 540), 50, 218)
    context.fillStyle = '#e8e0d0'
    context.font = '700 25px "Microsoft YaHei UI", sans-serif'
    context.fillText(fittedText(context, card.detail, 540), 50, 315)
    return surface
  }

  function clearExhibits() {
    for (const exhibit of exhibits) {
      scene.remove(exhibit.group)
      exhibit.texture.dispose()
      exhibit.faceMaterial.dispose()
    }
    exhibits.length = 0
  }

  function rebuild(cards: IntroAcknowledgementCard[]) {
    clearExhibits()
    const records = cards.length ? cards : [{ id: 'fallback', kind: 'supporter' as const, eyebrow: '鸣谢同路人', title: '致每一位同路人', message: '长夜执剑，幸与诸君同路。', detail: 'CS2AS', updatedAt: '' }]
    const slots = buildPlaqueLayout(cards)
    records.forEach((card, index) => {
      const slot = slots[index] ?? { id: card.id, x: 0, z: 0, rotationY: 0, order: index }
      const group = new THREE.Group()
      group.position.set(slot.x, 0.6, slot.z)
      group.rotation.y = slot.rotationY + Math.PI
      const base = new THREE.Mesh(pedestalBase, stone)
      base.castShadow = true
      const top = new THREE.Mesh(pedestalTop, bronze)
      top.position.y = 0.45
      const texture = new THREE.CanvasTexture(cardCanvas(card))
      texture.colorSpace = THREE.SRGBColorSpace
      texture.minFilter = THREE.LinearFilter
      textures.add(texture)
      const faceMaterial = new THREE.MeshBasicMaterial({ map: texture })
      const panel = new THREE.Mesh(cardBack, jade)
      panel.position.set(0, 2.2, 0)
      const face = new THREE.Mesh(faceGeometry, faceMaterial)
      face.position.set(0, 2.2, 0.085)
      const reverseFace = new THREE.Mesh(faceGeometry, faceMaterial)
      reverseFace.position.set(0, 2.2, -0.085)
      reverseFace.rotation.y = Math.PI
      const light = new THREE.PointLight(card.kind === 'upstream' ? 0xd6a35d : 0x6ddce0, 5.5, 5, 1.8)
      light.position.set(0, 2.4, 1.1)
      group.add(base, top, panel, face, reverseFace, light)
      scene.add(group)
      exhibits.push({ id: card.id, slot, group, texture, faceMaterial, light })
    })
  }

  function sync() {
    const state = getState()
    const next = introCardSignature(state.cards)
    if (next !== signature || !exhibits.length) { signature = next; rebuild(state.cards) }
    return state
  }

  function cameraAt(state: IntroSceneState) {
    const count = state.cards.length || 1
    const elapsed = state.elapsedMs
    if (state.mode === 'free-roam') {
      const index = Math.max(0, Math.min(count - 1, state.selectedIndex ?? 0))
      const exhibit = exhibits[index]
      const slot = exhibit?.slot ?? { x: 0, z: 0, rotationY: 0 }
      camera.position.set(slot.x + 10.8, 3.35, slot.z + 10.8)
      target.set(slot.x, 1.8, slot.z)
      return
    }
    if (elapsed < 650) {
      const t = easeInOut(elapsed / 650)
      camera.position.set(0, 2.8 + t * 0.3, 14 - t * 4.2)
      target.set(0, 1.7, 0)
      return
    }
    if (elapsed < state.supporterLockMs) {
      const t = (elapsed - 650) / (state.supporterLockMs - 650)
      const angle = -0.4 + t * Math.PI * 0.92
      camera.position.set(Math.sin(angle) * 12.4, 3.35, Math.cos(angle) * 12.4)
      target.set(Math.sin(angle) * 5.8, 2.0, Math.cos(angle) * 5.8)
      return
    }
    if (elapsed < 7_800) {
      const t = easeInOut((elapsed - state.supporterLockMs) / (7_800 - state.supporterLockMs))
      const supporterStart = Math.min(5, count - 1)
      const index = supporterStart + t * Math.max(1, count - supporterStart)
      const angle = index / count * Math.PI * 2
      camera.position.set(Math.sin(angle - 0.12) * 12.1, 3.05 + Math.sin(t * Math.PI) * 0.3, Math.cos(angle - 0.12) * 12.1)
      target.set(Math.sin(angle) * 6.0, 2.0, Math.cos(angle) * 6.0)
      return
    }
    const t = easeInOut((elapsed - 7_800) / Math.max(1, state.durationMs - 7_800))
    camera.position.set(Math.sin(2.25) * (8.8 + t * 4.8), 3 + t * 3.5, Math.cos(2.25) * (8.8 + t * 4.8))
    target.set(0, 1.4, 0)
  }

  function resize(size: StageSize) {
    renderer.setPixelRatio(Math.min(size.dpr, 1.5))
    renderer.setSize(size.width, size.height, false)
    camera.aspect = size.width / size.height
    camera.updateProjectionMatrix()
  }

  resize(initialSize)
  const initial = sync()
  cameraAt(initial)
  camera.lookAt(target)
  renderer.render(scene, camera)

  return {
    resize,
    frame(time) {
      const state = sync()
      cameraAt(state)
      camera.lookAt(target)
      exhibits.forEach((exhibit, index) => {
        exhibit.group.position.y = 0.6 + Math.sin(time * 0.55 + index * 0.7) * 0.025
        exhibit.light.intensity = 5.2 + Math.sin(time * 0.8 + index) * 0.35
      })
      lightPool.rotation.z = time * 0.018
      renderer.render(scene, camera)
    },
    setLowPerformance(enabled) {
      renderer.setPixelRatio(1)
      renderer.shadowMap.enabled = !enabled
      exhibits.forEach(exhibit => { exhibit.light.castShadow = false })
    },
    dispose() {
      clearExhibits()
      textures.forEach(value => value.dispose())
      geometries.forEach(value => value.dispose())
      materials.forEach(value => value.dispose())
      renderer.dispose()
      renderer.forceContextLoss()
    },
  }
}
