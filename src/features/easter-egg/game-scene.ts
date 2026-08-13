import type { StageInteraction, StageSize, ThreeStageController } from '@/components/three/stage'
import type { IntroAcknowledgementCard } from '@/features/intro/types'

export const GALLERY_MOVE_SPEED = 4.2
export const GALLERY_RADIUS = 10.4

function cardSignature(cards: IntroAcknowledgementCard[]) {
  return cards.map(card => `${card.id}:${card.updatedAt}`).join('|')
}

export function clampGalleryPosition(x: number, z: number, radius = GALLERY_RADIUS) {
  const distance = Math.hypot(x, z)
  if (distance <= radius) return { x, z }
  const scale = radius / distance
  return { x: x * scale, z: z * scale }
}

function fittedText(context: CanvasRenderingContext2D, value: string, maxWidth: number) {
  if (context.measureText(value).width <= maxWidth) return value
  let output = value
  while (output.length > 1 && context.measureText(`${output}...`).width > maxWidth) output = output.slice(0, -1)
  return `${output}...`
}

export async function createContributionGalleryScene(
  canvas: HTMLCanvasElement,
  initialSize: StageSize,
  getCards: () => IntroAcknowledgementCard[],
): Promise<ThreeStageController> {
  const THREE = await import('three')
  const { GLTFLoader } = await import('three/examples/jsm/loaders/GLTFLoader.js')
  const { DRACOLoader } = await import('three/examples/jsm/loaders/DRACOLoader.js')
  const { KTX2Loader } = await import('three/examples/jsm/loaders/KTX2Loader.js')
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' })
  renderer.setClearColor(0x17464a)
  renderer.outputColorSpace = THREE.SRGBColorSpace
  renderer.shadowMap.enabled = true
  renderer.shadowMap.type = THREE.PCFShadowMap
  const scene = new THREE.Scene()
  scene.background = new THREE.Color(0x17464a)
  scene.fog = new THREE.FogExp2(0x17464a, 0.022)
  const camera = new THREE.PerspectiveCamera(58, 1, 0.1, 80)
  camera.position.set(0, 1.72, 5.4)

  const geometries = new Set<InstanceType<typeof THREE.BufferGeometry>>()
  const materials = new Set<InstanceType<typeof THREE.Material>>()
  const textures = new Set<InstanceType<typeof THREE.Texture>>()
  const geometry = <T extends InstanceType<typeof THREE.BufferGeometry>>(value: T) => { geometries.add(value); return value }
  const material = <T extends InstanceType<typeof THREE.Material>>(value: T) => { materials.add(value); return value }

  scene.add(new THREE.HemisphereLight(0xffffff, 0x35666a, 2.8))
  const daylight = new THREE.DirectionalLight(0xeaffff, 5.6)
  daylight.position.set(6, 11, 7); daylight.castShadow = true; daylight.shadow.mapSize.set(1024, 1024); scene.add(daylight)
  const goldLight = new THREE.PointLight(0xffbd62, 74, 34, 1.4)
  goldLight.position.set(0, 5.6, 0); scene.add(goldLight)
  const cyanLight = new THREE.PointLight(0x78f5ff, 42, 28, 1.5)
  cyanLight.position.set(-7, 3.8, -5); scene.add(cyanLight)
  const artifactKeyLight = new THREE.PointLight(0xfff2d2, 68, 18, 1.35)
  artifactKeyLight.position.set(3.8, 4.6, 4.4); scene.add(artifactKeyLight)
  const artifactRimLight = new THREE.PointLight(0xff9c7c, 44, 14, 1.5)
  artifactRimLight.position.set(-3.6, 3.3, -2.8); scene.add(artifactRimLight)

  const stone = material(new THREE.MeshStandardMaterial({ color: 0x3d686b, roughness: 0.68, metalness: 0.06 }))
  const floorMaterial = material(new THREE.MeshStandardMaterial({ color: 0x294f52, roughness: 0.72, metalness: 0.12 }))
  const bronze = material(new THREE.MeshStandardMaterial({ color: 0xc58c3f, emissive: 0x56320c, roughness: 0.28, metalness: 0.64 }))
  const jade = material(new THREE.MeshStandardMaterial({ color: 0x318087, emissive: 0x155158, roughness: 0.4, metalness: 0.18 }))
  const floor = new THREE.Mesh(geometry(new THREE.CircleGeometry(13.6, 72)), floorMaterial)
  floor.rotation.x = -Math.PI / 2; floor.receiveShadow = true; scene.add(floor)
  const dais = new THREE.Mesh(geometry(new THREE.CylinderGeometry(4, 4.5, 0.34, 48)), stone)
  dais.position.y = 0.12; dais.receiveShadow = true; scene.add(dais)
  const pathRing = new THREE.Mesh(geometry(new THREE.TorusGeometry(7.1, 0.075, 12, 120)), bronze)
  pathRing.rotation.x = Math.PI / 2; pathRing.position.y = 0.08; scene.add(pathRing)
  const lightPool = new THREE.Mesh(geometry(new THREE.CircleGeometry(3.55, 48)), material(new THREE.MeshBasicMaterial({ color: 0x6ce8ee, transparent: true, opacity: 0.2 })))
  lightPool.rotation.x = -Math.PI / 2; lightPool.position.y = 0.31; scene.add(lightPool)

  const pillarGeometry = geometry(new THREE.CylinderGeometry(0.25, 0.36, 7.2, 12))
  for (let index = 0; index < 12; index += 1) {
    const angle = index / 12 * Math.PI * 2
    const pillar = new THREE.Mesh(pillarGeometry, stone)
    pillar.position.set(Math.sin(angle) * 11.5, 3.6, Math.cos(angle) * 11.5); pillar.castShadow = true; scene.add(pillar)
  }
  const roofRing = new THREE.Mesh(geometry(new THREE.TorusGeometry(11.5, 0.23, 12, 120)), bronze)
  roofRing.rotation.x = Math.PI / 2; roofRing.position.y = 7.1; scene.add(roofRing)

  const pedestalBase = geometry(new THREE.CylinderGeometry(1.02, 1.28, 0.65, 12))
  const panelBack = geometry(new THREE.BoxGeometry(3.35, 2.05, 0.18))
  const panelFace = geometry(new THREE.PlaneGeometry(3.14, 1.84))
  const exhibitGroups: InstanceType<typeof THREE.Group>[] = []
  const faceMaterials = new Set<InstanceType<typeof THREE.Material>>()
  let signature = ''
  function clearExhibits() {
    exhibitGroups.forEach(group => scene.remove(group)); exhibitGroups.length = 0
    faceMaterials.forEach(value => value.dispose()); faceMaterials.clear()
    textures.forEach(value => value.dispose()); textures.clear()
  }
  function rebuildExhibits(nextCards: IntroAcknowledgementCard[]) {
    clearExhibits()
    const cards = nextCards.length ? nextCards : [{ id: 'fallback', kind: 'supporter' as const, eyebrow: '鸣谢同路人', title: '致每一位同路人', message: '幸与诸君同路。', detail: 'CS2AS', updatedAt: '' }]
    cards.forEach((card, index) => {
    const surface = document.createElement('canvas'); surface.width = 640; surface.height = 380
    const context = surface.getContext('2d'); if (!context) throw new Error('无法创建贡献展牌')
    const gradient = context.createLinearGradient(0, 0, 640, 380); gradient.addColorStop(0, card.kind === 'upstream' ? '#315b5d' : '#267078'); gradient.addColorStop(1, '#173f43')
    context.fillStyle = gradient; context.fillRect(0, 0, 640, 380); context.strokeStyle = '#e0ad5c'; context.lineWidth = 8; context.strokeRect(18, 18, 604, 344)
    context.fillStyle = '#ffd27b'; context.font = '700 24px "Microsoft YaHei UI"'; context.fillText(card.eyebrow, 48, 70)
    context.fillStyle = '#ffffff'; context.font = '700 40px "Microsoft YaHei UI"'; context.fillText(fittedText(context, card.title, 540), 48, 140)
    context.fillStyle = '#d8eeee'; context.font = '400 22px "Microsoft YaHei UI"'; context.fillText(fittedText(context, card.message, 540), 48, 210)
    context.fillStyle = '#9ef9ff'; context.font = '700 24px "Microsoft YaHei UI"'; context.fillText(fittedText(context, card.detail, 540), 48, 305)
    const texture = new THREE.CanvasTexture(surface); texture.colorSpace = THREE.SRGBColorSpace; textures.add(texture)
    const faceMaterial = new THREE.MeshBasicMaterial({ map: texture }); faceMaterials.add(faceMaterial)
    const angle = index / cards.length * Math.PI * 2
    const group = new THREE.Group(); group.position.set(Math.sin(angle) * 7.15, 0.45, Math.cos(angle) * 7.15); group.rotation.y = angle + Math.PI
    const base = new THREE.Mesh(pedestalBase, stone); base.castShadow = true
    const panel = new THREE.Mesh(panelBack, jade); panel.position.y = 2.05
    const front = new THREE.Mesh(panelFace, faceMaterial); front.position.set(0, 2.05, 0.1)
    const back = new THREE.Mesh(panelFace, faceMaterial); back.position.set(0, 2.05, -0.1); back.rotation.y = Math.PI
    const lamp = new THREE.PointLight(card.kind === 'upstream' ? 0xffc770 : 0x78f5ff, 8, 6, 1.5); lamp.position.set(0, 2.4, 1.1)
      group.add(base, panel, front, back, lamp); scene.add(group); exhibitGroups.push(group)
    })
  }
  function syncExhibits() {
    const cards = getCards(); const next = cardSignature(cards)
    if (next !== signature || !exhibitGroups.length) { signature = next; rebuildExhibits(cards) }
  }
  syncExhibits()

  const artifactPedestal = new THREE.Mesh(geometry(new THREE.CylinderGeometry(1.7, 2.1, 0.9, 16)), bronze)
  artifactPedestal.position.y = 0.55; artifactPedestal.castShadow = true; scene.add(artifactPedestal)
  const artifactRoot = new THREE.Group(); artifactRoot.position.y = 1.08; scene.add(artifactRoot)
  const artifactBeacon = new THREE.Mesh(
    geometry(new THREE.CylinderGeometry(0.58, 0.76, 2.9, 8)),
    material(new THREE.MeshStandardMaterial({ color: 0xd7a550, emissive: 0x5f3508, emissiveIntensity: 0.72, roughness: 0.34, metalness: 0.5, wireframe: true })),
  )
  artifactBeacon.position.y = 1.48; artifactRoot.add(artifactBeacon)
  let artifact: InstanceType<typeof THREE.Object3D> | null = null
  let disposed = false
  const dracoLoader = new DRACOLoader().setDecoderPath('/museum/contribution-gallery/draco/').preload()
  const ktx2Loader = new KTX2Loader().setTranscoderPath('/museum/contribution-gallery/basis/').detectSupport(renderer)
  const gltfLoader = new GLTFLoader().setDRACOLoader(dracoLoader).setKTX2Loader(ktx2Loader)
  void gltfLoader.loadAsync('/museum/contribution-gallery/tang-painted-court-lady.glb').then((gltf) => {
    if (disposed) return
    artifact = gltf.scene
    artifact.updateMatrixWorld(true)
    const sourceBox = new THREE.Box3().setFromObject(artifact); const sourceSize = sourceBox.getSize(new THREE.Vector3())
    if (sourceBox.isEmpty() || !Number.isFinite(sourceSize.length()) || sourceSize.length() < 0.001) throw new Error('唐代彩绘仕女俑模型没有可显示的几何体')
    const scale = 3.2 / Math.max(sourceSize.x, sourceSize.y, sourceSize.z); artifact.scale.setScalar(scale)
    artifact.updateMatrixWorld(true)
    const fittedBox = new THREE.Box3().setFromObject(artifact); const center = fittedBox.getCenter(new THREE.Vector3())
    artifact.position.x -= center.x; artifact.position.z -= center.z; artifact.position.y -= fittedBox.min.y
    artifact.traverse(object => { if (object instanceof THREE.Mesh) { object.castShadow = true; object.receiveShadow = true } })
    artifactRoot.remove(artifactBeacon); artifactRoot.add(artifact)
  }).catch(error => console.warn('[ContributionGallery] central artifact unavailable', error))

  const keys = new Set<string>()
  let yaw = 0
  let pitch = -0.05
  let dragging = false
  let pointerX = 0
  let pointerY = 0
  let artifactRotation = 0
  let lowPerformance = false
  function resize(size: StageSize) { renderer.setPixelRatio(lowPerformance ? 1 : Math.min(size.dpr, 1.5)); renderer.setSize(size.width, size.height, false); camera.aspect = size.width / size.height; camera.updateProjectionMatrix() }
  function interact(event: StageInteraction) {
    if (event.type === 'key-down' && event.key) keys.add(event.key)
    if (event.type === 'key-up' && event.key) keys.delete(event.key)
    if (event.type === 'pointer-down') { dragging = true; pointerX = event.x ?? 0; pointerY = event.y ?? 0 }
    if (event.type === 'pointer-move' && dragging) {
      const x = event.x ?? pointerX; const y = event.y ?? pointerY
      yaw -= (x - pointerX) * 3.2; pitch = Math.max(-0.72, Math.min(0.72, pitch - (y - pointerY) * 2.4)); pointerX = x; pointerY = y
    }
    if (event.type === 'pointer-up') dragging = false
  }
  resize(initialSize)
  return {
    resize,
    interact,
    frame(time, delta) {
      syncExhibits()
      const forward = Number(keys.has('w') || keys.has('arrowup')) - Number(keys.has('s') || keys.has('arrowdown'))
      const strafe = Number(keys.has('d') || keys.has('arrowright')) - Number(keys.has('a') || keys.has('arrowleft'))
      if (forward || strafe) {
        const length = Math.hypot(forward, strafe) || 1; const distance = GALLERY_MOVE_SPEED * Math.min(delta, 0.05)
        const dx = (-Math.sin(yaw) * forward + Math.cos(yaw) * strafe) / length * distance
        const dz = (-Math.cos(yaw) * forward - Math.sin(yaw) * strafe) / length * distance
        const next = clampGalleryPosition(camera.position.x + dx, camera.position.z + dz); camera.position.x = next.x; camera.position.z = next.z
      }
      camera.rotation.order = 'YXZ'; camera.rotation.y = yaw; camera.rotation.x = pitch
      exhibitGroups.forEach((group, index) => { group.position.y = 0.45 + Math.sin(time * 0.65 + index) * 0.018 })
      artifactRotation += delta * 0.12; artifactRoot.rotation.y = artifactRotation; lightPool.rotation.z = time * 0.025
      renderer.render(scene, camera)
    },
    setLowPerformance(enabled) { lowPerformance = enabled; renderer.shadowMap.enabled = !enabled; renderer.setPixelRatio(enabled ? 1 : Math.min(initialSize.dpr, 1.5)) },
    dispose() {
      disposed = true; keys.clear(); clearExhibits(); dracoLoader.dispose(); ktx2Loader.dispose(); geometries.forEach(value => value.dispose()); materials.forEach(value => value.dispose())
      artifact?.traverse(object => { if (object instanceof THREE.Mesh) { object.geometry.dispose(); const values = Array.isArray(object.material) ? object.material : [object.material]; values.forEach(value => { value.map?.dispose(); value.dispose() }) } })
      renderer.dispose(); renderer.forceContextLoss()
    },
  }
}
