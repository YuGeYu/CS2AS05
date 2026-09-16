namespace CS2BotLlmChat;

public sealed record ChatTriggerResult(string UserMessage, string? Alias);

public static class ChatTriggerEvaluator
{
    public static ChatTriggerResult? Evaluate(string message, BotChatConfig config)
    {
        if (string.IsNullOrWhiteSpace(message))
        {
            return null;
        }

        var mention = ChatMentionMatcher.FindMention(message, config.Aliases);
        if (config.Chat.RequireMention && mention is null)
        {
            return null;
        }

        return new ChatTriggerResult(mention?.UserMessage ?? message.Trim(), mention?.Alias);
    }
}
