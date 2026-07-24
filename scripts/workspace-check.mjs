import { readFile } from 'node:fs/promises'
import { resolve, relative } from 'node:path'

const workspaceRoot = process.cwd()
const registryPath = resolve(workspaceRoot, 'config/workspace/projects.json')
const registry = JSON.parse(await readFile(registryPath, 'utf8'))

const projects = registry.projects ?? []

if (projects.length === 0) {
  throw new Error('workspace registry must contain at least one project')
}

const seen = new Map()

for (const project of projects) {
  for (const key of ['id', 'name', 'rootDir', 'distDir', 'bundleDir', 'logDir']) {
    if (!project[key] || typeof project[key] !== 'string') {
      throw new Error(`project ${project.id ?? '<unknown>'} is missing ${key}`)
    }
  }

  const normalizedRoot = resolve(workspaceRoot, project.rootDir)
  const normalizedDist = resolve(workspaceRoot, project.distDir)
  const normalizedBundle = resolve(workspaceRoot, project.bundleDir)
  const normalizedLog = resolve(workspaceRoot, project.logDir)

  const keyMap = {
    id: project.id,
    rootDir: normalizedRoot,
    distDir: normalizedDist,
    bundleDir: normalizedBundle,
    logDir: normalizedLog,
  }

  for (const [field, value] of Object.entries(keyMap)) {
    const bucket = `${field}:${value.toLowerCase()}`
    const previous = seen.get(bucket)
    if (previous) {
      throw new Error(`${field} conflict between ${previous} and ${project.id}`)
    }
    seen.set(bucket, project.id)
  }

  for (const [label, target] of Object.entries({
    rootDir: normalizedRoot,
    distDir: normalizedDist,
    bundleDir: normalizedBundle,
    logDir: normalizedLog,
  })) {
    const rel = relative(workspaceRoot, target)
    if (rel.startsWith('..') || rel === '') {
      throw new Error(`${project.id} has invalid ${label}: ${target}`)
    }
  }

  if (normalizedDist.startsWith(normalizedRoot) || normalizedBundle.startsWith(normalizedRoot)) {
    throw new Error(`${project.id} mixes build artifacts into the project source root`)
  }
}

console.log(`workspace registry valid: ${projects.length} projects`)
