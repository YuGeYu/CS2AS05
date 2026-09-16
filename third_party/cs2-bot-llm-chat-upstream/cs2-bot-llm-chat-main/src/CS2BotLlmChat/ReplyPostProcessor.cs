namespace CS2BotLlmChat;

public sealed record ProcessedReply(bool ShouldSend, string Text, string Reason)
{
    public static ProcessedReply Send(string text) => new(true, text, "ok");

    public static ProcessedReply Drop(string reason) => new(false, string.Empty, reason);
}

public sealed class ReplyPostProcessor
{
    private static readonly char[] SentenceEndings = ['。', '！', '？', '.', '!', '?', '，', ','];

    public ProcessedReply Process(string? rawReply, BotPersonaConfig persona, OutputConfig config)
    {
        var clean = ChatReplyFormatter.Sanitize(rawReply);
        if (clean.Length == 0)
        {
            return config.FallbackOnEmpty
                ? Process($"{persona.DisplayName} 没想好怎么接。", persona, config, withFallbackDisabled: true)
                : ProcessedReply.Drop("empty");
        }

        clean = StripMarkdownFence(clean);

        if (config.StripQuotes)
        {
            clean = StripWrappingQuotes(clean);
        }

        foreach (var prefix in config.ForbiddenPrefixes)
        {
            if (clean.StartsWith(prefix, StringComparison.OrdinalIgnoreCase))
            {
                clean = clean[prefix.Length..].TrimStart('：', ':', '，', ',', ' ', '\t');
            }
        }

        if (LooksLikeJson(clean))
        {
            return ProcessedReply.Drop("json-looking reply");
        }

        if (clean.Length > config.MaxReplyChars)
        {
            clean = TruncateAtSentenceBoundary(clean, config.MaxReplyChars);
        }

        return clean.Length == 0
            ? ProcessedReply.Drop("empty after processing")
            : ProcessedReply.Send(clean);
    }

    private ProcessedReply Process(string rawReply, BotPersonaConfig persona, OutputConfig config, bool withFallbackDisabled)
    {
        _ = withFallbackDisabled;
        var copy = new OutputConfig
        {
            PreferBotSay = config.PreferBotSay,
            MaxReplyChars = config.MaxReplyChars,
            StripNewlines = config.StripNewlines,
            StripQuotes = config.StripQuotes,
            ForbiddenPrefixes = config.ForbiddenPrefixes,
            FallbackOnEmpty = false
        };

        return Process(rawReply, persona, copy);
    }

    private static string StripMarkdownFence(string value)
    {
        var clean = value.Trim();
        if (clean.StartsWith("```", StringComparison.Ordinal))
        {
            clean = clean.Trim('`').Trim();
        }

        return clean;
    }

    private static string StripWrappingQuotes(string value)
    {
        var clean = value.Trim();
        var pairs = new[]
        {
            ('"', '"'),
            ('\'', '\''),
            ('“', '”'),
            ('「', '」'),
            ('『', '』')
        };

        foreach (var (left, right) in pairs)
        {
            if (clean.Length >= 2 && clean[0] == left && clean[^1] == right)
            {
                return clean[1..^1].Trim();
            }
        }

        return clean;
    }

    private static bool LooksLikeJson(string value)
    {
        return (value.StartsWith('{') && value.EndsWith('}'))
            || (value.StartsWith('[') && value.EndsWith(']'));
    }

    private static string TruncateAtSentenceBoundary(string value, int maxChars)
    {
        var capped = value[..maxChars].Trim();
        var split = capped.LastIndexOfAny(SentenceEndings);
        if (split > maxChars / 2)
        {
            return capped[..(split + 1)].Trim();
        }

        return capped;
    }
}
