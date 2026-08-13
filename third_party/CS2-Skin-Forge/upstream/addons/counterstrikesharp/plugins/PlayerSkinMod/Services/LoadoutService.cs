using System.Text.Json;
using Microsoft.Extensions.Logging;
using PlayerSkinMod.Models;

namespace PlayerSkinMod.Services;

public static class LoadoutService
{
    public static void LoadFromFile(
        string filePath,
        Dictionary<int, PlayerLoadout> playerLoadouts,
        ILogger logger)
    {
        try
        {
            if (!File.Exists(filePath))
            {
                logger.LogInformation($"[PlayerSkinMod] Loadout file not found: {filePath} (will use random skins)");
                return;
            }

            var json = File.ReadAllText(filePath);
            if (string.IsNullOrWhiteSpace(json))
            {
                logger.LogInformation("[PlayerSkinMod] Loadout file is empty");
                return;
            }

            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;

            foreach (var prop in root.EnumerateObject())
            {
                if (!int.TryParse(prop.Name, out int slot))
                {
                    logger.LogWarning($"[PlayerSkinMod] Skipping non-numeric key: {prop.Name}");
                    continue;
                }

                // Parse each slot independently: one malformed slot must not
                // abort loading the others.
                try
                {
                    var loadout = ParseLoadout(prop.Value);
                    playerLoadouts[slot] = loadout;
                    logger.LogInformation($"[PlayerSkinMod] Loaded loadout for slot {slot}: knifeCT={loadout.KnifeIndexCt}, knifeT={loadout.KnifeIndexT}, gloveCT={loadout.GloveIndexCt}, gloveT={loadout.GloveIndexT}, agentCT={loadout.AgentModelCt}, agentT={loadout.AgentModelT}, music={loadout.MusicKit}, weaponsCT={loadout.WeaponPaintsCt.Count}, weaponsT={loadout.WeaponPaintsT.Count}, keychains={loadout.WeaponKeychains.Count}, random={loadout.UseRandom}");
                }
                catch (Exception ex)
                {
                    logger.LogError($"[PlayerSkinMod] Failed to parse loadout for slot {slot}: {ex.Message}");
                }
            }
        }
        catch (Exception ex)
        {
            logger.LogError($"[PlayerSkinMod] LoadLoadoutFromFile failed: {ex.Message}");
        }
    }

    // ── Safe JSON accessors ─────────────────────────────────────────
    // The loadout file is written by external panel versions old and new;
    // never assume a field has the expected JSON kind.

    private static int GetInt(JsonElement parent, string name, int fallback)
    {
        return parent.TryGetProperty(name, out var el)
            && el.ValueKind == JsonValueKind.Number
            && el.TryGetInt32(out var v)
            ? v
            : fallback;
    }

    private static float GetFloat(JsonElement parent, string name, float fallback)
    {
        return parent.TryGetProperty(name, out var el) && el.ValueKind == JsonValueKind.Number
            ? (float)el.GetDouble()
            : fallback;
    }

    private static ushort GetUShort(JsonElement parent, string name, ushort fallback)
    {
        return parent.TryGetProperty(name, out var el)
            && el.ValueKind == JsonValueKind.Number
            && el.TryGetUInt16(out var v)
            ? v
            : fallback;
    }

    private static string GetString(JsonElement parent, string name, string fallback)
    {
        return parent.TryGetProperty(name, out var el) && el.ValueKind == JsonValueKind.String
            ? el.GetString() ?? fallback
            : fallback;
    }

