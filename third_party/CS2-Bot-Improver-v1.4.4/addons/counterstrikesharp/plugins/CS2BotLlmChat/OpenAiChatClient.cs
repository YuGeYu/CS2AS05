using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace CS2BotLlmChat;

public sealed class OpenAiChatClient
{
    private readonly HttpClient _httpClient;

    public OpenAiChatClient(HttpClient httpClient)
    {
        _httpClient = httpClient;
    }

    public async Task<string> CreateReplyAsync(
        BotChatConfig config,
        string playerName,
        string userMessage,
        CancellationToken cancellationToken)
    {
        var request = OpenAiChatRequestBuilder.Build(config, playerName, userMessage);
        return await CreateReplyAsync(config.Provider, request.Messages, cancellationToken).ConfigureAwait(false);
    }

    public async Task<string> CreateReplyAsync(
        LlmProviderConfig provider,
        IReadOnlyList<OpenAiChatMessage> messages,
        CancellationToken cancellationToken)
    {
        var apiKey = provider.ResolveApiKey();
        if (string.IsNullOrWhiteSpace(apiKey))
        {
            throw new InvalidOperationException($"No API key configured. Set {provider.ApiKeyEnvVar} or Provider.ApiKey.");
        }

        var models = new[] { provider.Model }.Concat(provider.FallbackModels ?? []).Distinct(StringComparer.OrdinalIgnoreCase);
        foreach (var model in models)
        {
            using var timeoutCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            timeoutCts.CancelAfter(TimeSpan.FromMilliseconds(provider.TimeoutMs));
            try
            {
                var modelProvider = provider.Clone();
                modelProvider.Model = model;
                var request = OpenAiChatRequestBuilder.Build(modelProvider, messages);
                using var httpRequest = new HttpRequestMessage(HttpMethod.Post, BuildChatCompletionsUri(provider.BaseUrl))
                {
                    Content = JsonContent.Create(request, options: JsonSerialization.Default)
                };
                httpRequest.Headers.Authorization = new AuthenticationHeaderValue("Bearer", apiKey);
                using var response = await _httpClient.SendAsync(httpRequest, HttpCompletionOption.ResponseHeadersRead, timeoutCts.Token).ConfigureAwait(false);
                var responseBody = await response.Content.ReadAsStringAsync(timeoutCts.Token).ConfigureAwait(false);
                if (!response.IsSuccessStatusCode) continue;
                var content = OpenAiChatResponseParser.ParseContent(responseBody);
                if (!string.IsNullOrWhiteSpace(content)) return content;
            }
            catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested) { break; }
            catch { /* silently try next model */ }
        }
        return string.Empty;
    }

    private static Uri BuildChatCompletionsUri(string baseUrl)
    {
        var trimmed = baseUrl.Trim().TrimEnd('/');
        if (trimmed.EndsWith("/chat/completions", StringComparison.OrdinalIgnoreCase))
        {
            return new Uri(trimmed);
        }

        return new Uri($"{trimmed}/chat/completions");
    }

    private static string TrimForLog(string value)
    {
        value = ChatReplyFormatter.Sanitize(value);
        return value.Length <= 400 ? value : value[..400];
    }
}

public static class OpenAiChatRequestBuilder
{
    public static OpenAiChatRequest Build(BotChatConfig config, string playerName, string userMessage)
    {
        var userContent = string.IsNullOrWhiteSpace(playerName)
            ? userMessage.Trim()
            : $"Player {playerName.Trim()} said: {userMessage.Trim()}\nReply directly. Do not prefix your reply with any player or bot name.";

        return Build(
            config.Provider,
            [
                new OpenAiChatMessage("system", config.Persona),
                new OpenAiChatMessage("user", userContent)
            ]);
    }

    public static OpenAiChatRequest Build(LlmProviderConfig provider, IReadOnlyList<OpenAiChatMessage> messages)
    {
        return new OpenAiChatRequest(
            provider.Model,
            messages,
            provider.MaxOutputTokens,
            provider.Temperature,
            provider.TopP,
            Stream: false);
    }
}

public static class OpenAiChatResponseParser
{
    public static string ParseContent(string json)
    {
        using var document = JsonDocument.Parse(json);
        if (!document.RootElement.TryGetProperty("choices", out var choices) || choices.GetArrayLength() == 0)
        {
            return string.Empty;
        }

        var firstChoice = choices[0];
        if (!firstChoice.TryGetProperty("message", out var message))
        {
            return string.Empty;
        }

        if (!message.TryGetProperty("content", out var content) || content.ValueKind != JsonValueKind.String)
        {
            return string.Empty;
        }

        return content.GetString() ?? string.Empty;
    }
}

public sealed record OpenAiChatRequest(
    [property: JsonPropertyName("model")] string Model,
    [property: JsonPropertyName("messages")] IReadOnlyList<OpenAiChatMessage> Messages,
    [property: JsonPropertyName("max_tokens")] int MaxTokens,
    [property: JsonPropertyName("temperature")] double Temperature,
    [property: JsonPropertyName("top_p")] double? TopP,
    [property: JsonPropertyName("stream")] bool Stream);

public sealed record OpenAiChatMessage(
    [property: JsonPropertyName("role")] string Role,
    [property: JsonPropertyName("content")] string Content);

public static class JsonSerialization
{
    public static readonly JsonSerializerOptions Default = new(JsonSerializerDefaults.Web)
    {
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };
}
