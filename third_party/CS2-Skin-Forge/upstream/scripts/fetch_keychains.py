import json, urllib.request

def fetch(url):
    req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.loads(r.read())

BASE = "https://raw.githubusercontent.com/ByMykel/CSGO-API/main/public/api"

# Fetch EN keychains
print("Fetching EN keychains...")
en_data = fetch(f"{BASE}/en/keychains.json")
print(f"  EN: {len(en_data)}")

# Fetch ZH keychains
print("Fetching ZH keychains...")
zh_data = fetch(f"{BASE}/zh-CN/keychains.json")
print(f"  ZH: {len(zh_data)}")

# Build ZH name map
zh_names = {}
for k in zh_data:
    kid = k.get('def_index', k.get('id', ''))
    zh_names[str(kid)] = k.get('name', '')

# Generate TypeScript
lines = []
lines.append('// Auto-generated from ByMykel CSGO-API (keychains.json)')
lines.append('// Source: https://github.com/ByMykel/CSGO-API')
lines.append('')
lines.append('export interface KeychainData {')
lines.append('  id: number;')
lines.append('  name: string;')
lines.append('  nameZh: string;')
lines.append('  image: string;')
lines.append('}')
lines.append('')
lines.append('export const allKeychains: KeychainData[] = [')

for k in en_data:
    kid = int(k.get('def_index', '0'))
    name_en = k.get('name', '').replace('"', '\\"')
    image = k.get('image', '').replace('"', '\\"')
    name_zh = zh_names.get(str(kid), name_en).replace('"', '\\"')
    lines.append(f'  {{ id: {kid}, name: "{name_en}", nameZh: "{name_zh}", image: "{image}" }},')

lines.append('];')
lines.append('')
lines.append('export function getKeychainImageUrl(image: string): string {')
lines.append('  if (image.startsWith("http")) return image;')
lines.append('  return `https://cdn.steamstatic.com/apps/730/icons/${image}.png`;')
lines.append('}')
lines.append('')

content = '\n'.join(lines)

with open('C:/Users/emptysuns/CS2-Skin-Forge/Panel/src/data/keychains.ts', 'w', encoding='utf-8') as f:
    f.write(content)

print(f"\nGenerated keychains.ts: {len(en_data)} keychains")
# Show first entry
print(f"Sample: id={en_data[0].get('def_index')}, name={en_data[0].get('name')}, image={en_data[0].get('image','')[:80]}...")
