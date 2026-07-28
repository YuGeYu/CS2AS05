import type { ThreeStageController, StageSize } from '@/components/three/stage'
import type { SupporterAcknowledgement, UpstreamProjectSummary } from './types'

export interface IntroSceneState {
  elapsedMs: number
  supporterSlotMs: number
  phase: 'supporters' | 'upstream'
  loading: boolean
  supporters: SupporterAcknowledgement[]
  upstream: UpstreamProjectSummary
}

export function introSupporterSignature(supporters: SupporterAcknowledgement[]) {
  return supporters.map(item => `${item.id}:${item.updatedAt}`).join('|')
}

function fittedText(context: CanvasRenderingContext2D, value: string, maxWidth: number) {
  if (context.measureText(value).width <= maxWidth) return value
  let output = value
  while (output.length > 1 && context.measureText(`${output}...`).width > maxWidth) output = output.slice(0, -1)
  return `${output}...`
}

export async function createIntroScene(
  canvas: HTMLCanvasElement,
  initialSize: StageSize,
  getState: () => IntroSceneState,
): Promise<ThreeStageController> {
  const THREE = await import('three')
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false, powerPreference: 'high-performance' })
  renderer.setClearColor(0x050d11, 1)
  renderer.outputColorSpace = THREE.SRGBColorSpace
  const scene = new THREE.Scene()
  scene.fog = new THREE.FogExp2(0x050d11, 0.032)
  const camera = new THREE.PerspectiveCamera(48, 1, 0.1, 120)
  camera.position.set(0, 2.25, 12.5)
  scene.add(new THREE.HemisphereLight(0x79dce6, 0x071014, 1.3))
  const key = new THREE.DirectionalLight(0x83ecf3, 3)
  key.position.set(4, 7, 6)
  scene.add(key)
  const bronzeLight = new THREE.PointLight(0xc59a4a, 26, 22)
  bronzeLight.position.set(-5, 2.5, 2)
  scene.add(bronzeLight)

  const geometries = new Set<InstanceType<typeof THREE.BufferGeometry>>()
  const materials = new Set<InstanceType<typeof THREE.Material>>()
  const geometry = <T extends InstanceType<typeof THREE.BufferGeometry>>(value: T) => { geometries.add(value); return value }
  const material = <T extends InstanceType<typeof THREE.Material>>(value: T) => { materials.add(value); return value }

  const floor = new THREE.Mesh(
    geometry(new THREE.PlaneGeometry(22, 80)),
    material(new THREE.MeshStandardMaterial({ color: 0x08191d, metalness: 0.15, roughness: 0.82 })),
  )
  floor.rotation.x = -Math.PI / 2
  floor.position.set(0, -2.25, -24)
  scene.add(floor)

  const stone = material(new THREE.MeshStandardMaterial({ color: 0x102f34, emissive: 0x061519, roughness: 0.7 }))
  const bronze = material(new THREE.MeshStandardMaterial({ color: 0x987136, emissive: 0x271906, metalness: 0.55, roughness: 0.38 }))
  const pillarGeometry = geometry(new THREE.CylinderGeometry(0.2, 0.28, 6.4, 8))
  const beamGeometry = geometry(new THREE.BoxGeometry(10, 0.32, 0.42))
  const roofGeometry = geometry(new THREE.BoxGeometry(11.2, 0.16, 1.1))
  const arches: InstanceType<typeof THREE.Group>[] = []
  for (const z of [-5, -18, -31]) {
    const arch = new THREE.Group()
    for (const x of [-4.5, 4.5]) {
      const pillar = new THREE.Mesh(pillarGeometry, stone)
      pillar.position.set(x, 0.9, 0)
      arch.add(pillar)
    }
    const beam = new THREE.Mesh(beamGeometry, bronze)
    beam.position.y = 3.75
    const roof = new THREE.Mesh(roofGeometry, stone)
    roof.position.y = 4.15
    arch.add(beam, roof)
    arch.position.z = z
    scene.add(arch)
    arches.push(arch)
  }

  const mountainMaterial = material(new THREE.MeshBasicMaterial({ color: 0x10252b, side: THREE.DoubleSide }))
  for (let index = 0; index < 5; index += 1) {
    const mountain = new THREE.Mesh(geometry(new THREE.ConeGeometry(5 + index * 0.8, 7 + index, 5)), mountainMaterial)
    mountain.position.set(index % 2 ? 8 : -8, -0.2, -35 - index * 3.5)
    mountain.rotation.y = index * 0.65
    scene.add(mountain)
  }

  const scanMaterial = material(new THREE.MeshBasicMaterial({ color: 0x45c6d7, wireframe: true, transparent: true, opacity: 0.16 }))
  const scanGeometry = geometry(new THREE.BoxGeometry(10.5, 6.8, 0.2))
  const scanGates = Array.from({ length: 4 }, (_, index) => {
    const gate = new THREE.Mesh(scanGeometry, scanMaterial)
    gate.position.set(0, 0.7, -8 - index * 11)
    scene.add(gate)
    return gate
  })

  const flowMaterial = material(new THREE.MeshBasicMaterial({ color: 0x32a9bd, transparent: true, opacity: 0.42 }))
  const flowGeometry = geometry(new THREE.BoxGeometry(13, 0.018, 0.055))
  const flowLines = Array.from({ length: 20 }, (_, index) => {
    const line = new THREE.Mesh(flowGeometry, flowMaterial)
    line.position.set(0, -2.21, 7 - index * 3.2)
    scene.add(line)
    return line
  })

  const sword = new THREE.Group()
  const blade = new THREE.Mesh(geometry(new THREE.ConeGeometry(0.14, 4.8, 4)), material(new THREE.MeshBasicMaterial({ color: 0x9af4ff })))
  blade.rotation.z = Math.PI
  const guard = new THREE.Mesh(geometry(new THREE.BoxGeometry(1.35, 0.12, 0.2)), bronze)
  guard.position.y = 2.42
  sword.add(blade, guard)
  sword.position.set(-3.6, 0.3, -1.5)
  sword.rotation.z = -0.48
  scene.add(sword)

  const pointCount = 170
  const positions = new Float32Array(pointCount * 3)
  for (let index = 0; index < pointCount; index += 1) {
    positions[index * 3] = (Math.random() - 0.5) * 28
    positions[index * 3 + 1] = Math.random() * 11 - 2
    positions[index * 3 + 2] = -Math.random() * 55
  }
  const pointsGeometry = geometry(new THREE.BufferGeometry())
  pointsGeometry.setAttribute('position', new THREE.BufferAttribute(positions, 3))
  const points = new THREE.Points(pointsGeometry, material(new THREE.PointsMaterial({ color: 0x79e5ef, size: 0.04, transparent: true, opacity: 0.65 })))
  scene.add(points)

  const tabletGeometry = geometry(new THREE.BoxGeometry(3.4, 2.15, 0.16))
  const tabletBackMaterial = material(new THREE.MeshStandardMaterial({ color: 0x123b40, emissive: 0x082329, metalness: 0.25, roughness: 0.42 }))
  const tablets: { group: InstanceType<typeof THREE.Group>; texture: InstanceType<typeof THREE.CanvasTexture>; faceMaterial: InstanceType<typeof THREE.MeshBasicMaterial> }[] = []
  let signature = ''
  let lowPerformance = false

  function clearTablets() {
    for (const tablet of tablets) {
      scene.remove(tablet.group)
      tablet.texture.dispose()
      tablet.faceMaterial.dispose()
    }
    tablets.length = 0
  }

  function canvasFor(supporter: SupporterAcknowledgement) {
    const surface = document.createElement('canvas')
    surface.width = 768
    surface.height = 480
    const context = surface.getContext('2d')
    if (!context) throw new Error('无法创建鸣谢碑文字纹理')
    const gradient = context.createLinearGradient(0, 0, 768, 480)
    gradient.addColorStop(0, '#174a51')
    gradient.addColorStop(1, '#0a2328')
    context.fillStyle = gradient
    context.fillRect(0, 0, 768, 480)
    context.strokeStyle = '#c59a4a'
    context.lineWidth = 10
    context.strokeRect(18, 18, 732, 444)
    context.strokeStyle = 'rgba(120,228,239,.46)'
    context.lineWidth = 2
    context.strokeRect(36, 36, 696, 408)
    context.fillStyle = '#78e4ef'
    context.font = '700 30px "Microsoft YaHei UI", "Microsoft YaHei", sans-serif'
    context.fillText('鸣谢同路人', 68, 95)
    context.fillStyle = '#ffffff'
    context.font = '700 58px "Microsoft YaHei UI", "Microsoft YaHei", sans-serif'
    context.fillText(fittedText(context, supporter.nickname || '青锋无名客', 630), 68, 190)
    context.fillStyle = 'rgba(231,237,240,.78)'
    context.font = '400 29px "Microsoft YaHei UI", "Microsoft YaHei", sans-serif'
    context.fillText(fittedText(context, supporter.message || '长夜执剑，幸与诸君同路。', 630), 68, 270)
    context.fillStyle = '#d7ae62'
    context.font = '700 36px "Microsoft YaHei UI", "Microsoft YaHei", sans-serif'
    context.fillText(`¥${(supporter.amountCents / 100).toFixed(2)}`, 68, 370)
    return surface
  }

  function rebuildTablets(supporters: SupporterAcknowledgement[]) {
    clearTablets()
    const visible = supporters.slice(0, 8)
    const records = visible.length ? visible : [{
      id: 'fallback', nickname: '致每一位同路人', message: '长夜执剑，幸与诸君同路。', amountCents: 0,
      sortOrder: 0, isVisible: true, createdAt: '', updatedAt: '',
    }]
    for (const supporter of records) {
      const texture = new THREE.CanvasTexture(canvasFor(supporter))
      texture.colorSpace = THREE.SRGBColorSpace
      texture.minFilter = THREE.LinearFilter
      const faceMaterial = new THREE.MeshBasicMaterial({ map: texture, transparent: true })
      const group = new THREE.Group()
      group.add(new THREE.Mesh(tabletGeometry, tabletBackMaterial))
      const face = new THREE.Mesh(new THREE.PlaneGeometry(3.18, 1.98), faceMaterial)
      geometries.add(face.geometry)
      face.position.z = 0.09
      group.add(face)
      scene.add(group)
      tablets.push({ group, texture, faceMaterial })
    }
  }

  function syncData() {
    const state = getState()
    const nextSignature = introSupporterSignature(state.supporters)
    if (nextSignature === signature && tablets.length) return state
    signature = nextSignature
    rebuildTablets(state.supporters)
    return state
  }

  function resize(size: StageSize) {
    renderer.setPixelRatio(Math.min(size.dpr, 1.5))
    renderer.setSize(size.width, size.height, false)
    camera.aspect = size.width / size.height
    camera.updateProjectionMatrix()
  }

  resize(initialSize)
  syncData()
  renderer.render(scene, camera)

  return {
    resize,
    frame(time, delta) {
      const state = syncData()
      const current = state.supporters.length ? state.elapsedMs / Math.max(1, state.supporterSlotMs) : 0
      tablets.forEach((tablet, index) => {
        const offset = index - current
        const wrapped = ((offset + tablets.length / 2) % tablets.length + tablets.length) % tablets.length - tablets.length / 2
        tablet.group.position.x = Math.sin(wrapped * 0.82) * 5.2
        tablet.group.position.y = 0.25 + Math.cos(time * 0.9 + index) * 0.08
        tablet.group.position.z = -1.8 - Math.abs(wrapped) * 2.2
        tablet.group.rotation.y = -Math.sin(wrapped * 0.62) * 0.58
        const focus = Math.max(0, 1 - Math.abs(wrapped) * 0.45)
        tablet.group.scale.setScalar(0.76 + focus * 0.24)
        tablet.group.visible = state.phase === 'supporters' && Math.abs(wrapped) < 3.4
      })
      camera.position.x = Math.sin(time * 0.24) * 0.28
      camera.position.y = 2.25 + Math.sin(time * 0.31) * 0.08
      camera.lookAt(0, 0.15, -6)
      sword.rotation.y = Math.sin(time * 0.72) * 0.18
      scanGates.forEach((gate, index) => {
        gate.position.z += delta * 1.8
        if (gate.position.z > 8) gate.position.z = -35 - index * 2
      })
      flowLines.forEach(line => {
        line.position.z += delta * 6.2
        if (line.position.z > 8) line.position.z -= 64
      })
      if (!lowPerformance) points.rotation.y += delta * 0.016
      renderer.render(scene, camera)
    },
    setLowPerformance(enabled) { lowPerformance = enabled; points.visible = !enabled; renderer.setPixelRatio(1) },
    dispose() {
      clearTablets()
      geometries.forEach(value => value.dispose())
      materials.forEach(value => value.dispose())
      renderer.dispose()
    },
  }
}