    private static PlayerLoadout ParseLoadout(JsonElement el)
    {
        var loadout = new PlayerLoadout();

        if (el.TryGetProperty("useRandom", out var useRandomEl)
            && (useRandomEl.ValueKind == JsonValueKind.True || useRandomEl.ValueKind == JsonValueKind.False))
            loadout.UseRandom = useRandomEl.GetBoolean();

        // Legacy shared knife fields (pre-v1.6.0 panels)
        loadout.KnifeIndex = GetInt(el, "knifeIndex", -1);
        loadout.KnifePaint = GetInt(el, "knifePaint", -1);
        loadout.KnifeWear = GetFloat(el, "knifeWear", 0.01f);
        loadout.KnifeSeed = GetInt(el, "knifeSeed", 0);

        // Per-team knives (v1.6.0+)
        loadout.KnifeIndexCt = GetInt(el, "knifeIndexCt", -1);
        loadout.KnifePaintCt = GetInt(el, "knifePaintCt", -1);
        loadout.KnifeWearCt = GetFloat(el, "knifeWearCt", 0.01f);
        loadout.KnifeSeedCt = GetInt(el, "knifeSeedCt", 0);
        loadout.KnifeIndexT = GetInt(el, "knifeIndexT", -1);
        loadout.KnifePaintT = GetInt(el, "knifePaintT", -1);
        loadout.KnifeWearT = GetFloat(el, "knifeWearT", 0.01f);
        loadout.KnifeSeedT = GetInt(el, "knifeSeedT", 0);

        // Backward compat: old shared knife fields → apply to both teams
        if (loadout.KnifeIndexCt < 0 && loadout.KnifeIndexT < 0 && loadout.KnifeIndex >= 0)
        {
            loadout.KnifeIndexCt = loadout.KnifeIndex;
            loadout.KnifeIndexT = loadout.KnifeIndex;
            loadout.KnifePaintCt = loadout.KnifePaint;
            loadout.KnifePaintT = loadout.KnifePaint;
            loadout.KnifeWearCt = loadout.KnifeWear;
            loadout.KnifeWearT = loadout.KnifeWear;
            loadout.KnifeSeedCt = loadout.KnifeSeed;
            loadout.KnifeSeedT = loadout.KnifeSeed;
        }

        // Per-team gloves
        loadout.GloveIndexCt = GetInt(el, "gloveIndexCt", -1);
        loadout.GlovePaintCt = GetInt(el, "glovePaintCt", -1);
        loadout.GloveWearCt = GetFloat(el, "gloveWearCt", 0.01f);
        loadout.GloveSeedCt = GetInt(el, "gloveSeedCt", 0);
        loadout.GloveDefIndexCt = GetUShort(el, "gloveDefIndexCt", 0);
        loadout.GloveIndexT = GetInt(el, "gloveIndexT", -1);
        loadout.GlovePaintT = GetInt(el, "glovePaintT", -1);
        loadout.GloveWearT = GetFloat(el, "gloveWearT", 0.01f);
        loadout.GloveSeedT = GetInt(el, "gloveSeedT", 0);
        loadout.GloveDefIndexT = GetUShort(el, "gloveDefIndexT", 0);

        // Backward compat: old single glove fields → use for both teams
        if (loadout.GloveIndexCt < 0 && loadout.GloveIndexT < 0)
        {
            var oldIdx = GetInt(el, "gloveIndex", -1);
            if (oldIdx >= 0)
            {
                loadout.GloveIndexCt = oldIdx;
                loadout.GloveIndexT = oldIdx;
            }
        }
        if (loadout.GlovePaintCt < 0 && loadout.GlovePaintT < 0)
        {
            var oldPaint = GetInt(el, "glovePaint", -1);
            if (oldPaint >= 0)
            {
                loadout.GlovePaintCt = oldPaint;
                loadout.GlovePaintT = oldPaint;
            }
        }
        // Backward compat: old singular gloveDefIndex (pre-v1.4.0 panels)
        if (loadout.GloveDefIndexCt == 0 && loadout.GloveDefIndexT == 0)
        {
            var oldDef = GetUShort(el, "gloveDefIndex", 0);
            if (oldDef > 0)
            {
                loadout.GloveDefIndexCt = oldDef;
                loadout.GloveDefIndexT = oldDef;
            }
        }
        // For wear/seed backward compat, only apply if old fields exist and new ones are default
        if (el.TryGetProperty("gloveWear", out var oldGloveWearEl) && oldGloveWearEl.ValueKind == JsonValueKind.Number)
        {
            var oldWear = (float)oldGloveWearEl.GetDouble();
            if (loadout.GloveWearCt == 0.01f) loadout.GloveWearCt = oldWear;
            if (loadout.GloveWearT == 0.01f) loadout.GloveWearT = oldWear;
        }
        if (el.TryGetProperty("gloveSeed", out var oldGloveSeedEl) && oldGloveSeedEl.ValueKind == JsonValueKind.Number)
        {
            var oldSeed = oldGloveSeedEl.GetInt32();
            if (loadout.GloveSeedCt == 0) loadout.GloveSeedCt = oldSeed;
            if (loadout.GloveSeedT == 0) loadout.GloveSeedT = oldSeed;
        }

        // Separate CT/T agent models
        loadout.AgentModelCt = GetInt(el, "agentModelCt", -1);
        loadout.AgentModelT = GetInt(el, "agentModelT", -1);
        // Model path for direct lookup (more reliable than index)
        loadout.AgentModelPathCt = GetString(el, "agentModelPathCt", "");
        loadout.AgentModelPathT = GetString(el, "agentModelPathT", "");
        // Backward compat: if old agentModel exists and new fields don't, use old value for both
        if (loadout.AgentModelCt < 0 && loadout.AgentModelT < 0)
        {
            var oldAgent = GetInt(el, "agentModel", -1);
            if (oldAgent >= 0)
            {
                loadout.AgentModelCt = oldAgent;
                loadout.AgentModelT = oldAgent;
            }
        }

        loadout.MusicKit = GetInt(el, "musicKit", -1);

        // Legacy shared weapon maps (pre-v1.6.0 panels)
        ParseUShortIntMap(el, "weaponPaints", loadout.WeaponPaints);
        ParseUShortIntMap(el, "weaponSeeds", loadout.WeaponSeeds);
        ParseUShortFloatMap(el, "weaponWears", loadout.WeaponWears);

        // Per-team weapon skins (v1.6.0+)
        ParseUShortIntMap(el, "weaponPaintsCt", loadout.WeaponPaintsCt);
        ParseUShortIntMap(el, "weaponSeedsCt", loadout.WeaponSeedsCt);
        ParseUShortFloatMap(el, "weaponWearsCt", loadout.WeaponWearsCt);
        ParseUShortIntMap(el, "weaponPaintsT", loadout.WeaponPaintsT);
        ParseUShortIntMap(el, "weaponSeedsT", loadout.WeaponSeedsT);
        ParseUShortFloatMap(el, "weaponWearsT", loadout.WeaponWearsT);

        // Backward compat: old shared weapon maps → apply to both teams
        if (loadout.WeaponPaintsCt.Count == 0 && loadout.WeaponPaintsT.Count == 0 && loadout.WeaponPaints.Count > 0)
        {
            foreach (var kv in loadout.WeaponPaints)
            {
                loadout.WeaponPaintsCt[kv.Key] = kv.Value;
                loadout.WeaponPaintsT[kv.Key] = kv.Value;
            }
            foreach (var kv in loadout.WeaponSeeds)
            {
                loadout.WeaponSeedsCt[kv.Key] = kv.Value;
                loadout.WeaponSeedsT[kv.Key] = kv.Value;
            }
            foreach (var kv in loadout.WeaponWears)
            {
                loadout.WeaponWearsCt[kv.Key] = kv.Value;
                loadout.WeaponWearsT[kv.Key] = kv.Value;
            }
        }

        // Stickers: { defindex: [ { id, offsetX, offsetY, wear, scale, rotation }, ... ] }
        if (el.TryGetProperty("weaponStickers", out var wstEl) && wstEl.ValueKind == JsonValueKind.Object)
        {
            foreach (var ws in wstEl.EnumerateObject())
            {
                if (!ushort.TryParse(ws.Name, out ushort defIdx)) continue;
                if (ws.Value.ValueKind != JsonValueKind.Array) continue;
                var stickerList = new List<StickerInfo>();
                foreach (var sEl in ws.Value.EnumerateArray())
                {
                    if (sEl.ValueKind != JsonValueKind.Object) continue;
                    stickerList.Add(new StickerInfo
                    {
                        Id = (uint)Math.Max(0, GetInt(sEl, "id", 0)),
                        OffsetX = GetFloat(sEl, "offsetX", 0f),
                        OffsetY = GetFloat(sEl, "offsetY", 0f),
                        Wear = GetFloat(sEl, "wear", 0f),
                        Scale = GetFloat(sEl, "scale", 1f),
                        Rotation = GetFloat(sEl, "rotation", 0f),
                    });
                }
                if (stickerList.Count > 0)
                    loadout.WeaponStickers[defIdx] = stickerList;
            }
        }

        // Keychains: { defindex: { id, offsetX, offsetY, offsetZ, seed } }
        if (el.TryGetProperty("weaponKeychains", out var wkEl) && wkEl.ValueKind == JsonValueKind.Object)
        {
            foreach (var wk in wkEl.EnumerateObject())
            {
                if (!ushort.TryParse(wk.Name, out ushort defIdx)) continue;
                if (wk.Value.ValueKind != JsonValueKind.Object) continue;
                var kc = new KeychainInfo
                {
                    Id = (uint)Math.Max(0, GetInt(wk.Value, "id", 0)),
                    OffsetX = GetFloat(wk.Value, "offsetX", 0f),
                    OffsetY = GetFloat(wk.Value, "offsetY", 0f),
                    OffsetZ = GetFloat(wk.Value, "offsetZ", 0f),
                    Seed = GetInt(wk.Value, "seed", 0),
                };
                if (kc.Id > 0)
                    loadout.WeaponKeychains[defIdx] = kc;
            }
        }

        // Nametags: { defindex: "name" }
        if (el.TryGetProperty("weaponNametags", out var wnEl) && wnEl.ValueKind == JsonValueKind.Object)
        {
            foreach (var wn in wnEl.EnumerateObject())
            {
                if (!ushort.TryParse(wn.Name, out ushort defIdx)) continue;
                if (wn.Value.ValueKind != JsonValueKind.String) continue;
                var name = wn.Value.GetString();
                if (!string.IsNullOrEmpty(name))
                    loadout.WeaponNametags[defIdx] = name;
            }
        }

        // StatTrak: { defindex: { enabled, count } }
        if (el.TryGetProperty("weaponStatTrak", out var wst2El) && wst2El.ValueKind == JsonValueKind.Object)
        {
            foreach (var ws in wst2El.EnumerateObject())
            {
                if (!ushort.TryParse(ws.Name, out ushort defIdx)) continue;
                if (ws.Value.ValueKind != JsonValueKind.Object) continue;
                var st = new StatTrakInfo
                {
                    Enabled = ws.Value.TryGetProperty("enabled", out var enEl) && enEl.ValueKind == JsonValueKind.True,
                    Count = GetInt(ws.Value, "count", 0),
                };
                if (st.Enabled)
                    loadout.WeaponStatTrak[defIdx] = st;
            }
        }

        return loadout;
    }

