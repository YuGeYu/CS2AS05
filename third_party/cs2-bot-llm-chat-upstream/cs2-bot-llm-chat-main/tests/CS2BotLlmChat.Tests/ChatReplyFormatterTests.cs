using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class ChatReplyFormatterTests
{
    [Fact]
    public void BuildChunks_SanitizesNewlinesAndControlCharacters()
    {
        var chunks = ChatReplyFormatter.BuildChunks("first\n\nsecond\tthird\u0001", new ChatBehaviorConfig());

        Assert.Single(chunks);
        Assert.Equal("first second third", chunks[0]);
    }

    [Fact]
    public void BuildChunks_CapsReplyAndChunkLengths()
    {
        var settings = new ChatBehaviorConfig
        {
            MaxReplyChars = 48,
            MaxChunkChars = 20,
            MaxChunks = 3
        };
        settings.Normalize();

        var chunks = ChatReplyFormatter.BuildChunks("one two three four five six seven", settings);

        Assert.True(chunks.Count <= 3);
        Assert.All(chunks, chunk => Assert.True(chunk.Length <= 20));
        Assert.Equal("one two three four", chunks[0]);
    }

    [Fact]
    public void BuildChunks_ReturnsNoChunksForEmptyReply()
    {
        var chunks = ChatReplyFormatter.BuildChunks(" \n\t ", new ChatBehaviorConfig());

        Assert.Empty(chunks);
    }
}
