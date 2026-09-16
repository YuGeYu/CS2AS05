namespace CS2BotLlmChat;

public enum CooldownBlockReason
{
    None,
    Bot,
    Player
}

public readonly record struct CooldownResult(bool Allowed, CooldownBlockReason Reason, TimeSpan Remaining)
{
    public static CooldownResult Allow() => new(true, CooldownBlockReason.None, TimeSpan.Zero);

    public static CooldownResult Block(CooldownBlockReason reason, TimeSpan remaining) => new(false, reason, remaining);
}

public sealed class CooldownGate
{
    private readonly Dictionary<string, DateTimeOffset> _lastReplyByPlayer = new(StringComparer.Ordinal);
    private ChatBehaviorConfig _settings;
    private DateTimeOffset? _lastBotReplyAt;

    public CooldownGate(ChatBehaviorConfig settings)
    {
        _settings = settings;
    }

    public void ApplySettings(ChatBehaviorConfig settings)
    {
        _settings = settings;
    }

    public CooldownResult TryAcquire(string playerKey, DateTimeOffset now)
    {
        var botCooldown = TimeSpan.FromSeconds(_settings.BotCooldownSeconds);
        if (_lastBotReplyAt is { } lastBotReplyAt)
        {
            var remaining = botCooldown - (now - lastBotReplyAt);
            if (remaining > TimeSpan.Zero)
            {
                return CooldownResult.Block(CooldownBlockReason.Bot, remaining);
            }
        }

        var playerCooldown = TimeSpan.FromSeconds(_settings.PerPlayerCooldownSeconds);
        if (_lastReplyByPlayer.TryGetValue(playerKey, out var lastPlayerReplyAt))
        {
            var remaining = playerCooldown - (now - lastPlayerReplyAt);
            if (remaining > TimeSpan.Zero)
            {
                return CooldownResult.Block(CooldownBlockReason.Player, remaining);
            }
        }

        _lastBotReplyAt = now;
        _lastReplyByPlayer[playerKey] = now;
        return CooldownResult.Allow();
    }
}
