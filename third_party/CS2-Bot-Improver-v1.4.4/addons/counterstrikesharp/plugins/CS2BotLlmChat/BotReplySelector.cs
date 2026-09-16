using CounterStrikeSharp.API.Modules.Utils;

namespace CS2BotLlmChat;

public interface IRandomSource
{
    double NextDouble();
}

public sealed class SystemRandomSource : IRandomSource
{
    private readonly Random _random = new();

    public double NextDouble()
    {
        return _random.NextDouble();
    }
}

public sealed class FixedRandomSource : IRandomSource
{
    private readonly Queue<double> _values;

    public FixedRandomSource(params double[] values)
    {
        _values = new Queue<double>(values.Length == 0 ? [0.0] : values);
    }

    public double NextDouble()
    {
        if (_values.Count == 0)
        {
            return 0.0;
        }

        var value = _values.Dequeue();
        _values.Enqueue(value);
        return Math.Clamp(value, 0.0, 0.999999);
    }
}

public sealed record SelectionContext(
    MessageIntent Intent,
    IReadOnlyList<ResolvedBot> ActiveBots,
    ConversationState State,
    DateTimeOffset Now,
    string PlayerKey);

public sealed record BotReplySelection(
    bool ShouldReply,
    string? PersonaId,
    ResolvedBot? Bot,
    string Reason,
    double Score,
    bool IsMentionTriggered)
{
    public static BotReplySelection Skip(string reason)
    {
        return new BotReplySelection(false, null, null, reason, 0, false);
    }
}

public sealed class BotReplySelector
{
    private readonly PersonaRegistry _personas;
    private readonly GlobalBotConfig _config;
    private readonly IRandomSource _random;

    public BotReplySelector(PersonaRegistry personas, GlobalBotConfig config, IRandomSource random)
    {
        _personas = personas;
        _config = config;
        _random = random;
    }

    public BotReplySelection Select(ChatEvent ev, SelectionContext context)
    {
        var intent = context.Intent;
        if (intent.ShouldIgnore)
        {
            return BotReplySelection.Skip(intent.IgnoreReason ?? "ignored");
        }

        if (context.ActiveBots.Count == 0)
        {
            return BotReplySelection.Skip("no active bot");
        }

        var globalBlock = CheckGlobalRateLimit(context);
        if (globalBlock is not null)
        {
            return BotReplySelection.Skip(globalBlock);
        }

        var candidates = BuildCandidates(ev, context).ToArray();
        if (candidates.Length == 0)
        {
            return BotReplySelection.Skip("no eligible bot");
        }

        var replyChance = CalculateReplyChance(context, candidates);
        if (_random.NextDouble() >= replyChance)
        {
            return BotReplySelection.Skip($"probability gate {replyChance:0.00}");
        }

        var selected = PickWeighted(candidates);
        if (selected is null)
        {
            return BotReplySelection.Skip("zero score");
        }

        return new BotReplySelection(
            true,
            selected.Persona.Id,
            selected.Bot,
            selected.Reason,
            selected.Score,
            intent.HasMention);
    }

    private string? CheckGlobalRateLimit(SelectionContext context)
    {
        var state = context.State;
        if (state.LastBotReplyAt is { } lastBotReplyAt)
        {
            var elapsed = context.Now - lastBotReplyAt;
            if (elapsed.TotalMilliseconds < _config.RateLimit.GlobalCooldownMs)
            {
                return $"global cooldown {elapsed.TotalMilliseconds:0}ms";
            }
        }

        if (_config.RateLimit.MaxRepliesPerMinute > 0 && state.RepliesLastMinute >= _config.RateLimit.MaxRepliesPerMinute)
        {
            return "global replies per minute reached";
        }

        if (_config.RateLimit.MaxRepliesPerRound > 0 && state.RepliesThisRound >= _config.RateLimit.MaxRepliesPerRound)
        {
            return "round reply budget reached";
        }

        if (!context.Intent.HasMention
            && _config.RateLimit.SuppressAfterConsecutiveBotReplies > 0
            && state.ConsecutiveBotReplies >= _config.RateLimit.SuppressAfterConsecutiveBotReplies)
        {
            return "suppressed after bot reply";
        }

        if (state.LastReplyByPlayer.TryGetValue(context.PlayerKey, out var lastPlayerReplyAt))
        {
            var elapsed = context.Now - lastPlayerReplyAt;
            if (elapsed.TotalMilliseconds < _config.RateLimit.PerPlayerCooldownMs)
            {
                return $"player cooldown {elapsed.TotalMilliseconds:0}ms";
            }
        }

        return null;
    }

