using CounterStrikeSharp.API;
using CounterStrikeSharp.API.Core;
using CounterStrikeSharp.API.Modules.Memory.DynamicFunctions;
using Microsoft.Extensions.Logging;
using PlayerSkinMod.Data;
using PlayerSkinMod.Models;

namespace PlayerSkinMod.Services;

public static class WeaponService
{
    private static ulong _nextItemId = 0xF00DCAFE;
    private static bool _skinErrorLogged;

    public static CCSPlayerController? GetPlayerFromItemServices(CCSPlayer_ItemServices itemServices)
    {
        var pawn = itemServices.Pawn.Value;
        if (pawn == null || !pawn.IsValid || pawn.Controller.Value == null || !pawn.Controller.IsValid)
            return null;

        var player = new CCSPlayerController(pawn.Controller.Value.Handle);
        return player.IsValid ? player : null;
    }

    public static float UIntToFloat(uint value) => BitConverter.Int32BitsToSingle((int)value);

    public static void ApplyStickers(
        CBasePlayerWeapon weapon,
        List<StickerInfo> stickers,
        MemoryFunctionVoid<nint, string, float> setAttrByName)
    {
        if (stickers.Count == 0) return;
        var item = weapon.AttributeManager?.Item;
        if (item == null) return;

        for (int i = 0; i < Math.Min(stickers.Count, 5); i++)
        {
            var sticker = stickers[i];
            if (sticker.Id == 0) continue;
            var handle = item.NetworkedDynamicAttributes.Handle;
            setAttrByName.Invoke(handle, $"sticker slot {i} id", UIntToFloat(sticker.Id));
            if (sticker.OffsetX != 0 || sticker.OffsetY != 0)
                setAttrByName.Invoke(handle, $"sticker slot {i} schema", 0f);
            setAttrByName.Invoke(handle, $"sticker slot {i} offset x", sticker.OffsetX);
            setAttrByName.Invoke(handle, $"sticker slot {i} offset y", sticker.OffsetY);
            setAttrByName.Invoke(handle, $"sticker slot {i} wear", sticker.Wear);
            setAttrByName.Invoke(handle, $"sticker slot {i} scale", sticker.Scale);
            setAttrByName.Invoke(handle, $"sticker slot {i} rotation", sticker.Rotation);
        }
    }

    public static void ApplyKeychains(
        CBasePlayerWeapon weapon,
        KeychainInfo keychain,
        MemoryFunctionVoid<nint, string, float> setAttrByName)
    {
        if (keychain.Id == 0) return;
        var item = weapon.AttributeManager?.Item;
        if (item == null) return;

        var handle = item.NetworkedDynamicAttributes.Handle;
        setAttrByName.Invoke(handle, "keychain slot 0 id", UIntToFloat(keychain.Id));
        setAttrByName.Invoke(handle, "keychain slot 0 offset x", keychain.OffsetX);
        setAttrByName.Invoke(handle, "keychain slot 0 offset y", keychain.OffsetY);
        setAttrByName.Invoke(handle, "keychain slot 0 offset z", keychain.OffsetZ);
        if (keychain.Seed > 0)
            setAttrByName.Invoke(handle, "keychain slot 0 seed", (float)keychain.Seed);
    }

