namespace CS2BotLlmChat;

public sealed class MessageIntentParser
{
    private static readonly string[] QuestionMarkers =
    [
        "?",
        "？",
        "吗",
        "呢",
        "谁",
        "什么",
        "怎么",
        "咋",
        "为何",
        "为什么",
        "怎么看",
        "why",
        "what",
        "how"
    ];

    private static readonly string[] Cs2Markers =
    [
        "cs",
        "cs2",
        "eco",
        "rush",
        "残局",
        "经济",
        "保枪",
        "起枪",
        "道具",
        "闪",
        "烟",
        "雷",
        "包点",
        "天梯"
    ];

    public MessageIntent Parse(string message, TriggerConfig config, PersonaRegistry personas, ConversationState state)
    {
        var clean = ChatReplyFormatter.Sanitize(message);
        if (clean.Length == 0)
        {
            return Ignore("empty");
        }

        if (clean.Length > config.MaxInputChars)
        {
            clean = clean[..config.MaxInputChars].Trim();
        }

        if (config.IgnoreCommandsStartingWith.Any(prefix => clean.StartsWith(prefix, StringComparison.Ordinal)))
        {
            return Ignore("command");
        }

        var mention = personas.MatchMention(clean);
        var textWithoutMention = mention?.UserMessage ?? clean;
        var hasMention = mention is not null;

        if (!hasMention && config.RequireMention)
        {
            return Ignore("mention required");
        }

        if (!hasMention && config.IgnoreEmptyOrTooShort && TextKey.NormalizeLooseText(textWithoutMention).Length < config.MinMessageLength)
        {
            return Ignore("too short");
        }

        var looksLikeQuestion = ContainsAny(textWithoutMention, QuestionMarkers);
        var containsCs2Context = ContainsAny(textWithoutMention, Cs2Markers);
        var looksLikeReplyToBot = hasMention
            || (!string.IsNullOrWhiteSpace(state.LastBotPersonaId)
                && (textWithoutMention.Contains("你", StringComparison.OrdinalIgnoreCase)
                    || textWithoutMention.Contains("刚才", StringComparison.OrdinalIgnoreCase)
                    || textWithoutMention.Contains("怎么看", StringComparison.OrdinalIgnoreCase)));

        return new MessageIntent(
            ShouldIgnore: false,
            IgnoreReason: null,
            HasMention: hasMention,
            MentionedPersonaId: mention?.Persona.Id,
            LooksLikeQuestion: looksLikeQuestion,
            LooksLikeReplyToBot: looksLikeReplyToBot,
            ContainsCs2Context: containsCs2Context,
            CleanText: textWithoutMention);
    }

    private static MessageIntent Ignore(string reason)
    {
        return new MessageIntent(
            ShouldIgnore: true,
            IgnoreReason: reason,
            HasMention: false,
            MentionedPersonaId: null,
            LooksLikeQuestion: false,
            LooksLikeReplyToBot: false,
            ContainsCs2Context: false,
            CleanText: string.Empty);
    }

    private static bool ContainsAny(string value, IEnumerable<string> markers)
    {
        return markers.Any(marker => value.Contains(marker, StringComparison.OrdinalIgnoreCase));
    }
}

