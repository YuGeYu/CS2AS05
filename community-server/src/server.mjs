import http from 'node:http'
import fs from 'node:fs'
import path from 'node:path'
import crypto from 'node:crypto'
import { DatabaseSync } from 'node:sqlite'
import { WebSocketServer } from 'ws'

const port = Number(process.env.COMMUNITY_PORT || 8787)
const dataDir = process.env.COMMUNITY_DATA_DIR || path.resolve('data')
const uploadDir = path.join(dataDir, 'uploads')
const secret = process.env.COMMUNITY_TOKEN_SECRET || ''
const maxMembers = Number(process.env.COMMUNITY_MAX_ROOM_MEMBERS || 64)
const maxUploadBytes = 10 * 1024 * 1024
const donationImagePath = process.env.COMMUNITY_DONATION_IMAGE || ''
const donationText = process.env.COMMUNITY_DONATION_TEXT || '我吃不起饭了，求哥哥姐姐美女帅哥救助，无论是否有人赞助，都感谢大家一直支持和理解我。'
fs.mkdirSync(uploadDir, { recursive: true })
const dbPath = path.join(dataDir, 'community.sqlite')
const db = new DatabaseSync(dbPath)
db.exec(`CREATE TABLE IF NOT EXISTS messages (id TEXT PRIMARY KEY, room TEXT NOT NULL, user_id TEXT NOT NULL, display_name TEXT NOT NULL, kind TEXT NOT NULL, content TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, expires_at INTEGER NOT NULL); CREATE INDEX IF NOT EXISTS messages_expiry ON messages(expires_at); CREATE TABLE IF NOT EXISTS daily_jobs (job_key TEXT PRIMARY KEY, created_at INTEGER NOT NULL);`)
try { db.exec("ALTER TABLE messages ADD COLUMN role TEXT NOT NULL DEFAULT 'user'") } catch {}

