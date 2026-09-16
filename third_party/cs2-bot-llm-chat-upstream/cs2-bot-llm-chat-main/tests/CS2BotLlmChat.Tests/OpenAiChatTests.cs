using System.Text.Json;
using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class OpenAiChatTests
{
    [Fact]
    public void Build_CreatesOpenAiCompatibleChatCompletionRequest()
    {
        var config = new BotChatConfig
        {
            Persona = "You are a pro player.",
            Provider = new LlmProviderConfig
            {
                Model = "test-model",
                MaxTokens = 64,
                Temperature = 0.7
            }
        };
        config.Normalize();

        var request = OpenAiChatRequestBuilder.Build(config, "PlayerOne", "why rotate?");
        var json = JsonSerializer.Serialize(request, JsonSerialization.Default);

        Assert.Contains("\"model\":\"test-model\"", json);
        Assert.Contains("\"role\":\"system\"", json);
        Assert.Contains("Player PlayerOne said: why rotate?", json);
        Assert.Contains("Reply directly. Do not prefix your reply", json);
        Assert.Contains("\"max_tokens\":64", json);
        Assert.Contains("\"stream\":false", json);
    }

    [Fact]
    public void ParseContent_ReadsFirstChoiceMessageContent()
    {
        const string json = """
        {
          "choices": [
            {
              "message": {
                "role": "assistant",
                "content": "short mid call"
              }
            }
          ]
        }
        """;

        var content = OpenAiChatResponseParser.ParseContent(json);

        Assert.Equal("short mid call", content);
    }

    [Fact]
    public void ParseContent_ReturnsEmptyStringForNullContent()
    {
        const string json = """
        {
          "choices": [
            {
              "message": {
                "role": "assistant",
                "content": null
              }
            }
          ]
        }
        """;

        var content = OpenAiChatResponseParser.ParseContent(json);

        Assert.Equal(string.Empty, content);
    }
}
