using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class ChatMentionMatcherTests
{
    [Fact]
    public void FindMention_MatchesConfiguredAliasWithBoundary()
    {
        var match = ChatMentionMatcher.FindMention("nice clutch @s1mple?", ["@s1mple"]);

        Assert.NotNull(match);
        Assert.Equal("@s1mple", match!.Alias);
        Assert.Equal("nice clutch ?", match.UserMessage);
    }

    [Fact]
    public void FindMention_DoesNotMatchAliasInsideAnotherToken()
    {
        var match = ChatMentionMatcher.FindMention("nots1mple should not trigger", ["s1mple"]);

        Assert.Null(match);
    }

    [Fact]
    public void FindMention_PrefersLongerAlias()
    {
        var match = ChatMentionMatcher.FindMention("@s1mple hello", ["s1mple", "@s1mple"]);

        Assert.NotNull(match);
        Assert.Equal("@s1mple", match!.Alias);
        Assert.Equal("hello", match.UserMessage);
    }
}