function b64(value) { return Buffer.from(value).toString('base64url') }
function verifyToken(token) {
  if (!secret) throw new Error('token secret is not configured')
  const parts = String(token || '').split('.')
  if (parts.length !== 3) throw new Error('invalid token')
  const expected = crypto.createHmac('sha256', secret).update(`${parts[0]}.${parts[1]}`).digest('base64url')
  if (!crypto.timingSafeEqual(Buffer.from(expected), Buffer.from(parts[2]))) throw new Error('invalid token')
  const payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'))
  if (!payload.sub || Number(payload.exp) <= Math.floor(Date.now() / 1000)) throw new Error('expired token')
  return payload
}
function send(ws, value) { if (ws.readyState === ws.OPEN) ws.send(JSON.stringify(value)) }
function normalize(text) {
  const value = String(text || '').trim()
  const match = value.match(/^\s*(?:connect)?\s*\[([A-Za-z]:\d+:\d+:\d+)\]\s*\((\d+)\)\s*$/i)
  return match ? `connect [${match[1]}] (${match[2]})` : value
}
function contentDisposition(originalName, extension) {
  const cleaned = String(originalName || `download${extension || ''}`).replace(/[\r\n\0-\x1F\x7F"\\]/g, '_').trim() || `download${extension || ''}`
  const fallback = cleaned.replace(/[^\x20-\x7E]/g, '_').replace(/[;:]/g, '_').slice(0, 120) || `download${extension || ''}`
  const encoded = encodeURIComponent(cleaned).replace(/[!'()*]/g, character => `%${character.charCodeAt(0).toString(16).toUpperCase()}`)
  return `attachment; filename="${fallback}"; filename*=UTF-8''${encoded}`
}
function cleanup() { const now = Date.now(); db.prepare('DELETE FROM messages WHERE expires_at <= ?').run(now); const live = new Set(db.prepare("SELECT content FROM messages WHERE kind = 'file' AND expires_at > ?").all(now).flatMap(row => { try { const url = JSON.parse(row.content).url; return url ? [path.basename(url)] : [] } catch { return [] } })); for (const file of fs.readdirSync(uploadDir)) { const full = path.join(uploadDir, file); try { if (!live.has(file)) fs.unlinkSync(full) } catch {} } }
setInterval(cleanup, 15 * 60_000).unref()
const rooms = new Map()
// 一个玩家可能同时打开多个助手窗口；在线人数按账号去重，避免“只有我却显示 2 人在线”。
// 仅统计仍处于 OPEN 状态的连接，并将 sub 统一转成字符串，避免同一账号
// 在不同令牌/序列化路径下出现 number/string 两个值而被 Set 误判为两人。
function distinctMemberCount(members) {
  return new Set([...members]
    .filter(peer => peer.readyState === peer.OPEN && peer.userId != null)
    .map(peer => String(peer.userId).trim())
    .filter(Boolean)).size
}
function authFromRequest(req) { const token = req.headers.authorization?.replace(/^Bearer\s+/i, '') || new URL(req.url, `http://${req.headers.host}`).searchParams.get('token'); return verifyToken(token) }
function json(res, status, body) { res.writeHead(status, { 'content-type': 'application/json; charset=utf-8', 'access-control-allow-origin': '*' }); res.end(JSON.stringify(body)) }
const server = http.createServer(async (req, res) => {
  try {
    const url = new URL(req.url, `http://${req.headers.host}`)
    // The reverse proxy may preserve or strip the public /community prefix.
    const routePath = url.pathname.replace(/^\/community(?=\/|$)/, '') || '/'
    url.pathname = routePath
    if (req.method === 'OPTIONS') { res.writeHead(204, { 'access-control-allow-origin': '*', 'access-control-allow-methods': 'GET,POST,OPTIONS', 'access-control-allow-headers': 'authorization,content-type,x-file-name' }); return res.end() }
    if (req.method === 'GET' && (url.pathname === '/health' || routePath === '/health')) return json(res, 200, { service: 'cs2as-community', ok: true })
    if (req.method === 'POST' && (url.pathname === '/upload' || routePath === '/upload')) {
      const user = authFromRequest(req); const declaredSize = Number(req.headers['content-length'] || 0)
      if (declaredSize > maxUploadBytes) return json(res, 413, { error: '文件不能超过 10 MB' })
      const chunks = []; let total = 0
      for await (const chunk of req) { total += chunk.length; if (total > maxUploadBytes) return json(res, 413, { error: '文件超过 10 MB' }); chunks.push(chunk) }
      if (total <= 0) return json(res, 400, { error: '文件不能为空' })
      const id = crypto.randomUUID(); let rawName = String(req.headers['x-file-name'] || 'resource.bin'); try { rawName = decodeURIComponent(rawName) } catch {} const safeName = rawName.replace(/[^\w.\-\u4e00-\u9fff]/g, '_').slice(0, 120); const filename = `${id}-${safeName}`
      fs.writeFileSync(path.join(uploadDir, filename), Buffer.concat(chunks)); const now = Date.now(); const mime = String(req.headers['content-type'] || 'application/octet-stream').split(';')[0]; const row = { id, room: 'lobby', user_id: user.sub, display_name: user.displayName, kind: 'file', content: JSON.stringify({ name: safeName, size: total, type: mime, url: `/file/${filename}` }), created_at: now, updated_at: now, expires_at: now + 40 * 3600_000, role: user.role || 'user' }
      db.prepare('INSERT INTO messages VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(...Object.values(row)); broadcast('lobby', { type: 'message', message: row }); return json(res, 201, { id, name: safeName, size: total, url: `/file/${filename}` })
    }
    if (req.method === 'GET' && url.pathname.startsWith('/file/')) { authFromRequest(req); let filename; try { filename = path.basename(decodeURIComponent(url.pathname.slice(6))) } catch { return json(res, 400, { error: '文件链接格式无效' }) } let full = path.resolve(uploadDir, filename); if (full.startsWith(path.resolve(uploadDir) + path.sep) && !fs.existsSync(full)) { const prefix = filename.split('-')[0]; const alternate = fs.readdirSync(uploadDir).find(file => file.startsWith(`${prefix}-`)); if (alternate) full = path.join(uploadDir, alternate) } if (!full.startsWith(path.resolve(uploadDir) + path.sep) || !fs.existsSync(full)) return json(res, 404, { error: '文件不存在或已过期' }); const row = db.prepare('SELECT content FROM messages WHERE kind = \'file\' AND content LIKE ? ORDER BY created_at DESC LIMIT 1').get(`%/file/${filename}%`); let originalName = filename; let mime = 'application/octet-stream'; try { const resource = JSON.parse(row?.content || '{}'); originalName = resource.name || originalName; mime = resource.type || mime } catch {} const ext = path.extname(originalName).toLowerCase() || path.extname(filename).toLowerCase(); const known = { '.png': 'image/png', '.jpg': 'image/jpeg', '.jpeg': 'image/jpeg', '.gif': 'image/gif', '.webp': 'image/webp', '.bmp': 'image/bmp', '.svg': 'image/svg+xml', '.mp4': 'video/mp4', '.webm': 'video/webm', '.mov': 'video/quicktime', '.txt': 'text/plain', '.md': 'text/markdown', '.json': 'application/json' }; if (mime === 'application/octet-stream' && known[ext]) mime = known[ext]; const download = url.searchParams.get('download') === '1'; res.writeHead(200, { 'content-type': mime, ...(download ? { 'content-disposition': contentDisposition(originalName, ext) } : {}), 'cache-control': 'private, max-age=60', 'accept-ranges': 'bytes' }); return fs.createReadStream(full).pipe(res) }
    return json(res, 200, { service: 'cs2as-community', ok: true })
  } catch (error) { return json(res, 401, { error: error instanceof Error ? error.message : 'unauthorized' }) }
})
function broadcast(room, value) { for (const peer of rooms.get(room) || []) send(peer, value) }
function dailyDonation() { const key = new Date().toLocaleDateString('en-CA', { timeZone: 'Asia/Shanghai' }); if (db.prepare('SELECT 1 FROM daily_jobs WHERE job_key = ?').get(`donation:${key}`)) return; const now = Date.now(); const textRow = { id: crypto.randomUUID(), room: 'lobby', user_id: 'community-bot', display_name: '圈子机器人', kind: 'text', content: donationText, created_at: now, updated_at: now, expires_at: now + 40 * 3600_000, role: 'bot' }; db.prepare('INSERT INTO messages VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(...Object.values(textRow)); broadcast('lobby', { type: 'message', message: textRow }); if (donationImagePath && fs.existsSync(donationImagePath)) { const name = `donation-${key}.png`; fs.copyFileSync(donationImagePath, path.join(uploadDir, name)); const imageRow = { ...textRow, id: crypto.randomUUID(), kind: 'file', content: JSON.stringify({ name: '微信赞赏码.png', size: fs.statSync(donationImagePath).size, url: `/file/${name}` }) }; db.prepare('INSERT INTO messages VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(...Object.values(imageRow)); broadcast('lobby', { type: 'message', message: imageRow }) } db.prepare('INSERT INTO daily_jobs VALUES (?, ?)').run(`donation:${key}`, now) }
setInterval(dailyDonation, 60_000).unref(); dailyDonation()
const wss = new WebSocketServer({ server, maxPayload: 12 * 1024 * 1024 })
wss.on('connection', (ws, req) => {
  const url = new URL(req.url, `http://${req.headers.host}`)
  let user
  try { user = verifyToken(url.searchParams.get('token')) } catch { ws.close(1008, 'unauthorized'); return }
  const room = url.searchParams.get('room') || 'lobby'
  const members = rooms.get(room) || new Set()
  if (members.size >= maxMembers) { ws.close(1013, 'room full'); return }
  members.add(ws); rooms.set(room, members)
  ws.userId = user.sub; ws.displayName = user.displayName
  send(ws, { type: 'ready', room, user: { id: user.sub, displayName: user.displayName, role: user.role || 'user' } })
  for (const peer of members) if (peer !== ws) { send(ws, { type: 'peer-joined', id: peer.userId, displayName: peer.displayName }); send(peer, { type: 'peer-joined', id: ws.userId, displayName: user.displayName }) }
  const rows = db.prepare('SELECT * FROM messages WHERE room = ? AND expires_at > ? ORDER BY created_at ASC LIMIT 200').all(room, Date.now())
  send(ws, { type: 'history', messages: rows })
  for (const peer of members) send(peer, { type: 'presence', count: distinctMemberCount(members) })
  ws.on('message', raw => { let message; try { message = JSON.parse(raw.toString()) } catch { return }
    if (message.type === 'chat') { const content = normalize(message.content); if (!content || content.length > 4000) return; const now = Date.now(); const row = { id: crypto.randomUUID(), room, user_id: user.sub, display_name: user.displayName, kind: 'text', content, created_at: now, updated_at: now, expires_at: now + 40 * 3600_000, role: user.role || 'user' }; db.prepare('INSERT INTO messages VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)').run(...Object.values(row)); for (const peer of members) send(peer, { type: 'message', message: row }) }
    if (message.type === 'edit') { const content = normalize(message.content); if (!content || content.length > 4000) return; const result = db.prepare('UPDATE messages SET content = ?, updated_at = ? WHERE id = ? AND user_id = ? AND expires_at > ?').run(content, Date.now(), message.id, user.sub, Date.now()); if (result.changes) for (const peer of members) send(peer, { type: 'message-edited', id: message.id, content }) }
    if (message.type === 'reaction' && message.id && message.emoji) { for (const peer of members) send(peer, { type: 'reaction', id: message.id, emoji: String(message.emoji).slice(0, 12), userId: user.sub }) }
    if (message.type === 'delete' && message.id) {
      const target = db.prepare('SELECT * FROM messages WHERE id = ? AND room = ?').get(message.id, room)
      const canDelete = target && (target.user_id === user.sub || user.role === 'owner' || (user.role === 'admin' && !['admin', 'owner'].includes(target.role || 'user')))
      if (canDelete) { try { if (target.kind === 'file') { const resource = JSON.parse(target.content); if (resource.url) fs.rmSync(path.join(uploadDir, path.basename(resource.url)), { force: true }) } } catch {} db.prepare('DELETE FROM messages WHERE id = ?').run(message.id); broadcast(room, { type: 'message-deleted', id: message.id }) }
    }
    if (message.type === 'signal' && message.to) { for (const peer of members) if (peer !== ws && peer.userId === message.to) send(peer, { type: 'signal', from: user.sub, payload: message.payload }) }
  })
  ws.on('close', () => { members.delete(ws); for (const peer of members) { send(peer, { type: 'peer-left', id: ws.userId }); send(peer, { type: 'presence', count: distinctMemberCount(members) }) }; if (!members.size) rooms.delete(room) })
})
server.listen(port, () => console.log(`CS2AS community listening on ${port}`))
