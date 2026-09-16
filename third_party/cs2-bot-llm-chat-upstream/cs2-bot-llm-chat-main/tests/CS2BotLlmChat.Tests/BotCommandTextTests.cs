using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class BotCommandTextTests
{
    [Fact]
    public void SanitizeForClientCommand_RemovesConsoleSeparatorsAndCapsLength()
    {
        var sanitized = BotCommandText.SanitizeForClientCommand("nice; quit `bad`\u001b next", 13);

        Assert.DoesNotContain(';', sanitized);
        Assert.DoesNotContain('`', sanitized);
        Assert.DoesNotContain('\u001b', sanitized);
        Assert.True(sanitized.Length <= 13);
        Assert.Equal("nice quit bad", sanitized);
    }
}