    /// <summary>
    /// Apply a paint kit (plus optional nametag / StatTrak) to a weapon.
    /// Single implementation shared by the GiveNamedItem hook and the
    /// respawn re-apply pass in the plugin.
    /// </summary>
    public static void ApplySkinToWeapon(
        CEconEntity weapon,
        ushort defIndex,
        int paintKit,
        HashSet<(ushort DefIndex, int Paint)> legacyPaints,
        MemoryFunctionVoid<nint, string, float> setAttrByName,
        int seed = 0,
        float wear = 0.01f,
        uint accountId = 0,
        string? nametag = null,
        StatTrakInfo? statTrak = null,
        ILogger? logger = null)
    {
        try
        {
            var item = weapon.AttributeManager?.Item;
            if (item == null) return;

            item.AttributeList.Attributes.RemoveAll();
            item.NetworkedDynamicAttributes.Attributes.RemoveAll();
            AssignItemId(item);
            if (accountId > 0) item.AccountID = accountId;

            weapon.FallbackPaintKit = paintKit;
            weapon.FallbackSeed = seed;
            weapon.FallbackWear = wear;

            // Mark fallback netvars dirty so they are (re)sent to clients.
            // Without this, whether the client sees the skin depends on whether the
            // initial entity snapshot happened to include these values — which is
            // why skins would intermittently render as the default texture even
            // though the inspect description showed the custom skin.
            Utilities.SetStateChanged(weapon, "CEconEntity", "m_nFallbackPaintKit");
            Utilities.SetStateChanged(weapon, "CEconEntity", "m_nFallbackSeed");
            Utilities.SetStateChanged(weapon, "CEconEntity", "m_flFallbackWear");

            setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture prefab", paintKit);
            setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture seed", (float)seed);
            setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture wear", wear);

            setAttrByName.Invoke(item.AttributeList.Handle, "set item texture prefab", paintKit);
            setAttrByName.Invoke(item.AttributeList.Handle, "set item texture seed", (float)seed);
            setAttrByName.Invoke(item.AttributeList.Handle, "set item texture wear", wear);

            // Apply nametag
            if (!string.IsNullOrEmpty(nametag))
            {
                item.CustomName = nametag;
            }

            // Apply StatTrak. The kill count must be written as raw uint bits
            // reinterpreted as float ("kill eater" attributes store integers in
            // float storage). Passing a plain float here makes the client read
            // garbage bits, which rendered as the capped 99999 display value.
            if (statTrak != null && statTrak.Enabled)
            {
                uint count = (uint)Math.Max(0, statTrak.Count);
                item.EntityQuality = 9; // StatTrak quality
                weapon.FallbackStatTrak = (int)count;
                Utilities.SetStateChanged(weapon, "CEconEntity", "m_nFallbackStatTrak");
                setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "kill eater", UIntToFloat(count));
                setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "kill eater score type", UIntToFloat(0));
                setAttrByName.Invoke(item.AttributeList.Handle, "kill eater", UIntToFloat(count));
                setAttrByName.Invoke(item.AttributeList.Handle, "kill eater score type", UIntToFloat(0));
            }

            Utilities.SetStateChanged(weapon, "CEconEntity", "m_AttributeManager");

