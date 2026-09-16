namespace CS2BotLlmChat;

public static class ChatReplyFormatter
{
    public static IReadOnlyList<string> BuildChunks(string? rawReply, ChatBehaviorConfig settings)
    {
        var clean = Sanitize(rawReply);
        if (clean.Length == 0)
        {
            return Array.Empty<string>();
        }

        if (clean.Length > settings.MaxReplyChars)
        {
            clean = clean[..settings.MaxReplyChars].Trim();
        }

        var chunks = new List<string>();
        var remaining = clean;

        while (remaining.Length > 0 && chunks.Count < settings.MaxChunks)
        {
            if (remaining.Length <= settings.MaxChunkChars)
            {
                chunks.Add(remaining);
                break;
            }

            var splitAt = FindSplitIndex(remaining, settings.MaxChunkChars);
            var chunk = remaining[..splitAt].Trim();
            if (chunk.Length > 0)
            {
                chunks.Add(chunk);
            }

            remaining = remaining[splitAt..].TrimStart();
        }

        return chunks;
    }

    public static string Sanitize(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return string.Empty;
        }

        var chars = new List<char>(value.Length);
        var lastWasSpace = false;

        foreach (var current in value)
        {
            if (char.IsControl(current) || char.IsWhiteSpace(current))
            {
                if (!lastWasSpace)
                {
                    chars.Add(' ');
                    lastWasSpace = true;
                }

                continue;
            }

            chars.Add(current);
            lastWasSpace = false;
        }

        return new string(chars.ToArray()).Trim();
    }

    private static int FindSplitIndex(string value, int maxChars)
    {
        var limit = Math.Min(maxChars, value.Length);
        for (var index = limit; index > limit / 2; index--)
        {
            var candidate = value[index - 1];
            if (char.IsWhiteSpace(candidate) || char.IsPunctuation(candidate))
            {
                return index;
            }
        }

        return limit;
    }
}