    private static void ParseUShortIntMap(JsonElement parent, string propName, Dictionary<ushort, int> target)
    {
        if (!parent.TryGetProperty(propName, out var mapEl) || mapEl.ValueKind != JsonValueKind.Object) return;
        foreach (var kv in mapEl.EnumerateObject())
        {
            if (!ushort.TryParse(kv.Name, out ushort defIndex)) continue;
            if (kv.Value.ValueKind != JsonValueKind.Number || !kv.Value.TryGetInt32(out var v)) continue;
            if (v >= 0)
                target[defIndex] = v;
        }
    }

    private static void ParseUShortFloatMap(JsonElement parent, string propName, Dictionary<ushort, float> target)
    {
        if (!parent.TryGetProperty(propName, out var mapEl) || mapEl.ValueKind != JsonValueKind.Object) return;
        foreach (var kv in mapEl.EnumerateObject())
        {
            if (!ushort.TryParse(kv.Name, out ushort defIndex)) continue;
            if (kv.Value.ValueKind != JsonValueKind.Number) continue;
            target[defIndex] = (float)kv.Value.GetDouble();
        }
    }

    public static HashSet<(ushort DefIndex, int Paint)> LoadLegacyPaints(string moduleDirectory, ILogger logger)
    {
        var legacyPaints = new HashSet<(ushort DefIndex, int Paint)>();
        try
        {
            var path = Path.Combine(moduleDirectory, "skins_en.json");
            if (!File.Exists(path))
            {
                logger.LogWarning("[PlayerSkinMod] skins_en.json not found; weapon skins may map to the wrong model position");
                return legacyPaints;
            }

            using var doc = JsonDocument.Parse(File.ReadAllText(path));
            foreach (var el in doc.RootElement.EnumerateArray())
            {
                if (!el.TryGetProperty("legacy_model", out var legacyEl)
                    || legacyEl.ValueKind != JsonValueKind.True)
                    continue;
                if (!el.TryGetProperty("weapon_defindex", out var defEl)) continue;
                if (!el.TryGetProperty("paint", out var paintEl)) continue;

                legacyPaints.Add(((ushort)ReadInt(defEl), ReadInt(paintEl)));
            }
        }
        catch (Exception ex)
        {
            logger.LogError($"[PlayerSkinMod] LoadLegacyPaints failed: {ex.Message}");
        }

        return legacyPaints;

        static int ReadInt(JsonElement e) =>
            e.ValueKind == JsonValueKind.Number
                ? e.GetInt32()
                : int.TryParse(e.GetString(), out var v) ? v : 0;
    }
}
