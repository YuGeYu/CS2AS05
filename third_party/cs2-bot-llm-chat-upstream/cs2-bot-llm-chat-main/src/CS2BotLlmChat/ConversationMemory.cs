namespace CS2BotLlmChat;

public sealed class ConversationMemory
{
    private readonly object _gate = new();
    private readonly Queue<ChatMemoryLine> _recent = new();
    private readonly Dictionary<string, Queue<MemoryItem>> _botMemory = new(StringComparer.OrdinalIgnoreCase);
    private readonly Dictionary<string, DateTimeOffset> _lastReplyByPlayer = new(StringComparer.Ordinal);
    private readonly Dictionary<string, DateTimeOffset> _lastReplyByPersona = new(StringComparer.OrdinalIgnoreCase);
    private readonly Dictionary<int, DateTimeOffset> _lastReplyByBotSlot = [];
    private readonly Dictionary<string, List<DateTimeOffset>> _replyTimesByPersona = new(StringComparer.OrdinalIgnoreCase);
    private readonly List<DateTimeOffset> _replyTimes = [];
    private string? _lastBotPersonaId;
    private int? _lastBotSlot;
    private DateTimeOffset? _lastBotReplyAt;
    private string? _lastHumanSpeakerName;
    private int _consecutiveBotReplies;
    private int _repliesThisRound;

    public void AddPlayerMessage(ChatEvent ev, MemoryConfig config)
    {
        if (!config.Enabled)
        {
            return;
        }

        var text = config.StoreFullPlayerMessages
            ? ev.Text
            : TruncateAtBoundary(ev.Text, 160);

        lock (_gate)
        {
            _recent.Enqueue(new ChatMemoryLine(ev.SpeakerName, IsBot: false, text, ev.Timestamp));
            TrimRecent(config);
            _lastHumanSpeakerName = ev.SpeakerName;
            _consecutiveBotReplies = 0;
        }
    }

    public void AddBotMessage(string personaId, int botSlot, string botName, string text, DateTimeOffset time, MemoryConfig config)
    {
        lock (_gate)
        {
            if (config.Enabled && config.StoreBotMessagesInRecentContext)
            {
                _recent.Enqueue(new ChatMemoryLine(botName, IsBot: true, TruncateAtBoundary(text, 160), time));
                TrimRecent(config);
            }

            _lastBotPersonaId = personaId;
            _lastBotSlot = botSlot >= 0 ? botSlot : null;
            _lastBotReplyAt = time;
            _lastReplyByPersona[personaId] = time;
            if (botSlot >= 0)
            {
                _lastReplyByBotSlot[botSlot] = time;
            }

            _replyTimes.Add(time);
            GetReplyTimes(personaId).Add(time);
            _repliesThisRound++;
            _consecutiveBotReplies++;
            PruneReplyTimes(time);
        }
    }

    public void MarkReplyForPlayer(string playerKey, string personaId, int botSlot, DateTimeOffset time)
    {
        lock (_gate)
        {
            _lastReplyByPlayer[playerKey] = time;
            _lastReplyByPersona[personaId] = time;
            if (botSlot >= 0)
            {
                _lastReplyByBotSlot[botSlot] = time;
            }
        }
    }

    public IReadOnlyList<ChatMemoryLine> GetRecentContext(int maxLines, int maxChars)
    {
        lock (_gate)
        {
            if (maxLines <= 0 || maxChars <= 0)
            {
                return [];
            }

            var selected = new List<ChatMemoryLine>();
            var usedChars = 0;
            foreach (var line in _recent.Reverse())
            {
                var chars = line.ToString().Length;
                if (selected.Count >= maxLines || usedChars + chars > maxChars)
                {
                    break;
                }

                selected.Add(line);
                usedChars += chars;
            }

            selected.Reverse();
            return selected;
        }
    }

    public IReadOnlyList<MemoryItem> GetBotMemory(string personaId, int maxItems, int maxChars)
    {
        lock (_gate)
        {
            if (!_botMemory.TryGetValue(personaId, out var items) || maxItems <= 0 || maxChars <= 0)
            {
                return [];
            }

            var selected = new List<MemoryItem>();
            var usedChars = 0;
            foreach (var item in items.Reverse())
            {
                if (selected.Count >= maxItems || usedChars + item.Text.Length > maxChars)
                {
                    break;
                }

                selected.Add(item);
                usedChars += item.Text.Length;
            }

            selected.Reverse();
            return selected;
        }
    }

    public void UpdateBotMemoryAfterReply(string personaId, ChatEvent playerMessage, string botReply, MemoryConfig config)
    {
        if (!config.Enabled || !config.PerBotShortMemoryEnabled || config.PerBotShortMemoryMaxItems <= 0)
        {
            return;
        }

        var item = new MemoryItem(
            playerMessage.Timestamp,
            TruncateAtBoundary($"玩家 {playerMessage.SpeakerName} 刚聊：{playerMessage.Text}；你回：{botReply}", 80),
            MemoryKind.BotReplied);

        lock (_gate)
        {
            var queue = GetMemoryQueue(personaId);
            queue.Enqueue(item);
            while (queue.Count > config.PerBotShortMemoryMaxItems)
            {
                queue.Dequeue();
            }
        }
    }

    public ConversationState GetState(DateTimeOffset now)
    {
        lock (_gate)
        {
            PruneReplyTimes(now);
            return new ConversationState(
                _lastBotPersonaId,
                _lastBotSlot,
                _lastBotReplyAt,
                _lastHumanSpeakerName,
                _consecutiveBotReplies,
                _repliesThisRound,
                _replyTimes.Count,
                new Dictionary<string, DateTimeOffset>(_lastReplyByPlayer, StringComparer.Ordinal),
                new Dictionary<string, DateTimeOffset>(_lastReplyByPersona, StringComparer.OrdinalIgnoreCase),
                new Dictionary<int, DateTimeOffset>(_lastReplyByBotSlot),
                _replyTimesByPersona.ToDictionary(
                    pair => pair.Key,
                    pair => pair.Value.Count,
                    StringComparer.OrdinalIgnoreCase));
        }
    }

    public void ResetRound()
    {
        lock (_gate)
        {
            _repliesThisRound = 0;
        }
    }

    private Queue<MemoryItem> GetMemoryQueue(string personaId)
    {
        if (!_botMemory.TryGetValue(personaId, out var queue))
        {
            queue = new Queue<MemoryItem>();
            _botMemory[personaId] = queue;
        }

        return queue;
    }

    private List<DateTimeOffset> GetReplyTimes(string personaId)
    {
        if (!_replyTimesByPersona.TryGetValue(personaId, out var times))
        {
            times = [];
            _replyTimesByPersona[personaId] = times;
        }

        return times;
    }

    private void TrimRecent(MemoryConfig config)
    {
        while (_recent.Count > config.RecentContextMessages)
        {
            _recent.Dequeue();
        }

        while (_recent.Sum(line => line.ToString().Length) > config.RecentContextMaxChars && _recent.Count > 0)
        {
            _recent.Dequeue();
        }
    }

    private void PruneReplyTimes(DateTimeOffset now)
    {
        var cutoff = now - TimeSpan.FromMinutes(1);
        _replyTimes.RemoveAll(time => time < cutoff);
        foreach (var pair in _replyTimesByPersona)
        {
            pair.Value.RemoveAll(time => time < cutoff);
        }
    }

    private static string TruncateAtBoundary(string value, int maxChars)
    {
        var clean = ChatReplyFormatter.Sanitize(value);
        if (clean.Length <= maxChars)
        {
            return clean;
        }

        return clean[..maxChars].Trim();
    }
}
