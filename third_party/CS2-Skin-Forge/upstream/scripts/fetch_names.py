import json, urllib.request, sys, os

def fetch(url):
    req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.loads(r.read())

BASE = "https://raw.githubusercontent.com/ByMykel/CSGO-API/main/public/api"

# Fetch EN agents
print("Fetching EN agents...")
en_agents = fetch(f"{BASE}/en/agents.json")
ct_en = [d for d in en_agents if d.get('team',{}).get('id') == 'ct']
t_en = [d for d in en_agents if d.get('team',{}).get('id') == 't']
print(f"  EN: CT={len(ct_en)}, T={len(t_en)}")

# Fetch ZH agents  
print("Fetching ZH agents...")
zh_agents = fetch(f"{BASE}/zh-CN/agents.json")
ct_zh = [d for d in zh_agents if d.get('team',{}).get('id') == 'ct']
t_zh = [d for d in zh_agents if d.get('team',{}).get('id') == 't']
print(f"  ZH: CT={len(ct_zh)}, T={len(t_zh)}")

# Build agent name mapping by id: {id: {en: name, zh: name}}
agent_names = {}
for a in en_agents:
    aid = a.get('id','')
    agent_names[aid] = {'en': a.get('name',''), 'model': a.get('model','')}
for a in zh_agents:
    aid = a.get('id','')
    if aid in agent_names:
        agent_names[aid]['zh'] = a.get('name','')
    else:
        agent_names[aid] = {'zh': a.get('name',''), 'model': a.get('model','')}

# Fetch EN music kits
print("Fetching EN music kits...")
en_kits = fetch(f"{BASE}/en/music_kits.json")
print(f"  EN: {len(en_kits)}")

# Fetch ZH music kits
print("Fetching ZH music kits...")
zh_kits = fetch(f"{BASE}/zh-CN/music_kits.json")
print(f"  ZH: {len(zh_kits)}")

# Build music kit name mapping
kit_names = {}
for k in en_kits:
    kid = k.get('id','')
    kit_names[kid] = {'en': k.get('name','')}
for k in zh_kits:
    kid = k.get('id','')
    if kid in kit_names:
        kit_names[kid]['zh'] = k.get('name','')
    else:
        kit_names[kid] = {'zh': k.get('name','')}

# Output as JSON
output = {
    'agent_names': agent_names,
    'kit_names': kit_names
}
with open('C:/Users/emptysuns/CS2-Skin-Forge/Panel/src/data/nameMap.json', 'w', encoding='utf-8') as f:
    json.dump(output, f, ensure_ascii=False, indent=2)

print(f"\nSaved nameMap.json with {len(agent_names)} agents and {len(kit_names)} music kits")

# Show some samples
print("\nSample agents:")
for aid, names in list(agent_names.items())[:3]:
    print(f"  {aid}: en='{names.get('en','?')}', zh='{names.get('zh','?')}'")
print("\nSample music kits:")
for kid, names in list(kit_names.items())[:3]:
    print(f"  {kid}: en='{names.get('en','?')}', zh='{names.get('zh','?')}'")