            bool isLegacy = legacyPaints.Contains((defIndex, paintKit));
            weapon.AcceptInput("SetBodygroup", value: $"body,{(isLegacy ? 1 : 0)}");
            Utilities.SetStateChanged(weapon, "CBaseModelEntity", "m_CBodyComponent");
        }
        catch (Exception ex)
        {
            // Log only once — this fires per weapon per spawn and would spam.
            if (!_skinErrorLogged)
            {
                _skinErrorLogged = true;
                logger?.LogError($"[PlayerSkinMod] ApplySkinToWeapon failed: {ex.Message}");
            }
        }
    }

    /// <summary>
    /// Replace the player's knife with the specified knife type and skin.
    /// Aligned with Nereziel/cs2-WeaponPaints approach:
    ///   ChangeSubclass → set defindex → apply skin → force slot3
    /// The slot3 command forces the client to rebuild the view-model,
    /// which picks up the correct AnimGraph2 for the new knife family.
    /// </summary>
    public static void ReplaceKnife(
        CCSPlayerController player,
        CCSPlayerPawn pawn,
        ushort defIndex,
        int paintKit,
        HashSet<(ushort DefIndex, int Paint)> legacyPaints,
        MemoryFunctionVoid<nint, string, float> setAttrByName,
        int knifeSeed = 0,
        float knifeWear = 0.01f)
    {
        try
        {
            var weapons = pawn.WeaponServices?.MyWeapons;
            if (weapons == null) return;

            foreach (var handle in weapons)
            {
                var w = handle.Value;
                if (w == null || !w.IsValid) continue;
                var name = w.DesignerName;
                if (string.IsNullOrEmpty(name)) continue;
                if (name != "weapon_knife" && name != "weapon_knife_t") continue;

                bool typeChanged = w.AttributeManager?.Item?.ItemDefinitionIndex != defIndex;

                // ChangeSubclass swaps the weapon's entity class (model + VData)
                w.AcceptInput("ChangeSubclass", value: defIndex.ToString());

                var item = w.AttributeManager?.Item;
                if (item == null) break;

                item.ItemDefinitionIndex = defIndex;
                item.EntityQuality = 3;

                item.AttributeList.Attributes.RemoveAll();
                item.NetworkedDynamicAttributes.Attributes.RemoveAll();

                AssignItemId(item);

                if (paintKit > 0)
                {
                    w.FallbackPaintKit = paintKit;
                    w.FallbackSeed = knifeSeed;
                    w.FallbackWear = knifeWear;
                    Utilities.SetStateChanged(w, "CEconEntity", "m_nFallbackPaintKit");
                    Utilities.SetStateChanged(w, "CEconEntity", "m_nFallbackSeed");
                    Utilities.SetStateChanged(w, "CEconEntity", "m_flFallbackWear");

                    setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture prefab", paintKit);
                    setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture seed", (float)knifeSeed);
                    setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture wear", knifeWear);

                    setAttrByName.Invoke(item.AttributeList.Handle, "set item texture prefab", paintKit);
                    setAttrByName.Invoke(item.AttributeList.Handle, "set item texture seed", (float)knifeSeed);
                    setAttrByName.Invoke(item.AttributeList.Handle, "set item texture wear", knifeWear);
                }

                Utilities.SetStateChanged(w, "CEconEntity", "m_AttributeManager");

                bool isLegacy = legacyPaints.Contains((defIndex, paintKit));
                w.AcceptInput("SetBodygroup", value: $"body,{(isLegacy ? 1 : 0)}");
                Utilities.SetStateChanged(w, "CBaseModelEntity", "m_CBodyComponent");

                // When the knife family changes (e.g. Butterfly → Talon),
                // force the client to re-select the knife slot so the engine
                // rebuilds the view-model with the correct animation graph.
                if (typeChanged && player != null && player.IsValid)
                    player.ExecuteClientCommand("slot3");

                break;
            }
        }
        catch
        {
            // Knife replacement failed silently — common with invalid defindex or timing issues
        }
    }

    /// <summary>
    /// Apply gloves to a player pawn.
    ///   1. Clear stale attributes
    ///   2. Set defindex + skin attributes on both attribute lists
    ///   3. Notify engine via SetStateChanged (critical for rendering)
    ///   4. Mark Initialized = true
    ///   5. Bodygroup toggle 0 -> 1 (with 0.2f delay) to force material refresh
    /// Note: "lastinv" is intentionally NOT used — it can interfere with the
    /// glove model swap on certain team/model combinations.
    /// </summary>
    public static void ApplyGloves(
        CCSPlayerController player,
        CCSPlayerPawn pawn,
        ushort defIndex,
        int paintKit,
        MemoryFunctionVoid<nint, string, float> setAttrByName,
        int seed = 0,
        float wear = 0.01f,
        Action<float, Action>? addTimer = null)
    {
        try
        {
            var item = pawn.EconGloves;
            if (item == null)
            {
                player.PrintToConsole($"[PlayerSkinMod] EconGloves is null for {player.PlayerName}");
                return;
            }

            // Clear stale attributes from previous glove type
            item.NetworkedDynamicAttributes.Attributes.RemoveAll();
            item.AttributeList.Attributes.RemoveAll();

            // Set glove identity — drives which 3D model + UV layout is used
            item.ItemDefinitionIndex = defIndex;
            AssignItemId(item);

            // Apply skin attributes on BOTH lists (critical for CS2 rendering)
            setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture prefab", paintKit);
            setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture seed", (float)seed);
            setAttrByName.Invoke(item.NetworkedDynamicAttributes.Handle, "set item texture wear", wear);

            setAttrByName.Invoke(item.AttributeList.Handle, "set item texture prefab", paintKit);
            setAttrByName.Invoke(item.AttributeList.Handle, "set item texture seed", (float)seed);
            setAttrByName.Invoke(item.AttributeList.Handle, "set item texture wear", wear);

            // Notify engine that attributes changed (missing from earlier versions — likely
            // the root cause of T-side gloves rendering with default textures)
            Utilities.SetStateChanged(pawn, "CBaseModelEntity", "m_CBodyComponent");

            item.Initialized = true;

            // Force glove material/texture refresh via bodygroup toggle.
            // Set to 0 immediately, toggle to 1 after 0.2f delay.
            pawn.AcceptInput("SetBodygroup", value: "first_or_third_person,0");
            if (addTimer != null)
            {
                addTimer(0.2f, () =>
                {
                    if (pawn.IsValid)
                        pawn.AcceptInput("SetBodygroup", value: "first_or_third_person,1");
                });
            }
            else
            {
                Server.NextFrame(() =>
                {
                    if (pawn.IsValid)
                        pawn.AcceptInput("SetBodygroup", value: "first_or_third_person,1");
                });
            }
        }
        catch (Exception ex)
        {
            player.PrintToConsole($"[PlayerSkinMod] ApplyGloves error: {ex.Message}");
        }
    }

    public static void AssignItemId(CEconItemView item)
    {
        var id = unchecked(_nextItemId++);
        item.ItemID = id;
        item.ItemIDLow = (uint)(id & 0xFFFFFFFF);
        item.ItemIDHigh = (uint)(id >> 32);
    }
}
