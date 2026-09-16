using CounterStrikeSharp.API.Modules.Utils;

namespace CS2BotLlmChat;

public sealed class BotIdentityResolver
{
    private readonly Dictionary<string, ResolvedBot> _bindings = new(StringComparer.OrdinalIgnoreCase);
    private readonly HashSet<int> _knownBotSlots = [];
    private readonly HashSet<string> _knownBotNames = new(StringComparer.OrdinalIgnoreCase);

    public void RefreshBindings(
        PersonaRegistry personas,
        BotHiderIntegrationConfig botHiderConfig,
        IReadOnlyList<PlayerSnapshot> players)
    {
        _bindings.Clear();
        _knownBotSlots.Clear();
        _knownBotNames.Clear();

        var validPlayers = players.Where(player => player.IsValid).ToArray();
        foreach (var bot in validPlayers.Where(IsKnownBotSnapshot))
        {
            _knownBotSlots.Add(bot.Slot);
            if (!string.IsNullOrWhiteSpace(bot.Name))
            {
                _knownBotNames.Add(bot.Name);
            }

            if (!string.IsNullOrWhiteSpace(bot.BotHiderName))
            {
                _knownBotNames.Add(bot.BotHiderName);
            }
        }

        var usedSlots = new HashSet<int>();
        foreach (var persona in personas.GetEnabledPersonas())
        {
            var resolved = ResolvePersona(persona, botHiderConfig, validPlayers, usedSlots);
            if (resolved is null)
            {
                continue;
            }

            _bindings[persona.Id] = resolved;
            usedSlots.Add(resolved.Slot);
            _knownBotSlots.Add(resolved.Slot);
            _knownBotNames.Add(resolved.DisplayName);
            if (!string.IsNullOrWhiteSpace(resolved.CurrentGameName))
            {
                _knownBotNames.Add(resolved.CurrentGameName);
            }
        }
    }

    public ResolvedBot? ResolveByPersonaId(string personaId)
    {
        return _bindings.GetValueOrDefault(personaId);
    }

    public IReadOnlyList<ResolvedBot> GetActivePersonaBots()
    {
        return _bindings.Values.ToArray();
    }

    public bool IsKnownBotSlot(int slot)
    {
        return _knownBotSlots.Contains(slot);
    }

    public bool IsKnownBotName(string name)
    {
        return !string.IsNullOrWhiteSpace(name) && _knownBotNames.Contains(name.Trim());
    }

    private static ResolvedBot? ResolvePersona(
        BotPersonaConfig persona,
        BotHiderIntegrationConfig botHiderConfig,
        IReadOnlyList<PlayerSnapshot> players,
        ISet<int> usedSlots)
    {
        var configuredSlot = botHiderConfig.ManagedSlots.FirstOrDefault(slot =>
            slot.PersonaId.Equals(persona.Id, StringComparison.OrdinalIgnoreCase));

        var slot = persona.Binding.Slot ?? configuredSlot?.Slot;
        if (slot is { } exactSlot)
        {
            var exact = players.FirstOrDefault(player => player.Slot == exactSlot && !usedSlots.Contains(player.Slot));
            if (exact is not null)
            {
                return ToResolvedBot(persona, exact);
            }
        }

        var byBotHiderName = ResolveByConfiguredBotHiderName(persona, configuredSlot, players, usedSlots);
        if (byBotHiderName is not null)
        {
            return byBotHiderName;
        }

        var byName = ResolveByName(persona, players, usedSlots);
        if (byName is not null)
        {
            return byName;
        }

        if (persona.Binding.AllowBotHiderManagedSlot && botHiderConfig.Enabled)
        {
            var managed = players.FirstOrDefault(player =>
                player.IsBotHiderManaged
                && !usedSlots.Contains(player.Slot)
                && IsTeamBot(player));
            if (managed is not null)
            {
                return ToResolvedBot(persona, managed);
            }
        }

        var nativeBot = players.FirstOrDefault(player =>
            player.IsBot
            && !usedSlots.Contains(player.Slot)
            && IsTeamBot(player));

        return nativeBot is null ? null : ToResolvedBot(persona, nativeBot);
    }

    private static ResolvedBot? ResolveByConfiguredBotHiderName(
        BotPersonaConfig persona,
        BotHiderManagedSlotConfig? configuredSlot,
        IEnumerable<PlayerSnapshot> players,
        ISet<int> usedSlots)
    {
        if (configuredSlot is null || string.IsNullOrWhiteSpace(configuredSlot.Name))
        {
            return null;
        }

        var player = players.FirstOrDefault(candidate =>
            !usedSlots.Contains(candidate.Slot)
            && candidate.IsBotHiderManaged
            && MatchesAnyName(candidate, [configuredSlot.Name, .. configuredSlot.Aliases]));

        return player is null ? null : ToResolvedBot(persona, player);
    }

    private static ResolvedBot? ResolveByName(
        BotPersonaConfig persona,
        IEnumerable<PlayerSnapshot> players,
        ISet<int> usedSlots)
    {
        var names = PersonaRegistry.GetMentionAliases(persona)
            .Concat(persona.Binding.NameContains)
            .Concat(persona.Binding.FallbackNameContains)
            .Where(name => !string.IsNullOrWhiteSpace(name))
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();

        var exact = players.FirstOrDefault(candidate =>
            !usedSlots.Contains(candidate.Slot)
            && IsKnownBotSnapshot(candidate)
            && names.Any(name => candidate.Name.Equals(name.TrimStart('@'), StringComparison.OrdinalIgnoreCase)
                || (candidate.BotHiderName?.Equals(name.TrimStart('@'), StringComparison.OrdinalIgnoreCase) ?? false)));

        if (exact is not null)
        {
            return ToResolvedBot(persona, exact);
        }

        var contains = players.FirstOrDefault(candidate =>
            !usedSlots.Contains(candidate.Slot)
            && IsKnownBotSnapshot(candidate)
            && MatchesAnyName(candidate, names));

        return contains is null ? null : ToResolvedBot(persona, contains);
    }

    private static bool MatchesAnyName(PlayerSnapshot candidate, IEnumerable<string> names)
    {
        var candidateNames = new[]
        {
            candidate.Name,
            candidate.BotHiderName ?? string.Empty
        };

        return names.Any(name =>
        {
            var trimmed = name.Trim().TrimStart('@');
            return trimmed.Length > 0
                && candidateNames.Any(candidateName =>
                    candidateName.Contains(trimmed, StringComparison.OrdinalIgnoreCase));
        });
    }

    private static ResolvedBot ToResolvedBot(BotPersonaConfig persona, PlayerSnapshot player)
    {
        return new ResolvedBot(
            persona.Id,
            persona.DisplayName,
            player.Slot,
            player.IsBotHiderManaged,
            string.IsNullOrWhiteSpace(player.Name) ? player.BotHiderName : player.Name,
            player.Team);
    }

    private static bool IsKnownBotSnapshot(PlayerSnapshot player)
    {
        return player.IsBot || player.IsBotHiderManaged;
    }

    private static bool IsTeamBot(PlayerSnapshot player)
    {
        return player.Team is CsTeam.Terrorist or CsTeam.CounterTerrorist;
    }
}

