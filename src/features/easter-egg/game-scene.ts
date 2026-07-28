import type { StageInteraction, StageSize, ThreeStageController } from '@/components/three/stage'

export interface GameSceneEvent {
  score: number
  combo: number
  bestCombo: number
  lives: number
}

export type GameTargetKind = 'jade' | 'bronze' | 'directional'

export interface GameTarget {
  id: number
  kind: GameTargetKind
  x: number
  y: number
  z: number
  radius: number
}

interface ScreenTarget { id: number; x: number; y: number; radius: number }
interface Point { x: number; y: number }

export const TARGET_SPEED = 6.5
export const TARGET_SPAWN_Z = -18
export const TARGET_MISS_Z = 7
const SPAWN_INTERVAL = 0.92
const MAX_TARGETS = 16
const MISS_PENALTY_COOLDOWN = 3.4

export function distanceToSegment(point: Point, start: Point, end: Point) {
  const dx = end.x - start.x
  const dy = end.y - start.y
  const lengthSquared = dx * dx + dy * dy
  if (lengthSquared === 0) return Math.hypot(point.x - start.x, point.y - start.y)
  const t = Math.max(0, Math.min(1, ((point.x - start.x) * dx + (point.y - start.y) * dy) / lengthSquared))
  return Math.hypot(point.x - (start.x + dx * t), point.y - (start.y + dy * t))
}

export class GameSimulation {
  readonly targets: GameTarget[] = []
  score = 0
  combo = 0
  bestCombo = 0
  lives = 3
  active = false
  private nextId = 1
  private spawnAccumulator = 0
  private missPenaltyCooldown = 0

  constructor(
    private readonly random: () => number = Math.random,
    private readonly onChange: (event: GameSceneEvent) => void = () => undefined,
  ) {}

  setActive(active: boolean) {
    if (active === this.active) return
    this.active = active
    this.spawnAccumulator = 0
    if (active && this.targets.length === 0) this.spawn()
  }

  step(delta: number) {
    if (!this.active) return
    const safeDelta = Math.min(Math.max(delta, 0), 0.05)
    this.missPenaltyCooldown = Math.max(0, this.missPenaltyCooldown - safeDelta)
    this.spawnAccumulator += safeDelta
    while (this.spawnAccumulator >= SPAWN_INTERVAL && this.targets.length < MAX_TARGETS) {
      this.spawnAccumulator -= SPAWN_INTERVAL
      this.spawn()
    }
    for (let index = this.targets.length - 1; index >= 0; index -= 1) {
      const target = this.targets[index]
      if (!target) continue
      target.z += TARGET_SPEED * safeDelta
      if (target.z <= TARGET_MISS_Z) continue
      this.targets.splice(index, 1)
      if (target.kind !== 'bronze' && this.missPenaltyCooldown === 0) {
        this.combo = 0
        this.lives = Math.max(0, this.lives - 1)
        this.missPenaltyCooldown = MISS_PENALTY_COOLDOWN
        this.publish()
      }
    }
  }

  slash(start: Point, end: Point, projected: ScreenTarget[]) {
    if (!this.active || Math.hypot(end.x - start.x, end.y - start.y) < 0.012) return []
    const hits: { target: GameTarget; blocked: boolean }[] = []
    for (const screen of projected) {
      const targetIndex = this.targets.findIndex(target => target.id === screen.id)
      if (targetIndex < 0 || distanceToSegment(screen, start, end) > screen.radius) continue
      const [target] = this.targets.splice(targetIndex, 1)
      if (!target) continue
      const blocked = target.kind === 'bronze'
      if (blocked) this.combo = 0
      else {
        this.combo += 1
        this.bestCombo = Math.max(this.bestCombo, this.combo)
        this.score += 100 + this.combo * 10
      }
      hits.push({ target, blocked })
      this.publish()
    }
    return hits
  }

  reset() {
    this.targets.length = 0
    this.score = 0
    this.combo = 0
    this.bestCombo = 0
    this.lives = 3
    this.active = false
    this.spawnAccumulator = 0
    this.missPenaltyCooldown = 0
    this.nextId = 1
    this.publish()
  }

