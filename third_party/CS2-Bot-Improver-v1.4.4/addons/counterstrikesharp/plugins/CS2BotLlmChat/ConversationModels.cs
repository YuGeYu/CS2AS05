using System.Globalization;
using System.Text;
using CounterStrikeSharp.API.Modules.Utils;

namespace CS2BotLlmChat;

public enum ChatChannel
{
    All,
    Team
}

public sealed record ChatEvent(
    int SpeakerSlot,
    string SpeakerName,
    bool IsHuman,
    bool IsKnownBot,
    string Text,
    DateTimeOffset Timestamp,
    ChatChannel Channel,
    CsTeam Team)
{
    public string SpeakerKey => SpeakerSlot >= 0
        ? SpeakerSlot.ToString(CultureInfo.InvariantCulture)
        : SpeakerName;
}

public sealed record PlayerSnapshot(
    int Slot,
    string Name,
    bool IsBot,
    bool IsValid,
    bool IsBotHiderManaged,
    string? BotHiderName,
    CsTeam Team);

public sealed record ResolvedBot(
    string PersonaId,
    string DisplayName,
    int Slot,
    bool IsBotHiderManaged,
    string? CurrentGameName,
    CsTeam Team);

public sealed record PersonaMentionMatch(
    BotPersonaConfig Persona,
    string Alias,
    string UserMessage);

public sealed record MessageIntent(
    bool ShouldIgnore,
    string? IgnoreReason,
    bool HasMention,
    string? MentionedPersonaId,
    bool LooksLikeQuestion,
    bool LooksLikeReplyToBot,
    bool ContainsCs2Context,
    string CleanText);

public sealed record ChatMemoryLine(string SpeakerName, bool IsBot, string Text, DateTimeOffset Time)
{
    public override string ToString()
    {
        var label = IsBot ? SpeakerName : $"玩家 {SpeakerName}";
        return $"[{label}] {Text}";
    }
}

public enum MemoryKind
{
    PlayerAsked,
    BotReplied,
    AddressedBot
}

public sealed record MemoryItem(DateTimeOffset Time, string Text, MemoryKind Kind);

public sealed record ConversationState(
    string? LastBotPersonaId,
    int? LastBotSlot,
    DateTimeOffset? LastBotReplyAt,
    string? LastHumanSpeakerName,
    int ConsecutiveBotReplies,
    int RepliesThisRound,
    int RepliesLastMinute,
    IReadOnlyDictionary<string, DateTimeOffset> LastReplyByPlayer,
    IReadOnlyDictionary<string, DateTimeOffset> LastReplyByPersona,
    IReadOnlyDictionary<int, DateTimeOffset> LastReplyByBotSlot,
    IReadOnlyDictionary<string, int> RepliesLastMinuteByPersona);

public static class TextKey
{
    public static string Normalize(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return string.Empty;
        }

        var builder = new StringBuilder(value.Length);
        foreach (var current in value.Trim().ToLowerInvariant())
        {
            if (char.IsWhiteSpace(current) || current == '@')
            {
                continue;
            }

            builder.Append(current);
        }

        return builder.ToString();
    }

    public static string NormalizeAlias(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return string.Empty;
        }

        var trimmed = value.Trim();
        if (trimmed.StartsWith('@'))
        {
            trimmed = trimmed[1..];
        }

        return Normalize(trimmed);
    }

    public static string NormalizeLooseText(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return string.Empty;
        }

        var builder = new StringBuilder(value.Length);
        foreach (var current in value.Trim().ToLowerInvariant())
        {
            if (char.IsLetterOrDigit(current))
            {
                builder.Append(current);
            }
        }

        return builder.ToString();
    }
}
