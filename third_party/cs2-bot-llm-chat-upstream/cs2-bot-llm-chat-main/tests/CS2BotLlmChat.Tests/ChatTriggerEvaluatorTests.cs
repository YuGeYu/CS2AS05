using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class ChatTriggerEvaluatorTests
{
    [Fact]
    public void Evaluate_RequiresMentionByDefault()
    {
        var config = new BotChatConfig();
        config.Normalize();

        var trigger = ChatTriggerEvaluator.Evaluate("hello", config);

        Assert.Null(trigger);
    }

    [Fact]
    public void Evaluate_AllowsAnyMessageWhenMentionIsNotRequired()
    {
        var config = new BotChatConfig();
        config.Chat.RequireMention = false;
        config.Normalize();

        var trigger = ChatTriggerEvaluator.Evaluate("hello", config);

        Assert.NotNull(trigger);
        Assert.Equal("hello", trigger!.UserMessage);
        Assert.Null(trigger.Alias);
    }

    [Fact]
    public void Evaluate_StripsMentionWhenPresent()
    {
        var config = new BotChatConfig();
        config.Normalize();

        var trigger = ChatTriggerEvaluator.Evaluate("@s1mple hello", config);

        Assert.NotNull(trigger);
        Assert.Equal("hello", trigger!.UserMessage);
        Assert.Equal("@s1mple", trigger.Alias);
    }
}
