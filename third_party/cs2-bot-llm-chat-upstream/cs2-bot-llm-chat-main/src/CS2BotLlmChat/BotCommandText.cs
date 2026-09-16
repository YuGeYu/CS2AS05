using System.Text;

namespace CS2BotLlmChat;

public static class BotCommandText
{
    private static readonly HashSet<char> ConsoleSeparators = [';', '`', '\u001b'];

    public static string SanitizeForClientCommand(string? value, int maxChars)
    {
        var clean = ChatReplyFormatter.Sanitize(value);
        if (clean.Length == 0)
        {
            return string.Empty;
        }

        maxChars = Math.Clamp(maxChars, 1, 800);

        var builder = new StringBuilder(clean.Length);
        var lastWasSpace = false;
        foreach (var current in clean)
        {
            if (ConsoleSeparators.Contains(current) || char.IsControl(current))
            {
                AppendSpace(builder, ref lastWasSpace);
                continue;
            }

            builder.Append(current);
            lastWasSpace = false;
        }

        var sanitized = ChatReplyFormatter.Sanitize(builder.ToString());
        return sanitized.Length <= maxChars ? sanitized : sanitized[..maxChars].Trim();
    }

    private static void AppendSpace(StringBuilder builder, ref bool lastWasSpace)
    {
        if (builder.Length == 0 || lastWasSpace)
        {
            return;
        }

        builder.Append(' ');
        lastWasSpace = true;
    }
}
