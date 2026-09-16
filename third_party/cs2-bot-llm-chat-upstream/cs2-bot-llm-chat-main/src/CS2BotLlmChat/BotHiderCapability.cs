using System.Reflection;
using CounterStrikeSharp.API.Core;
using CounterStrikeSharp.API.Core.Capabilities;
using Microsoft.Extensions.Logging;

namespace CS2BotLlmChat;

public sealed class BotHiderCapability
{
    private readonly object _api;
    private readonly MethodInfo _getManagedSlots;
    private readonly MethodInfo _getPersonaName;
    private readonly MethodInfo _isManagedBot;

    private BotHiderCapability(object api, Type apiType)
    {
        _api = api;
        _isManagedBot = RequireMethod(apiType, "IsManagedBot");
        _getManagedSlots = RequireMethod(apiType, "GetManagedSlots");
        _getPersonaName = RequireMethod(apiType, "GetPersonaName");
    }

    public static BotHiderCapability? TryResolve(ILogger logger)
    {
        try
        {
            var apiType = ResolveApiType();
            if (apiType is null)
            {
                logger.LogInformation("BotHiderApi is not loaded.");
                return null;
            }

            var capabilityType = typeof(PluginCapability<>).MakeGenericType(apiType);
            var capability = Activator.CreateInstance(capabilityType, "bothider:api");
            var api = capabilityType.GetMethod("Get")?.Invoke(capability, null);
            if (api is null)
            {
                logger.LogInformation("BotHiderApi assembly found, but bothider:api capability is not available.");
                return null;
            }

            logger.LogInformation("BotHider capability resolved.");
            return new BotHiderCapability(api, apiType);
        }
        catch (Exception ex)
        {
            logger.LogWarning(ex, "BotHider capability resolution failed.");
            return null;
        }
    }

    public bool IsManagedBot(CCSPlayerController player)
    {
        return player.IsValid && IsManagedBot(player.Slot);
    }

    public bool IsManagedBot(int slot)
    {
        return _isManagedBot.Invoke(_api, [slot]) is true;
    }

    public int[] GetManagedSlots()
    {
        return _getManagedSlots.Invoke(_api, null) as int[] ?? [];
    }

    public string GetPersonaName(int slot)
    {
        return _getPersonaName.Invoke(_api, [slot]) as string ?? string.Empty;
    }

    private static Type? ResolveApiType()
    {
        var loadedType = AppDomain.CurrentDomain
            .GetAssemblies()
            .Select(assembly => assembly.GetType("BotHiderApi.IBotHiderApi", throwOnError: false))
            .FirstOrDefault(type => type is not null);

        if (loadedType is not null)
        {
            return loadedType;
        }

        try
        {
            return Assembly.Load("BotHiderApi").GetType("BotHiderApi.IBotHiderApi", throwOnError: false);
        }
        catch
        {
            return null;
        }
    }

    private static MethodInfo RequireMethod(Type type, string name)
    {
        return type.GetMethod(name) ?? throw new MissingMethodException(type.FullName, name);
    }
}
