namespace CS2BotLlmChat;

public sealed record MentionMatch(string Alias, string UserMessage);

public static class ChatMentionMatcher
{
    public static MentionMatch? FindMention(string message, IEnumerable<string> aliases)
    {
        if (string.IsNullOrWhiteSpace(message))
        {
            return null;
        }

        foreach (var alias in aliases.Where(a => !string.IsNullOrWhiteSpace(a)).OrderByDescending(a => a.Length))
        {
            var trimmedAlias = alias.Trim();
            if (!TryFindAlias(message, trimmedAlias, out var index))
            {
                continue;
            }

            var userMessage = RemoveAlias(message, index, trimmedAlias.Length);
            return new MentionMatch(trimmedAlias, userMessage);
        }

        return null;
    }

    private static bool TryFindAlias(string message, string alias, out int index)
    {
        var start = 0;
        while (start < message.Length)
        {
            index = message.IndexOf(alias, start, StringComparison.OrdinalIgnoreCase);
            if (index < 0)
            {
                return false;
            }

            if (HasBoundaryBefore(message, index) && HasBoundaryAfter(message, index + alias.Length))
            {
                return true;
            }

            start = index + 1;
        }

        index = -1;
        return false;
    }

    private static string RemoveAlias(string message, int index, int length)
    {
        var withoutAlias = string.Concat(message.AsSpan(0, index), message.AsSpan(index + length)).Trim();
        return string.IsNullOrWhiteSpace(withoutAlias) ? message.Trim() : withoutAlias;
    }

    private static bool HasBoundaryBefore(string value, int index)
    {
        return index == 0 || IsBoundary(value[index - 1]);
    }

    private static bool HasBoundaryAfter(string value, int index)
    {
        return index >= value.Length || IsBoundary(value[index]);
    }

    private static bool IsBoundary(char value)
    {
        return !char.IsLetterOrDigit(value) && value != '_';
    }
}
