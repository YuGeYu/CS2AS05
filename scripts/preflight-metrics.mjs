export function percentile(values, fraction) {
  if (!Array.isArray(values) || values.length === 0) return null
  const ordered = [...values].sort((left, right) => left - right)
  return ordered[Math.max(0, Math.ceil(ordered.length * fraction) - 1)]
}

export function cpuPercent(previousCpuSeconds, currentCpuSeconds, elapsedSeconds, logicalProcessors) {
  if (![previousCpuSeconds, currentCpuSeconds, elapsedSeconds, logicalProcessors].every(Number.isFinite) || elapsedSeconds <= 0 || logicalProcessors <= 0) return null
  return 100 * (currentCpuSeconds - previousCpuSeconds) / elapsedSeconds / logicalProcessors
}

export function linearSlope(values) {
  if (!Array.isArray(values) || values.length < 2 || values.some(value => !Number.isFinite(value))) return null
  const n = values.length
  const sumX = n * (n + 1) / 2
  const sumY = values.reduce((sum, value) => sum + value, 0)
  const sumXY = values.reduce((sum, value, index) => sum + value * (index + 1), 0)
  const sumXX = n * (n + 1) * (2 * n + 1) / 6
  return (n * sumXY - sumX * sumY) / (n * sumXX - sumX * sumX)
}

export function mergeCandidate(expectedHash, parts) {
  if (!expectedHash || !Array.isArray(parts) || parts.some(part => part?.exeSha256 !== expectedHash)) throw new Error('EXE hash mismatch')
  const sessions = new Set(parts.map(part => part.sessionId).filter(Boolean))
  if (sessions.size > 1) throw new Error('session id mismatch')
  return { exeSha256: expectedHash, sessionId: [...sessions][0] || null, parts }
}

