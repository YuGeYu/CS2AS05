namespace CS2BotLlmChat;

public sealed class OwnBotOutputCache
{
    private readonly object _gate = new();
    private readonly List<Entry> _entries = [];

    public void Add(int slot, string botName, string text, DateTimeOffset now, TimeSpan ttl)
    {
        var normalized = TextKey.NormalizeLooseText(text);
        if (normalized.Length == 0)
        {
            return;
        }

        lock (_gate)
        {
            Prune(now);
            _entries.Add(new Entry(slot, botName, normalized, now + ttl));
        }
    }

    public bool Matches(int slot, string speakerName, string text, DateTimeOffset now)
    {
        var normalized = TextKey.NormalizeLooseText(text);
        if (normalized.Length == 0)
        {
            return false;
        }

        lock (_gate)
        {
            Prune(now);
            return _entries.Any(entry =>
                entry.Slot == slot
                || entry.BotName.Equals(speakerName, StringComparison.OrdinalIgnoreCase)
                || entry.NormalizedText.Equals(normalized, StringComparison.OrdinalIgnoreCase));
        }
    }

    private void Prune(DateTimeOffset now)
    {
        _entries.RemoveAll(entry => entry.ExpiresAt <= now);
    }

    private sealed record Entry(int Slot, string BotName, string NormalizedText, DateTimeOffset ExpiresAt);
}