  private spawn() {
    const roll = this.random()
    this.targets.push({
      id: this.nextId++,
      kind: roll < 0.18 ? 'bronze' : roll < 0.4 ? 'directional' : 'jade',
      x: (this.random() - 0.5) * 7.4,
      y: this.random() * 4.2 - 0.9,
      z: TARGET_SPAWN_Z,
      radius: 0.075,
    })
  }

  private publish() {
    this.onChange({ score: this.score, combo: this.combo, bestCombo: this.bestCombo, lives: this.lives })
  }
}

export async function createGameScene(
  canvas: HTMLCanvasElement,
  initialSize: StageSize,
  onChange: (event: GameSceneEvent) => void,
): Promise<ThreeStageController> {
  const THREE = await import('three')
  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' })
  renderer.setClearColor(0x050d11)
  renderer.outputColorSpace = THREE.SRGBColorSpace
  const scene = new THREE.Scene()
  scene.fog = new THREE.Fog(0x050d11, 14, 48)
  const camera = new THREE.PerspectiveCamera(52, 1, 0.1, 90)
  camera.position.set(0, 1.7, 8)
  camera.lookAt(0, 0.3, -13)
  scene.add(new THREE.HemisphereLight(0x86eaf1, 0x071014, 1.45))
  const keyLight = new THREE.PointLight(0xc99b4a, 30, 25)
  keyLight.position.set(-5, 4.5, 1)
  scene.add(keyLight)
  const fillLight = new THREE.PointLight(0x42c8dc, 22, 22)
  fillLight.position.set(5, 2.8, -7)
  scene.add(fillLight)

  const disposableGeometries = new Set<InstanceType<typeof THREE.BufferGeometry>>()
  const disposableMaterials = new Set<InstanceType<typeof THREE.Material>>()
  const geometry = <T extends InstanceType<typeof THREE.BufferGeometry>>(value: T) => { disposableGeometries.add(value); return value }
  const material = <T extends InstanceType<typeof THREE.Material>>(value: T) => { disposableMaterials.add(value); return value }

  const stoneMaterial = material(new THREE.MeshStandardMaterial({ color: 0x102d32, metalness: 0.2, roughness: 0.7, emissive: 0x061417 }))
  const trimMaterial = material(new THREE.MeshStandardMaterial({ color: 0x8c6b32, metalness: 0.62, roughness: 0.34, emissive: 0x241806 }))
  const cyanMaterial = material(new THREE.MeshBasicMaterial({ color: 0x55ddeb }))
  const jadeMaterial = material(new THREE.MeshStandardMaterial({ color: 0x2b8792, emissive: 0x15535e, metalness: 0.25, roughness: 0.34 }))
  const bronzeMaterial = material(new THREE.MeshStandardMaterial({ color: 0x9b702d, emissive: 0x402706, metalness: 0.7, roughness: 0.3 }))
  const targetBodyGeometry = geometry(new THREE.BoxGeometry(1.15, 1.65, 0.24))
  const targetMarkGeometry = geometry(new THREE.BoxGeometry(0.12, 1.05, 0.06))
  const bronzeRingGeometry = geometry(new THREE.TorusGeometry(0.68, 0.15, 8, 24))

  const floorMaterial = material(new THREE.MeshStandardMaterial({ color: 0x091a1e, roughness: 0.84, metalness: 0.1 }))
  const floor = new THREE.Mesh(geometry(new THREE.PlaneGeometry(18, 70)), floorMaterial)
  floor.rotation.x = -Math.PI / 2
  floor.position.set(0, -2.15, -22)
  scene.add(floor)

  const archGroups: InstanceType<typeof THREE.Group>[] = []
  const pillarGeometry = geometry(new THREE.CylinderGeometry(0.22, 0.28, 6.2, 8))
  const beamGeometry = geometry(new THREE.BoxGeometry(9.5, 0.34, 0.42))
  const eaveGeometry = geometry(new THREE.BoxGeometry(10.7, 0.18, 1.05))
  for (const z of [-3, -15, -27]) {
    const arch = new THREE.Group()
    for (const x of [-4.25, 4.25]) {
      const pillar = new THREE.Mesh(pillarGeometry, stoneMaterial)
      pillar.position.set(x, 0.85, 0)
      arch.add(pillar)
    }
    const beam = new THREE.Mesh(beamGeometry, trimMaterial)
    beam.position.y = 3.65
    const eave = new THREE.Mesh(eaveGeometry, stoneMaterial)
    eave.position.y = 4.05
    eave.rotation.z = 0.015
    arch.add(beam, eave)
    arch.position.z = z
    scene.add(arch)
    archGroups.push(arch)
  }

  const mountainMaterial = material(new THREE.MeshBasicMaterial({ color: 0x10252b, side: THREE.DoubleSide }))
  for (let index = 0; index < 4; index += 1) {
    const mountain = new THREE.Mesh(geometry(new THREE.ConeGeometry(5 + index, 7 + index * 1.4, 5)), mountainMaterial)
    mountain.position.set(index % 2 ? 7.5 : -8, 0.2, -34 - index * 4)
    mountain.rotation.y = index * 0.7
    scene.add(mountain)
  }

  const scanMaterial = material(new THREE.MeshBasicMaterial({ color: 0x37b8cc, wireframe: true, transparent: true, opacity: 0.18 }))
  const scanGeometry = geometry(new THREE.BoxGeometry(10, 6.5, 0.22))
  const scanGates = Array.from({ length: 4 }, (_, index) => {
    const gate = new THREE.Mesh(scanGeometry, scanMaterial)
    gate.position.set(0, 0.7, -8 - index * 10)
    scene.add(gate)
    return gate
  })

  const flowMaterial = material(new THREE.MeshBasicMaterial({ color: 0x2da9bd, transparent: true, opacity: 0.48 }))
  const flowGeometry = geometry(new THREE.BoxGeometry(12, 0.018, 0.055))
  const flowLines = Array.from({ length: 18 }, (_, index) => {
    const line = new THREE.Mesh(flowGeometry, flowMaterial)
    line.position.set(0, -2.11, 5 - index * 3.2)
    scene.add(line)
    return line
  })

  const lanternGeometry = geometry(new THREE.CylinderGeometry(0.22, 0.3, 0.75, 8))
  const lanternMaterial = material(new THREE.MeshBasicMaterial({ color: 0xd9a84d }))
  const lanterns = [-1, 1].flatMap(side => archGroups.map((arch, index) => {
    const lantern = new THREE.Mesh(lanternGeometry, lanternMaterial)
    lantern.position.set(side * 3.45, 2.7, arch.position.z + 0.2)
    lantern.userData.phase = index * 0.8 + side
    scene.add(lantern)
    return lantern
  }))

  const particleCount = 110
  const particlePositions = new Float32Array(particleCount * 3)
  for (let index = 0; index < particleCount; index += 1) {
    particlePositions[index * 3] = (Math.random() - 0.5) * 16
    particlePositions[index * 3 + 1] = Math.random() * 8 - 2
    particlePositions[index * 3 + 2] = -Math.random() * 45
  }
  const particleGeometry = geometry(new THREE.BufferGeometry())
  particleGeometry.setAttribute('position', new THREE.BufferAttribute(particlePositions, 3))
  const particles = new THREE.Points(particleGeometry, material(new THREE.PointsMaterial({ color: 0x85e9ee, size: 0.04, transparent: true, opacity: 0.6 })))
  scene.add(particles)

  const trailPool = Array.from({ length: 24 }, () => {
    const positions = new Float32Array(6)
    const trailGeometry = geometry(new THREE.BufferGeometry())
    trailGeometry.setAttribute('position', new THREE.BufferAttribute(positions, 3))
    const trailMaterial = material(new THREE.LineBasicMaterial({ color: 0xb9f9ff, transparent: true, opacity: 0 }))
    const line = new THREE.Line(trailGeometry, trailMaterial)
    line.visible = false
    scene.add(line)
    return { line, geometry: trailGeometry, material: trailMaterial, age: 1 }
  })
  let trailCursor = 0

  const sparkPositions = new Float32Array(16 * 3)
  const sparkVelocities = Array.from({ length: 16 }, () => new THREE.Vector3())
  const sparkGeometry = geometry(new THREE.BufferGeometry())
  sparkGeometry.setAttribute('position', new THREE.BufferAttribute(sparkPositions, 3))
  const sparkMaterial = material(new THREE.PointsMaterial({ color: 0x9ef6ff, size: 0.12, transparent: true, opacity: 0 }))
  const sparks = new THREE.Points(sparkGeometry, sparkMaterial)
  scene.add(sparks)
  let sparkAge = 1

  const simulation = new GameSimulation(Math.random, onChange)
  const targetMeshes = new Map<number, InstanceType<typeof THREE.Group>>()
  let strokePoint: Point | null = null
  let lowPerformance = false

  function makeTarget(target: GameTarget) {
    const group = new THREE.Group()
    if (target.kind === 'bronze') {
      group.add(new THREE.Mesh(bronzeRingGeometry, bronzeMaterial))
      const seal = new THREE.Mesh(targetMarkGeometry, bronzeMaterial)
      seal.rotation.z = Math.PI / 2
      group.add(seal)
    } else {
      group.add(new THREE.Mesh(targetBodyGeometry, jadeMaterial))
      const mark = new THREE.Mesh(targetMarkGeometry, cyanMaterial)
      mark.position.z = 0.16
      mark.rotation.z = target.kind === 'directional' ? Math.PI / 4 : 0
      group.add(mark)
    }
    group.position.set(target.x, target.y, target.z)
    scene.add(group)
    targetMeshes.set(target.id, group)
  }

  function syncTargets(time: number) {
    for (const target of simulation.targets) {
      if (!targetMeshes.has(target.id)) makeTarget(target)
      const group = targetMeshes.get(target.id)
      if (!group) continue
      group.position.set(target.x, target.y + Math.sin(time * 2.1 + target.id) * 0.12, target.z)
      group.rotation.y += 0.018
      group.rotation.z = Math.sin(time * 0.8 + target.id) * 0.12
    }
    for (const [id, group] of targetMeshes) {
      if (simulation.targets.some(target => target.id === id)) continue
      scene.remove(group)
      targetMeshes.delete(id)
    }
  }

  function screenTargets(): ScreenTarget[] {
    const projected = new THREE.Vector3()
    return simulation.targets.flatMap(target => {
      const group = targetMeshes.get(target.id)
      if (!group) return []
      projected.copy(group.position).project(camera)
      const depthScale = Math.max(0.045, 0.14 * (1 - Math.max(-1, Math.min(1, projected.z)) * 0.35))
      return [{ id: target.id, x: (projected.x + 1) / 2, y: (1 - projected.y) / 2, radius: depthScale }]
    })
  }

  function worldOnGameplayPlane(point: Point) {
    const vector = new THREE.Vector3(point.x * 2 - 1, -(point.y * 2 - 1), 0.2).unproject(camera)
    const direction = vector.sub(camera.position).normalize()
    const distance = (-2 - camera.position.z) / direction.z
    return camera.position.clone().add(direction.multiplyScalar(distance))
  }

  function addTrail(start: Point, end: Point) {
    const trail = trailPool[trailCursor++ % trailPool.length]
    if (!trail) return
    const from = worldOnGameplayPlane(start)
    const to = worldOnGameplayPlane(end)
    const attribute = trail.geometry.getAttribute('position') as InstanceType<typeof THREE.BufferAttribute>
    attribute.setXYZ(0, from.x, from.y, from.z)
    attribute.setXYZ(1, to.x, to.y, to.z)
    attribute.needsUpdate = true
    trail.age = 0
    trail.line.visible = true
    trail.material.opacity = 0.92
  }

  function burst(target: GameTarget, blocked: boolean) {
    sparkAge = 0
    sparkMaterial.color.setHex(blocked ? 0xd8a64d : 0x9ef6ff)
    sparkMaterial.opacity = 1
    for (let index = 0; index < 16; index += 1) {
      sparkPositions[index * 3] = target.x
      sparkPositions[index * 3 + 1] = target.y
      sparkPositions[index * 3 + 2] = target.z
      sparkVelocities[index]?.set((Math.random() - 0.5) * 4, (Math.random() - 0.2) * 4, (Math.random() - 0.5) * 2)
    }
    ;(sparkGeometry.getAttribute('position') as InstanceType<typeof THREE.BufferAttribute>).needsUpdate = true
  }

  function interact(event: StageInteraction) {
    if (!simulation.active) return
    const point = { x: event.x, y: event.y }
    if (event.type === 'pointer-down') { strokePoint = point; return }
    if (event.type === 'pointer-up') { strokePoint = null; return }
    if (!strokePoint) strokePoint = point
    const start = strokePoint
    if (Math.hypot(point.x - start.x, point.y - start.y) < 0.012) return
    addTrail(start, point)
    const hits = simulation.slash(start, point, screenTargets())
    hits.forEach(hit => burst(hit.target, hit.blocked))
    strokePoint = point
  }

  function resize(size: StageSize) {
    renderer.setPixelRatio(Math.min(size.dpr, 1.5))
    renderer.setSize(size.width, size.height, false)
    camera.aspect = size.width / size.height
    camera.updateProjectionMatrix()
  }

  resize(initialSize)
  renderer.render(scene, camera)
  onChange({ score: 0, combo: 0, bestCombo: 0, lives: 3 })

  return {
    resize,
    interact,
    setActive(active) { simulation.setActive(active); if (!active) strokePoint = null },
    frame(time, delta) {
      simulation.step(delta)
      syncTargets(time)
      const environmentDelta = delta * (simulation.active ? 1 : 0.2)
      scanGates.forEach((gate, index) => {
        gate.position.z += environmentDelta * 2.2
        if (gate.position.z > 7) gate.position.z = -33 - index * 2
      })
      flowLines.forEach(line => {
        line.position.z += environmentDelta * 8
        if (line.position.z > 8) line.position.z -= 57.6
      })
      lanterns.forEach(lantern => { lantern.rotation.z = Math.sin(time * 1.1 + Number(lantern.userData.phase)) * 0.08 })
      if (!lowPerformance) particles.rotation.y += environmentDelta * 0.018
      for (const trail of trailPool) {
        if (trail.age >= 0.26) continue
        trail.age += delta
        trail.material.opacity = Math.max(0, 1 - trail.age / 0.26)
        if (trail.material.opacity === 0) trail.line.visible = false
      }
      if (sparkAge < 0.42) {
        sparkAge += delta
        for (let index = 0; index < 16; index += 1) {
          const velocity = sparkVelocities[index]
          if (!velocity) continue
          sparkPositions[index * 3] = (sparkPositions[index * 3] ?? 0) + velocity.x * delta
          sparkPositions[index * 3 + 1] = (sparkPositions[index * 3 + 1] ?? 0) + velocity.y * delta
          sparkPositions[index * 3 + 2] = (sparkPositions[index * 3 + 2] ?? 0) + velocity.z * delta
          velocity.y -= 3.2 * delta
        }
        ;(sparkGeometry.getAttribute('position') as InstanceType<typeof THREE.BufferAttribute>).needsUpdate = true
        sparkMaterial.opacity = Math.max(0, 1 - sparkAge / 0.42)
      }
      renderer.render(scene, camera)
    },
    setLowPerformance(enabled) { lowPerformance = enabled; particles.visible = !enabled; renderer.setPixelRatio(1) },
    dispose() {
      simulation.setActive(false)
      targetMeshes.forEach(group => scene.remove(group))
      disposableGeometries.forEach(value => value.dispose())
      disposableMaterials.forEach(value => value.dispose())
      renderer.dispose()
    },
  }
}
