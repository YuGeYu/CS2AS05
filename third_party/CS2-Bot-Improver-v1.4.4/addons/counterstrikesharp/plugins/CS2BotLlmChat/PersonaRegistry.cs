namespace CS2BotLlmChat;

public sealed class PersonaRegistry
{
    private readonly IReadOnlyList<BotPersonaConfig> _personas;
    private readonly Dictionary<string, BotPersonaConfig> _byId;
    private readonly IReadOnlyList<(string Alias, BotPersonaConfig Persona)> _mentionAliases;

    public PersonaRegistry(IEnumerable<BotPersonaConfig> personas)
    {
        _personas = personas
            .Where(persona => persona.Enabled)
            .ToArray();

        _byId = _personas
            .GroupBy(persona => persona.Id, StringComparer.OrdinalIgnoreCase)
            .ToDictionary(group => group.Key, group => group.First(), StringComparer.OrdinalIgnoreCase);

        _mentionAliases = _personas
            .SelectMany(persona => GetMentionAliases(persona).Select(alias => (Alias: alias, Persona: persona)))
            .Where(candidate => !string.IsNullOrWhiteSpace(candidate.Alias))
            .OrderByDescending(candidate => candidate.Alias.Length)
            .ToArray();
    }

    public IReadOnlyList<BotPersonaConfig> GetEnabledPersonas()
    {
        return _personas;
    }

    public BotPersonaConfig? GetById(string? personaId)
    {
        return string.IsNullOrWhiteSpace(personaId)
            ? null
            : _byId.GetValueOrDefault(personaId);
    }

    public PersonaMentionMatch? MatchMention(string text)
    {
        if (string.IsNullOrWhiteSpace(text))
        {
            return null;
        }

        foreach (var (alias, persona) in _mentionAliases)
        {
            var mention = ChatMentionMatcher.FindMention(text, [alias]);
            if (mention is not null)
            {
                return new PersonaMentionMatch(persona, mention.Alias, mention.UserMessage);
            }
        }

        return null;
    }

    public IReadOnlyList<PersonaMentionMatch> MatchAllMentions(string text)
    {
        var matches = new List<PersonaMentionMatch>();
        foreach (var persona in _personas)
        {
            var aliases = GetMentionAliases(persona).OrderByDescending(alias => alias.Length);
            var mention = ChatMentionMatcher.FindMention(text, aliases);
            if (mention is not null)
            {
                matches.Add(new PersonaMentionMatch(persona, mention.Alias, mention.UserMessage));
            }
        }

        return matches;
    }

    public static IReadOnlyList<string> GetMentionAliases(BotPersonaConfig persona)
    {
        var aliases = new List<string>();
        AddAlias(aliases, persona.DisplayName);

        foreach (var alias in persona.Aliases)
        {
            AddAlias(aliases, alias);
        }

        if (!string.IsNullOrWhiteSpace(persona.Id))
        {
            AddAlias(aliases, persona.Id);
        }

        return aliases
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .OrderByDescending(alias => alias.Length)
            .ToArray();
    }

    private static void AddAlias(ICollection<string> aliases, string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return;
        }

        var trimmed = value.Trim();
        aliases.Add(trimmed);

        if (!trimmed.StartsWith('@') && !trimmed.Contains(' ', StringComparison.Ordinal))
        {
            aliases.Add($"@{trimmed}");
        }
    }
}

