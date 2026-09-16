using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class CooldownGateTests
{
    [Fact]
    public void TryAcquire_BlocksOnGlobalBotCooldown()
    {
        var settings = new ChatBehaviorConfig
        {
            BotCooldownSeconds = 10,
            PerPlayerCooldownSeconds = 30
        };
        settings.Normalize();
        var gate = new CooldownGate(settings);
        var now = DateTimeOffset.Parse("2026-06-02T00:00:00Z");

        var first = gate.TryAcquire("player-1", now);
        var second = gate.TryAcquire("player-2", now.AddSeconds(3));

        Assert.True(first.Allowed);
        Assert.False(second.Allowed);
        Assert.Equal(CooldownBlockReason.Bot, second.Reason);
    }

    [Fact]
    public void TryAcquire_BlocksSamePlayerAfterBotCooldown()
    {
        var settings = new ChatBehaviorConfig
        {
            BotCooldownSeconds = 2,
            PerPlayerCooldownSeconds = 30
        };
        settings.Normalize();
        var gate = new CooldownGate(settings);
        var now = DateTimeOffset.Parse("2026-06-02T00:00:00Z");

        gate.TryAcquire("player-1", now);
        var second = gate.TryAcquire("player-1", now.AddSeconds(3));

        Assert.False(second.Allowed);
        Assert.Equal(CooldownBlockReason.Player, second.Reason);
    }

    [Fact]
    public void TryAcquire_AllowsAfterCooldownsExpire()
    {
        var settings = new ChatBehaviorConfig
        {
            BotCooldownSeconds = 2,
            PerPlayerCooldownSeconds = 4
        };
        settings.Normalize();
        var gate = new CooldownGate(settings);
        var now = DateTimeOffset.Parse("2026-06-02T00:00:00Z");

        gate.TryAcquire("player-1", now);
        var second = gate.TryAcquire("player-1", now.AddSeconds(5));

        Assert.True(second.Allowed);
    }
}