    private IEnumerable<Candidate> BuildCandidates(ChatEvent ev, SelectionContext context)
    {
        foreach (var activeBot in context.ActiveBots)
        {
            var persona = _personas.GetById(activeBot.PersonaId);
            if (persona is null || !persona.Enabled)
            {
                continue;
            }

            if (context.Intent.HasMention
                && !persona.Id.Equals(context.Intent.MentionedPersonaId, StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            if (!PassesBotCooldown(persona, activeBot, context))
            {
                continue;
            }

            if (persona.Selection.MaxRepliesPerMinute > 0
                && context.State.RepliesLastMinuteByPersona.TryGetValue(persona.Id, out var count)
                && count >= persona.Selection.MaxRepliesPerMinute)
            {
                continue;
            }

            var score = ScoreCandidate(ev, context, persona, activeBot);
            if (score > 0)
            {
                yield return new Candidate(persona, activeBot, score, BuildReason(context, persona, score));
            }
        }
    }

    private bool PassesBotCooldown(BotPersonaConfig persona, ResolvedBot bot, SelectionContext context)
    {
        if (!context.State.LastReplyByBotSlot.TryGetValue(bot.Slot, out var lastReplyAt))
        {
            return true;
        }

        var configuredCooldown = persona.Selection.CooldownMs > 0
            ? persona.Selection.CooldownMs
            : _config.RateLimit.PerBotCooldownMs;

        var cooldown = Math.Max(configuredCooldown, _config.RateLimit.PerBotCooldownMs);
        if (context.Intent.HasMention && _config.RateLimit.AllowMentionBypassCooldown)
        {
            cooldown = Math.Min(cooldown, _config.RateLimit.MentionBypassMinCooldownMs);
        }

        return (context.Now - lastReplyAt).TotalMilliseconds >= cooldown;
    }

    private double CalculateReplyChance(SelectionContext context, IReadOnlyList<Candidate> candidates)
    {
        if (context.Intent.HasMention)
        {
            var personaChance = candidates.Max(candidate => candidate.Persona.Selection.MentionReplyChance);
            return Math.Clamp(Math.Max(personaChance, _config.Selection.MentionReplyChance), 0, 1);
        }

        var chance = _config.Selection.NoMentionBaseReplyChance;
        if (context.Intent.LooksLikeQuestion)
        {
            chance += _config.Selection.QuestionBoost;
        }

        if (context.State.LastBotReplyAt is { } lastBotReplyAt
            && (context.Now - lastBotReplyAt).TotalSeconds < 10)
        {
            chance -= _config.Selection.RecentlySpokenPenalty;
        }

        chance += Jitter();
        return Math.Clamp(chance, 0, 1);
    }

    private double ScoreCandidate(ChatEvent ev, SelectionContext context, BotPersonaConfig persona, ResolvedBot bot)
    {
        var score = Math.Max(persona.Selection.BaseReplyChance, 0.05) * persona.Selection.ActivityWeight;

        if (context.Intent.HasMention)
        {
            score += _config.Selection.NameMentionBoost + persona.Selection.MentionReplyChance;
        }

        if (context.Intent.LooksLikeQuestion)
        {
            score += Math.Max(_config.Selection.QuestionBoost, persona.Selection.QuestionReplyChance);
        }

        if (context.Intent.LooksLikeReplyToBot && context.State.LastBotSlot == bot.Slot)
        {
            score += 0.25;
        }

        if (ev.Channel == ChatChannel.Team && bot.Team == ev.Team && bot.Team is CsTeam.Terrorist or CsTeam.CounterTerrorist)
        {
            score += _config.Selection.TeamOnlyBias;
        }

        if (context.State.LastBotSlot == bot.Slot)
        {
            score -= _config.Selection.SameBotRepeatPenalty;
        }

        score += Jitter();
        return Math.Max(0, score);
    }

    private Candidate? PickWeighted(IReadOnlyList<Candidate> candidates)
    {
        var total = candidates.Sum(candidate => candidate.Score);
        if (total <= 0)
        {
            return null;
        }

        var threshold = _random.NextDouble() * total;
        var current = 0.0;
        foreach (var candidate in candidates)
        {
            current += candidate.Score;
            if (current >= threshold)
            {
                return candidate;
            }
        }

        return candidates[^1];
    }

    private double Jitter()
    {
        if (_config.Selection.RandomJitter <= 0)
        {
            return 0;
        }

        return ((_random.NextDouble() * 2) - 1) * _config.Selection.RandomJitter;
    }

    private static string BuildReason(SelectionContext context, BotPersonaConfig persona, double score)
    {
        var prefix = context.Intent.HasMention ? "mention" : "rules";
        return $"{prefix}:{persona.Id}:score={score:0.00}";
    }

    private sealed record Candidate(BotPersonaConfig Persona, ResolvedBot Bot, double Score, string Reason);
}
