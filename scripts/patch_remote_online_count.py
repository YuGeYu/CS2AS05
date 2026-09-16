import base64
import paramiko

host = "8.133.185.37"
password = "LBroot100728."
script = r'''from pathlib import Path
p = Path("/opt/cs2as-community/src/server.mjs")
s = p.read_text()
old1 = "const rooms = new Map()\nfunction authFromRequest"
new1 = "const rooms = new Map()\n// Count unique authenticated accounts, not browser/window connections.\nfunction distinctMemberCount(members) { return new Set([...members].map(peer => peer.userId).filter(Boolean)).size }\nfunction authFromRequest"
old2 = "for (const peer of members) send(peer, { type: 'presence', count: members.size })"
new2 = "for (const peer of members) send(peer, { type: 'presence', count: distinctMemberCount(members) })"
old3 = "send(peer, { type: 'presence', count: members.size })"
new3 = "send(peer, { type: 'presence', count: distinctMemberCount(members) })"
if "function distinctMemberCount" in s:
    raise SystemExit("already patched")
if old1 not in s or old2 not in s or old3 not in s:
    raise SystemExit("expected snippets missing")
backup = p.with_name("server.mjs.backup-online-count-20260914")
backup.write_bytes(p.read_bytes())
s = s.replace(old1, new1, 1).replace(old2, new2, 1).replace(old3, new3, 1)
p.write_text(s)
print("patched", p, "backup", backup)
'''

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(host, username="root", password=password, timeout=12)
encoded = base64.b64encode(script.encode()).decode()
cmd = f"echo {encoded} | base64 -d | python3; docker restart cs2as-community; sleep 2; docker logs --tail 5 cs2as-community"
stdin, stdout, stderr = client.exec_command(cmd)
print(stdout.read().decode())
print(stderr.read().decode())
stdin, stdout, stderr = client.exec_command("docker exec cs2as-community grep -n distinctMemberCount /app/src/server.mjs")
print(stdout.read().decode())
print(stderr.read().decode())
client.close()
