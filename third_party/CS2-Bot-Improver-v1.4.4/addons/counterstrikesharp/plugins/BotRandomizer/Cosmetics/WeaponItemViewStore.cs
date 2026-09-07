using System.Runtime.InteropServices;
using CounterStrikeSharp.API.Core;
using CounterStrikeSharp.API.Modules.Memory;
using CounterStrikeSharp.API.Modules.Memory.DynamicFunctions;
using Microsoft.Extensions.Logging;

namespace BotRandomizer;

internal sealed class WeaponItemViewStore : IDisposable
{
    private const ulong SteamId64AccountBase = 76_561_197_960_265_728;

    private readonly MemoryFunctionWithReturn<nint, nint>? _constructor;
    private readonly MemoryFunctionWithReturn<nint, string, float, int>? _setAttributeByName;
    private readonly ILogger _logger;
    private readonly Dictionary<(int Slot, int UserId, ushort DefIndex), nint> _views = [];
    private bool _errorLogged;
    private bool _disposed;

    internal WeaponItemViewStore(
        MemoryFunctionWithReturn<nint, nint>? constructor,
        MemoryFunctionWithReturn<nint, string, float, int>? setAttributeByName,
        ILogger logger)
    {
        _constructor = constructor;
        _setAttributeByName = setAttributeByName;
        _logger = logger;
    }

    internal bool NativeAvailable => _constructor is not null && _setAttributeByName is not null;

    internal bool TryPrepare(
        SlotCosmeticState state,
        WeaponCatalogEntry weapon,
        WeaponCosmeticSelection selection,
        bool includeStickers,
        bool includeCharms,
        ulong steamId,
        out nint itemViewHandle)
    {
        itemViewHandle = nint.Zero;
        if (_disposed || _constructor is null || _setAttributeByName is null)
            return false;

        try
        {
            var key = (state.Slot, state.UserId, weapon.DefIndex);
            if (!_views.TryGetValue(key, out itemViewHandle))
            {
                var classSize = Schema.GetClassSize("CEconItemView");
                if (classSize <= 0)
                    return false;

                itemViewHandle = Marshal.AllocHGlobal(classSize);
                try
                {
                    _constructor.Invoke(itemViewHandle);
                    _views.Add(key, itemViewHandle);
                }
                catch
                {
                    Marshal.FreeHGlobal(itemViewHandle);
                    itemViewHandle = nint.Zero;
                    throw;
                }
            }

            var item = new CEconItemView(itemViewHandle);
            var attributes = item.NetworkedDynamicAttributes;
            if (attributes.Handle == nint.Zero)
                return false;

            item.Initialized = true;
            item.ItemDefinitionIndex = weapon.DefIndex;
            AssignItemId(item);
            item.AccountID = AccountIdFromSteamId(steamId);
            item.EntityQuality = 4;

            attributes.Attributes.RemoveAll();
            SetAttribute(attributes, "set item texture prefab", selection.PaintKit);
            SetAttribute(attributes, "set item texture seed", selection.Seed);
            SetAttribute(attributes, "set item texture wear", selection.Wear);

            if (includeStickers)
            {
                foreach (var sticker in selection.Stickers)
                    SetStickerAttributes(attributes, sticker);
            }

            if (includeCharms && selection.Keychain is not null)
                SetKeychainAttributes(attributes, selection.Keychain);

            return true;
        }
        catch (Exception exception)
        {
            if (!_errorLogged)
            {
                _errorLogged = true;
                _logger.LogError(exception, "[BotRandomizer] Failed to prepare a weapon item view");
            }
            itemViewHandle = nint.Zero;
            return false;
        }
    }

    internal void ClearSlot(int slot)
    {
        foreach (var key in _views.Keys.Where(key => key.Slot == slot).ToArray())
        {
            if (_views.Remove(key, out var handle))
                Marshal.FreeHGlobal(handle);
        }
    }

    internal void Clear()
    {
        foreach (var handle in _views.Values)
            Marshal.FreeHGlobal(handle);
        _views.Clear();
    }

    public void Dispose()
    {
        if (_disposed)
            return;

        _disposed = true;
        Clear();
    }

    private void SetStickerAttributes(CAttributeList attributes, StickerSelection sticker)
    {
        var slot = $"sticker slot {sticker.Slot}";
        SetAttribute(attributes, $"{slot} id", AttributeEncoding.UInt32BitsToSingle(sticker.DefIndex));
        SetAttribute(attributes, $"{slot} schema", AttributeEncoding.UInt32BitsToSingle(sticker.Schema));
        SetAttribute(attributes, $"{slot} wear", sticker.Wear);
        if (sticker.Rotation is float rotation)
            SetAttribute(attributes, $"{slot} rotation", rotation);
        if (sticker.X is float x)
            SetAttribute(attributes, $"{slot} offset x", x);
        if (sticker.Y is float y)
            SetAttribute(attributes, $"{slot} offset y", y);
    }

    private void SetKeychainAttributes(CAttributeList attributes, KeychainSelection keychain)
    {
        var slot = $"keychain slot {keychain.Slot}";
        SetAttribute(attributes, $"{slot} id", AttributeEncoding.UInt32BitsToSingle(keychain.DefIndex));
        SetAttribute(attributes, $"{slot} seed", AttributeEncoding.Int32BitsToSingle(keychain.Seed));
        if (keychain.Sticker is uint sticker)
            SetAttribute(attributes, $"{slot} sticker", AttributeEncoding.UInt32BitsToSingle(sticker));
        if (keychain.X is float x)
            SetAttribute(attributes, $"{slot} offset x", x);
        if (keychain.Y is float y)
            SetAttribute(attributes, $"{slot} offset y", y);
        if (keychain.Z is float z)
            SetAttribute(attributes, $"{slot} offset z", z);
    }

    private void SetAttribute(CAttributeList attributes, string name, float value)
        => _setAttributeByName!.Invoke(attributes.Handle, name, value);

    private static void AssignItemId(CEconItemView item)
    {
        var itemId = EconItemIdAllocator.Next();
        item.ItemID = itemId;
        item.ItemIDLow = (uint)(itemId & uint.MaxValue);
        item.ItemIDHigh = (uint)(itemId >> 32);
    }

    private static uint AccountIdFromSteamId(ulong steamId)
    {
        if (steamId >= SteamId64AccountBase)
        {
            var accountId = steamId - SteamId64AccountBase;
            return accountId <= uint.MaxValue ? (uint)accountId : 0;
        }
        return steamId <= uint.MaxValue ? (uint)steamId : 0;
    }
}
