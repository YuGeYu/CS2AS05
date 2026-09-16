using System.Text.Json;
using System.Text.Json.Serialization;
using CounterStrikeSharp.API.Core;
using static CS2BotLlmChat.ConfigHelpers;

namespace CS2BotLlmChat;

public sealed class BotChatConfig : BasePluginConfig
{
    [JsonPropertyName("version")]
    public int SchemaVersion { get; set; } = 2;

    [JsonPropertyName("global")]
    public GlobalBotConfig Global { get; set; } = new();

    [JsonPropertyName("botHider")]
    public BotHiderIntegrationConfig BotHider { get; set; } = new();

    [JsonPropertyName("personas")]
    public List<BotPersonaConfig> Personas { get; set; } = [];

    [JsonPropertyName("Enabled")]
    public bool Enabled { get; set; } = true;

    [JsonPropertyName("BotName")]
    public string BotName { get; set; } = "s1mple";

    [JsonPropertyName("Aliases")]
    public string[] Aliases { get; set; } = ["@s1mple", "s1mple"];

    [JsonPropertyName("Persona")]
    public string Persona { get; set; } = "你是一个友好、随和、什么都愿意聊的游戏内聊天助手。优先用中文回复，语气自然像真人玩家。回复要短，不要刷屏，不要总是劝人专心打游戏；只有玩家主动聊 CS2 或对局时，才轻微带一点游戏语境。";

    [JsonPropertyName("Provider")]
    public LlmProviderConfig Provider { get; set; } = new();

    [JsonPropertyName("Chat")]
    public ChatBehaviorConfig Chat { get; set; } = new();

    public void Normalize()
    {
        NormalizeLegacyFields();

        var usingLegacyConfig = Personas.Count == 0;
        if (usingLegacyConfig)
        {
            ApplyLegacyConfigToV2();
        }
        else
        {
            MergeLegacySecretsForConvenience();
        }

        Global.Normalize();
        BotHider.Normalize();

        Personas = Personas
            .Where(persona => persona is not null)
            .Select(persona =>
            {
                persona.Normalize();
                return persona;
            })
            .Where(persona => !string.IsNullOrWhiteSpace(persona.Id))
            .GroupBy(persona => persona.Id, StringComparer.OrdinalIgnoreCase)
            .Select(group => group.First())
            .ToList();

        if (Personas.Count == 0)
        {
            Personas.Add(BotPersonaConfig.FromLegacy(BotName, Aliases, Persona, Chat));
        }

        if (!Personas.Any(persona => persona.Enabled))
        {
            Personas[0].Enabled = true;
        }
    }

    public BotPersonaConfig GetSharedPersona()
    {
        return Personas.FirstOrDefault(persona => persona.Enabled) ?? Personas[0];
    }

    public BotChatConfig CloneForRuntime()
    {
        var json = JsonSerializer.Serialize(this, JsonSerialization.Default);
        var clone = JsonSerializer.Deserialize<BotChatConfig>(json, JsonSerialization.Default) ?? new BotChatConfig();
        clone.Normalize();
        return clone;
    }

    private void NormalizeLegacyFields()
    {
        BotName = string.IsNullOrWhiteSpace(BotName) ? "s1mple" : BotName.Trim();
        Persona = string.IsNullOrWhiteSpace(Persona)
            ? $"You are {BotName}, a friendly in-game chat assistant who can talk about anything. Reply briefly and naturally. Do not keep telling players to focus on the match."
            : Persona.Trim();

        Aliases = Aliases
            .Where(alias => !string.IsNullOrWhiteSpace(alias))
            .Select(alias => alias.Trim())
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();

        if (Aliases.Length == 0)
        {
            Aliases = [$"@{BotName}", BotName];
        }

        Provider.Normalize();
        Chat.Normalize();
    }

    private void ApplyLegacyConfigToV2()
    {
        Global.Enabled = Enabled;
        Global.Provider = Provider.Clone();
        Global.Trigger.RequireMention = Chat.RequireMention;
        Global.Output.PreferBotSay = Chat.PreferBotSay;
        Global.Output.MaxReplyChars = Math.Min(Chat.MaxReplyChars, 160);
        Global.Output.FallbackOnEmpty = false;
        Global.RateLimit.PerPlayerCooldownMs = Chat.PerPlayerCooldownSeconds * 1000;
        Global.RateLimit.GlobalCooldownMs = Chat.BotCooldownSeconds * 1000;
        Global.RateLimit.PerBotCooldownMs = Math.Max(Chat.BotCooldownSeconds * 1000, 2000);
        Personas.Add(BotPersonaConfig.FromLegacy(BotName, Aliases, Persona, Chat));
    }

    private void MergeLegacySecretsForConvenience()
    {
        if (string.IsNullOrWhiteSpace(Global.Provider.ApiKey) && !string.IsNullOrWhiteSpace(Provider.ApiKey))
        {
            Global.Provider.ApiKey = Provider.ApiKey;
        }

        if (string.IsNullOrWhiteSpace(Global.Provider.ApiKeyEnvVar) && !string.IsNullOrWhiteSpace(Provider.ApiKeyEnvVar))
        {
            Global.Provider.ApiKeyEnvVar = Provider.ApiKeyEnvVar;
        }
    }
}

public sealed class GlobalBotConfig
{
    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; } = true;

    [JsonPropertyName("provider")]
    public LlmProviderConfig Provider { get; set; } = new();

    [JsonPropertyName("trigger")]
    public TriggerConfig Trigger { get; set; } = new();

    [JsonPropertyName("rateLimit")]
    public RateLimitConfig RateLimit { get; set; } = new();

    [JsonPropertyName("selection")]
    public SelectionConfig Selection { get; set; } = new();

    [JsonPropertyName("memory")]
    public MemoryConfig Memory { get; set; } = new();

    [JsonPropertyName("antiLoop")]
    public AntiLoopConfig AntiLoop { get; set; } = new();

    [JsonPropertyName("output")]
    public OutputConfig Output { get; set; } = new();

    [JsonPropertyName("botInteraction")]
    public BotInteractionConfig BotInteraction { get; set; } = new();

    [JsonPropertyName("tokenBudget")]
    public TokenBudgetConfig TokenBudget { get; set; } = new();

    [JsonPropertyName("debug")]
    public DebugConfig Debug { get; set; } = new();

    public void Normalize()
    {
        Provider.Normalize();
        Trigger.Normalize();
        RateLimit.Normalize();
        Selection.Normalize();
        Memory.Normalize();
        Output.Normalize();
        TokenBudget.Normalize();
    }
}

public sealed class LlmProviderConfig
{
    [JsonPropertyName("BaseUrl")]
    public string BaseUrl { get; set; } = "https://api.deepseek.com";

    [JsonPropertyName("Model")]
    public string Model { get; set; } = "deepseek-chat";

    [JsonPropertyName("ApiKey")]
    public string ApiKey { get; set; } = string.Empty;

    [JsonPropertyName("ApiKeyEnvVar")]
    public string ApiKeyEnvVar { get; set; } = "CS2_BOT_LLM_API_KEY";

    [JsonPropertyName("TimeoutSeconds")]
    public int TimeoutSeconds { get; set; } = 8;

    [JsonPropertyName("TimeoutMs")]
    public int TimeoutMs { get; set; } = 3500;

    [JsonPropertyName("MaxTokens")]
    public int MaxTokens { get; set; } = 80;

    [JsonPropertyName("MaxOutputTokens")]
    public int MaxOutputTokens { get; set; } = 96;

    [JsonPropertyName("Temperature")]
    public double Temperature { get; set; } = 0.85;

    [JsonPropertyName("TopP")]
    public double TopP { get; set; } = 0.9;

    public void Normalize()
    {
        BaseUrl = string.IsNullOrWhiteSpace(BaseUrl) ? "https://api.deepseek.com" : BaseUrl.Trim();
        Model = string.IsNullOrWhiteSpace(Model) ? "deepseek-chat" : Model.Trim();
        ApiKey = ApiKey.Trim();
        ApiKeyEnvVar = string.IsNullOrWhiteSpace(ApiKeyEnvVar) ? "CS2_BOT_LLM_API_KEY" : ApiKeyEnvVar.Trim();

        TimeoutSeconds = Math.Clamp(TimeoutSeconds, 1, 60);
        TimeoutMs = TimeoutMs <= 0 ? TimeoutSeconds * 1000 : Math.Clamp(TimeoutMs, 500, 60000);

        if (MaxTokens != 80 && MaxOutputTokens == 96)
        {
            MaxOutputTokens = MaxTokens;
        }
        else if (MaxOutputTokens != 96 && MaxTokens == 80)
        {
            MaxTokens = MaxOutputTokens;
        }

        MaxTokens = Math.Clamp(MaxTokens, 16, 512);
        MaxOutputTokens = Math.Clamp(MaxOutputTokens, 16, 512);
        Temperature = Math.Clamp(Temperature, 0.0, 2.0);
        TopP = Math.Clamp(TopP, 0.0, 1.0);
    }

    public string ResolveApiKey()
    {
        if (!string.IsNullOrWhiteSpace(ApiKey))
        {
            var trimmed = ApiKey.Trim();
            if (trimmed.StartsWith("ENV:", StringComparison.OrdinalIgnoreCase))
            {
                var envName = trimmed[4..].Trim();
                return string.IsNullOrWhiteSpace(envName)
                    ? string.Empty
                    : Environment.GetEnvironmentVariable(envName)?.Trim() ?? string.Empty;
            }

            return trimmed;
        }

        return Environment.GetEnvironmentVariable(ApiKeyEnvVar)?.Trim() ?? string.Empty;
    }

    public LlmProviderConfig Clone()
    {
        return new LlmProviderConfig
        {
            BaseUrl = BaseUrl,
            Model = Model,
            ApiKey = ApiKey,
            ApiKeyEnvVar = ApiKeyEnvVar,
            TimeoutSeconds = TimeoutSeconds,
            TimeoutMs = TimeoutMs,
            MaxTokens = MaxTokens,
            MaxOutputTokens = MaxOutputTokens,
            Temperature = Temperature,
            TopP = TopP
        };
    }
}

public sealed class TriggerConfig
{
    [JsonPropertyName("requireMention")]
    public bool RequireMention { get; set; }

    [JsonPropertyName("mentionPrefixes")]
    public string[] MentionPrefixes { get; set; } = ["@", "bot ", "机器人 "];

    [JsonPropertyName("ignoreCommandsStartingWith")]
    public string[] IgnoreCommandsStartingWith { get; set; } = ["!", "/", "."];

    [JsonPropertyName("ignoreEmptyOrTooShort")]
    public bool IgnoreEmptyOrTooShort { get; set; } = true;

    [JsonPropertyName("minMessageLength")]
    public int MinMessageLength { get; set; } = 2;

    [JsonPropertyName("maxInputChars")]
    public int MaxInputChars { get; set; } = 240;

    public void Normalize()
    {
        MentionPrefixes = NormalizeStringArray(MentionPrefixes, ["@", "bot ", "机器人 "]);
        IgnoreCommandsStartingWith = NormalizeStringArray(IgnoreCommandsStartingWith, ["!", "/", "."]);
        MinMessageLength = Math.Clamp(MinMessageLength, 1, 32);
        MaxInputChars = Math.Clamp(MaxInputChars, MinMessageLength, 1000);
    }
}

public sealed class RateLimitConfig
{
    [JsonPropertyName("globalCooldownMs")]
    public int GlobalCooldownMs { get; set; } = 6500;

    [JsonPropertyName("perBotCooldownMs")]
    public int PerBotCooldownMs { get; set; } = 18000;

    [JsonPropertyName("perPlayerCooldownMs")]
    public int PerPlayerCooldownMs { get; set; } = 10000;

    [JsonPropertyName("maxRepliesPerMinute")]
    public int MaxRepliesPerMinute { get; set; } = 5;

    [JsonPropertyName("maxRepliesPerRound")]
    public int MaxRepliesPerRound { get; set; } = 12;

    [JsonPropertyName("suppressAfterConsecutiveBotReplies")]
    public int SuppressAfterConsecutiveBotReplies { get; set; } = 1;

    [JsonPropertyName("allowMentionBypassCooldown")]
    public bool AllowMentionBypassCooldown { get; set; } = true;

    [JsonPropertyName("mentionBypassMinCooldownMs")]
    public int MentionBypassMinCooldownMs { get; set; } = 5000;

    [JsonPropertyName("allowConcurrentRequests")]
    public bool AllowConcurrentRequests { get; set; }

    public void Normalize()
    {
        GlobalCooldownMs = Math.Clamp(GlobalCooldownMs, 0, 120000);
        PerBotCooldownMs = Math.Clamp(PerBotCooldownMs, 0, 600000);
        PerPlayerCooldownMs = Math.Clamp(PerPlayerCooldownMs, 0, 600000);
        MaxRepliesPerMinute = Math.Clamp(MaxRepliesPerMinute, 0, 120);
        MaxRepliesPerRound = Math.Clamp(MaxRepliesPerRound, 0, 200);
        SuppressAfterConsecutiveBotReplies = Math.Clamp(SuppressAfterConsecutiveBotReplies, 0, 20);
        MentionBypassMinCooldownMs = Math.Clamp(MentionBypassMinCooldownMs, 0, PerBotCooldownMs);
    }
}

public sealed class SelectionConfig
{
    [JsonPropertyName("mode")]
    public string Mode { get; set; } = "rules";

    [JsonPropertyName("noMentionBaseReplyChance")]
    public double NoMentionBaseReplyChance { get; set; } = 0.18;

    [JsonPropertyName("mentionReplyChance")]
    public double MentionReplyChance { get; set; } = 0.95;

    [JsonPropertyName("questionBoost")]
    public double QuestionBoost { get; set; } = 0.18;

    [JsonPropertyName("nameMentionBoost")]
    public double NameMentionBoost { get; set; } = 0.55;

    [JsonPropertyName("recentlySpokenPenalty")]
    public double RecentlySpokenPenalty { get; set; } = 0.35;

    [JsonPropertyName("sameBotRepeatPenalty")]
    public double SameBotRepeatPenalty { get; set; } = 0.45;

    [JsonPropertyName("teamOnlyBias")]
    public double TeamOnlyBias { get; set; } = 0.15;

    [JsonPropertyName("randomJitter")]
    public double RandomJitter { get; set; } = 0.12;

    public void Normalize()
    {
        Mode = string.IsNullOrWhiteSpace(Mode) ? "rules" : Mode.Trim();
        NoMentionBaseReplyChance = ClampChance(NoMentionBaseReplyChance);
        MentionReplyChance = ClampChance(MentionReplyChance);
        QuestionBoost = Math.Clamp(QuestionBoost, 0, 1);
        NameMentionBoost = Math.Clamp(NameMentionBoost, 0, 1);
        RecentlySpokenPenalty = Math.Clamp(RecentlySpokenPenalty, 0, 1);
        SameBotRepeatPenalty = Math.Clamp(SameBotRepeatPenalty, 0, 1);
        TeamOnlyBias = Math.Clamp(TeamOnlyBias, 0, 1);
        RandomJitter = Math.Clamp(RandomJitter, 0, 1);
    }
}

public sealed class MemoryConfig
{
    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; } = true;

    [JsonPropertyName("recentContextMessages")]
    public int RecentContextMessages { get; set; } = 6;

    [JsonPropertyName("recentContextMaxChars")]
    public int RecentContextMaxChars { get; set; } = 700;

    [JsonPropertyName("perBotShortMemoryEnabled")]
    public bool PerBotShortMemoryEnabled { get; set; }

    [JsonPropertyName("perBotShortMemoryMaxItems")]
    public int PerBotShortMemoryMaxItems { get; set; }

    [JsonPropertyName("perBotShortMemoryMaxChars")]
    public int PerBotShortMemoryMaxChars { get; set; }

    [JsonPropertyName("globalRollingSummaryEnabled")]
    public bool GlobalRollingSummaryEnabled { get; set; }

    [JsonPropertyName("globalRollingSummaryMaxChars")]
    public int GlobalRollingSummaryMaxChars { get; set; } = 300;

    [JsonPropertyName("storeBotMessagesInRecentContext")]
    public bool StoreBotMessagesInRecentContext { get; set; } = true;

    [JsonPropertyName("storeFullPlayerMessages")]
    public bool StoreFullPlayerMessages { get; set; }

    public void Normalize()
    {
        RecentContextMessages = Math.Clamp(RecentContextMessages, 0, 20);
        RecentContextMaxChars = Math.Clamp(RecentContextMaxChars, 0, 4000);
        PerBotShortMemoryMaxItems = Math.Clamp(PerBotShortMemoryMaxItems, 0, 20);
        PerBotShortMemoryMaxChars = Math.Clamp(PerBotShortMemoryMaxChars, 0, 2000);
        GlobalRollingSummaryMaxChars = Math.Clamp(GlobalRollingSummaryMaxChars, 0, 2000);
    }
}

public sealed class AntiLoopConfig
{
    [JsonPropertyName("ignoreBotMessages")]
    public bool IgnoreBotMessages { get; set; } = true;

    [JsonPropertyName("markOwnBotMessages")]
    public bool MarkOwnBotMessages { get; set; } = true;

    [JsonPropertyName("ownBotMessageTtlMs")]
    public int OwnBotMessageTtlMs { get; set; } = 15000;

    [JsonPropertyName("ignoreMessagesFromKnownBotSlots")]
    public bool IgnoreMessagesFromKnownBotSlots { get; set; } = true;

    [JsonPropertyName("ignoreMessagesMatchingRecentBotOutputs")]
    public bool IgnoreMessagesMatchingRecentBotOutputs { get; set; } = true;
}

public sealed class OutputConfig
{
    [JsonPropertyName("preferBotSay")]
    public bool PreferBotSay { get; set; } = true;

    [JsonPropertyName("maxReplyChars")]
    public int MaxReplyChars { get; set; } = 96;

    [JsonPropertyName("stripNewlines")]
    public bool StripNewlines { get; set; } = true;

    [JsonPropertyName("stripQuotes")]
    public bool StripQuotes { get; set; } = true;

    [JsonPropertyName("forbiddenPrefixes")]
    public string[] ForbiddenPrefixes { get; set; } = ["作为一个AI", "我是一个AI", "As an AI"];

    [JsonPropertyName("fallbackOnEmpty")]
    public bool FallbackOnEmpty { get; set; }

    public void Normalize()
    {
        MaxReplyChars = Math.Clamp(MaxReplyChars, 20, 800);
        ForbiddenPrefixes = NormalizeStringArray(ForbiddenPrefixes, ["作为一个AI", "我是一个AI", "As an AI"]);
    }
}

public sealed class BotInteractionConfig
{
    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; }

    [JsonPropertyName("allowFollowUpAfterPlayerMessage")]
    public bool AllowFollowUpAfterPlayerMessage { get; set; }

    [JsonPropertyName("maxBotFollowUps")]
    public int MaxBotFollowUps { get; set; } = 1;

    [JsonPropertyName("followUpChance")]
    public double FollowUpChance { get; set; } = 0.08;

    [JsonPropertyName("followUpCooldownMs")]
    public int FollowUpCooldownMs { get; set; } = 30000;

    [JsonPropertyName("followUpRequiresDifferentBot")]
    public bool FollowUpRequiresDifferentBot { get; set; } = true;
}

public sealed class TokenBudgetConfig
{
    [JsonPropertyName("maxPromptTokensApprox")]
    public int MaxPromptTokensApprox { get; set; } = 900;

    [JsonPropertyName("maxSystemChars")]
    public int MaxSystemChars { get; set; } = 1200;

    [JsonPropertyName("maxPersonaChars")]
    public int MaxPersonaChars { get; set; } = 700;

    [JsonPropertyName("maxRecentContextChars")]
    public int MaxRecentContextChars { get; set; } = 700;

    [JsonPropertyName("maxMemoryChars")]
    public int MaxMemoryChars { get; set; } = 0;

    [JsonPropertyName("truncateFromOldest")]
    public bool TruncateFromOldest { get; set; } = true;

    public void Normalize()
    {
        MaxPromptTokensApprox = Math.Clamp(MaxPromptTokensApprox, 200, 4000);
        MaxSystemChars = Math.Clamp(MaxSystemChars, 200, 4000);
        MaxPersonaChars = Math.Clamp(MaxPersonaChars, 200, 3000);
        MaxRecentContextChars = Math.Clamp(MaxRecentContextChars, 0, 3000);
        MaxMemoryChars = Math.Clamp(MaxMemoryChars, 0, 2000);
    }
}

public sealed class DebugConfig
{
    [JsonPropertyName("logSelection")]
    public bool LogSelection { get; set; } = true;

    [JsonPropertyName("logPromptStats")]
    public bool LogPromptStats { get; set; } = true;

    [JsonPropertyName("logBotIdentityResolution")]
    public bool LogBotIdentityResolution { get; set; }

    [JsonPropertyName("logLlmLatency")]
    public bool LogLlmLatency { get; set; } = true;

    [JsonPropertyName("dryRunNoSay")]
    public bool DryRunNoSay { get; set; }
}

public sealed class BotHiderIntegrationConfig
{
    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; } = true;

    [JsonPropertyName("bindingStrategy")]
    public string BindingStrategy { get; set; } = "slotThenNameThenPool";

    [JsonPropertyName("managedSlots")]
    public List<BotHiderManagedSlotConfig> ManagedSlots { get; set; } = [];

    [JsonPropertyName("unboundManagedSlotPolicy")]
    public string UnboundManagedSlotPolicy { get; set; } = "assignFromPersonaPool";

    [JsonPropertyName("refreshOnMapStart")]
    public bool RefreshOnMapStart { get; set; } = true;

    [JsonPropertyName("refreshOnPlayerConnect")]
    public bool RefreshOnPlayerConnect { get; set; } = true;

    public void Normalize()
    {
        BindingStrategy = string.IsNullOrWhiteSpace(BindingStrategy) ? "slotThenNameThenPool" : BindingStrategy.Trim();
        UnboundManagedSlotPolicy = string.IsNullOrWhiteSpace(UnboundManagedSlotPolicy) ? "assignFromPersonaPool" : UnboundManagedSlotPolicy.Trim();
        ManagedSlots = ManagedSlots
            .Where(slot => slot is not null)
            .Select(slot =>
            {
                slot.Normalize();
                return slot;
            })
            .GroupBy(slot => slot.Slot)
            .Select(group => group.First())
            .ToList();
    }
}

public sealed class BotHiderManagedSlotConfig
{
    [JsonPropertyName("slot")]
    public int Slot { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("personaId")]
    public string PersonaId { get; set; } = string.Empty;

    [JsonPropertyName("aliases")]
    public string[] Aliases { get; set; } = [];

    public void Normalize()
    {
        Slot = Math.Clamp(Slot, 0, 255);
        Name = Name.Trim();
        PersonaId = PersonaId.Trim();
        Aliases = NormalizeStringArray(Aliases, []);
    }
}

public sealed class BotPersonaConfig
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("enabled")]
    public bool Enabled { get; set; } = true;

    [JsonPropertyName("displayName")]
    public string DisplayName { get; set; } = string.Empty;

    [JsonPropertyName("aliases")]
    public string[] Aliases { get; set; } = [];

    [JsonPropertyName("binding")]
    public PersonaBindingConfig Binding { get; set; } = new();

    [JsonPropertyName("style")]
    public PersonaStyleConfig Style { get; set; } = new();

    [JsonPropertyName("character")]
    public PersonaCharacterConfig Character { get; set; } = new();

    [JsonPropertyName("selection")]
    public PersonaSelectionConfig Selection { get; set; } = new();

    public void Normalize()
    {
        Id = string.IsNullOrWhiteSpace(Id) ? TextKey.Normalize(DisplayName) : TextKey.Normalize(Id);
        DisplayName = string.IsNullOrWhiteSpace(DisplayName) ? Id : DisplayName.Trim();
        Aliases = NormalizeStringArray(Aliases, [DisplayName]);
        Binding.Normalize();
        Style.Normalize();
        Character.Normalize();
        Selection.Normalize();
    }

    public static BotPersonaConfig FromLegacy(string botName, IEnumerable<string> aliases, string persona, ChatBehaviorConfig chat)
    {
        var aliasArray = aliases
            .Where(alias => !string.IsNullOrWhiteSpace(alias))
            .Select(alias => alias.Trim())
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray();

        return new BotPersonaConfig
        {
            Id = TextKey.Normalize(botName),
            DisplayName = botName,
            Aliases = aliasArray.Length == 0 ? [$"@{botName}", botName] : aliasArray,
            Binding = new PersonaBindingConfig
            {
                Type = "pool",
                AllowBotHiderManagedSlot = true
            },
            Style = new PersonaStyleConfig
            {
                Tone = persona,
                MaxSentences = 2
            },
            Character = new PersonaCharacterConfig
            {
                Role = "随机被选中发言的普通 CS2 bot",
                Traits = ["随和", "自然", "短句"],
                Interests = ["最近几句聊天", "普通闲聊", "轻度 CS2 语境"],
                Boundaries = ["不刷屏", "不攻击现实身份", "不假装自己有复杂身份"]
            },
            Selection = new PersonaSelectionConfig
            {
                BaseReplyChance = chat.RequireMention ? 0.0 : 0.18,
                MentionReplyChance = 0.98,
                CooldownMs = Math.Max(chat.BotCooldownSeconds * 1000, 2000),
                ActivityWeight = 1.0
            }
        };
    }
}

public sealed class PersonaBindingConfig
{
    [JsonPropertyName("type")]
    public string Type { get; set; } = "pool";

    [JsonPropertyName("slot")]
    public int? Slot { get; set; }

    [JsonPropertyName("fallbackNameContains")]
    public string[] FallbackNameContains { get; set; } = [];

    [JsonPropertyName("nameContains")]
    public string[] NameContains { get; set; } = [];

    [JsonPropertyName("steamId")]
    public string SteamId { get; set; } = string.Empty;

    [JsonPropertyName("allowBotHiderManagedSlot")]
    public bool AllowBotHiderManagedSlot { get; set; } = true;

    public void Normalize()
    {
        Type = string.IsNullOrWhiteSpace(Type) ? "pool" : Type.Trim();
        if (Slot is < 0)
        {
            Slot = null;
        }

        SteamId = SteamId.Trim();
        FallbackNameContains = NormalizeStringArray(FallbackNameContains, []);
        NameContains = NormalizeStringArray(NameContains, []);
    }
}

public sealed class PersonaStyleConfig
{
    [JsonPropertyName("language")]
    public string Language { get; set; } = "zh-CN";

    [JsonPropertyName("tone")]
    public string Tone { get; set; } = "轻松、自然、短句，像真人玩家随口接话";

    [JsonPropertyName("catchphrases")]
    public string[] Catchphrases { get; set; } = [];

    [JsonPropertyName("avoid")]
    public string[] Avoid { get; set; } = ["长篇说教", "总是提醒专心打游戏", "AI 助手口吻"];

    [JsonPropertyName("maxSentences")]
    public int MaxSentences { get; set; } = 2;

    public void Normalize()
    {
        Language = string.IsNullOrWhiteSpace(Language) ? "zh-CN" : Language.Trim();
        Tone = string.IsNullOrWhiteSpace(Tone) ? "轻松、自然、短句，像真人玩家随口接话" : Tone.Trim();
        Catchphrases = NormalizeStringArray(Catchphrases, []);
        Avoid = NormalizeStringArray(Avoid, ["长篇说教", "总是提醒专心打游戏", "AI 助手口吻"]);
        MaxSentences = Math.Clamp(MaxSentences, 1, 4);
    }
}

public sealed class PersonaCharacterConfig
{
    [JsonPropertyName("role")]
    public string Role { get; set; } = "喜欢接玩家闲聊的 bot";

    [JsonPropertyName("traits")]
    public string[] Traits { get; set; } = ["自然", "反应快"];

    [JsonPropertyName("interests")]
    public string[] Interests { get; set; } = ["玩家聊天", "局势吐槽", "普通问题"];

    [JsonPropertyName("boundaries")]
    public string[] Boundaries { get; set; } = ["不刷屏", "不持续针对同一个玩家"];

    public void Normalize()
    {
        Role = string.IsNullOrWhiteSpace(Role) ? "喜欢接玩家闲聊的 bot" : Role.Trim();
        Traits = NormalizeStringArray(Traits, ["自然", "反应快"]);
        Interests = NormalizeStringArray(Interests, ["玩家聊天", "局势吐槽", "普通问题"]);
        Boundaries = NormalizeStringArray(Boundaries, ["不刷屏", "不持续针对同一个玩家"]);
    }
}

public sealed class PersonaSelectionConfig
{
    [JsonPropertyName("baseReplyChance")]
    public double BaseReplyChance { get; set; } = 0.18;

    [JsonPropertyName("mentionReplyChance")]
    public double MentionReplyChance { get; set; } = 0.98;

    [JsonPropertyName("questionReplyChance")]
    public double QuestionReplyChance { get; set; } = 0.38;

    [JsonPropertyName("cooldownMs")]
    public int CooldownMs { get; set; } = 18000;

    [JsonPropertyName("maxRepliesPerMinute")]
    public int MaxRepliesPerMinute { get; set; } = 2;

    [JsonPropertyName("activityWeight")]
    public double ActivityWeight { get; set; } = 1.0;

    public void Normalize()
    {
        BaseReplyChance = ClampChance(BaseReplyChance);
        MentionReplyChance = ClampChance(MentionReplyChance);
        QuestionReplyChance = ClampChance(QuestionReplyChance);
        CooldownMs = Math.Clamp(CooldownMs, 0, 600000);
        MaxRepliesPerMinute = Math.Clamp(MaxRepliesPerMinute, 0, 60);
        ActivityWeight = Math.Clamp(ActivityWeight, 0.05, 10);
    }
}

public sealed class ChatBehaviorConfig
{
    [JsonPropertyName("RequireMention")]
    public bool RequireMention { get; set; } = true;

    [JsonPropertyName("PreferBotSay")]
    public bool PreferBotSay { get; set; }

    [JsonPropertyName("MaxReplyChars")]
    public int MaxReplyChars { get; set; } = 220;

    [JsonPropertyName("MaxChunkChars")]
    public int MaxChunkChars { get; set; } = 110;

    [JsonPropertyName("MaxChunks")]
    public int MaxChunks { get; set; } = 3;

    [JsonPropertyName("ChunkDelaySeconds")]
    public float ChunkDelaySeconds { get; set; } = 0.9f;

    [JsonPropertyName("PerPlayerCooldownSeconds")]
    public int PerPlayerCooldownSeconds { get; set; } = 5;

    [JsonPropertyName("BotCooldownSeconds")]
    public int BotCooldownSeconds { get; set; } = 2;

    public void Normalize()
    {
        MaxReplyChars = Math.Clamp(MaxReplyChars, 40, 800);
        MaxChunkChars = Math.Clamp(MaxChunkChars, 20, MaxReplyChars);
        MaxChunks = Math.Clamp(MaxChunks, 1, 8);
        ChunkDelaySeconds = Math.Clamp(ChunkDelaySeconds, 0.0f, 5.0f);
        PerPlayerCooldownSeconds = Math.Clamp(PerPlayerCooldownSeconds, 0, 600);
        BotCooldownSeconds = Math.Clamp(BotCooldownSeconds, 0, 600);
    }
}

public readonly record struct ConfigValidationResult(bool IsValid, IReadOnlyList<string> Errors, IReadOnlyList<string> Warnings);

public static class BotChatConfigValidator
{
    public static ConfigValidationResult ValidateRaw(BotChatConfig config)
    {
        var errors = new List<string>();
        var warnings = new List<string>();
        var personas = config.Personas ?? [];
        var managedSlots = config.BotHider?.ManagedSlots ?? [];
        var enabledPersonas = personas.Where(persona => persona is { Enabled: true }).ToArray();
        var nonNullManagedSlots = managedSlots.Where(slot => slot is not null).ToArray();

        if (config.Global is null)
        {
            errors.Add("Global config is required.");
        }

        if (config.BotHider is null)
        {
            errors.Add("BotHider config is required.");
        }

        if (config.Personas is null)
        {
            errors.Add("Personas must be an array when present.");
        }

        AddDuplicateErrors(
            enabledPersonas.Select(GetRawPersonaId),
            "persona id",
            errors);

        AddDuplicateErrors(
            nonNullManagedSlots.Select(slot => slot.Slot.ToString(System.Globalization.CultureInfo.InvariantCulture)),
            "BotHider managed slot",
            errors);

        AddAliasConflictWarnings(enabledPersonas, warnings);

        return new ConfigValidationResult(errors.Count == 0, errors, warnings);
    }

    public static ConfigValidationResult Validate(BotChatConfig config)
    {
        var errors = new List<string>();
        var warnings = new List<string>();

        var enabledPersonas = config.Personas.Where(persona => persona.Enabled).ToArray();
        if (enabledPersonas.Length == 0)
        {
            errors.Add("At least one enabled persona is required.");
        }

        AddDuplicateErrors(
            enabledPersonas.Select(persona => persona.Id),
            "persona id",
            errors);

        AddAliasConflictWarnings(enabledPersonas, warnings);

        AddDuplicateErrors(
            config.BotHider.ManagedSlots.Select(slot => slot.Slot.ToString(System.Globalization.CultureInfo.InvariantCulture)),
            "BotHider managed slot",
            errors);

        return new ConfigValidationResult(errors.Count == 0, errors, warnings);
    }

    private static string GetRawPersonaId(BotPersonaConfig persona)
    {
        return string.IsNullOrWhiteSpace(persona.Id)
            ? TextKey.Normalize(persona.DisplayName)
            : TextKey.Normalize(persona.Id);
    }

    private static void AddDuplicateErrors(IEnumerable<string> values, string label, ICollection<string> output)
    {
        foreach (var duplicate in values
            .Where(value => !string.IsNullOrWhiteSpace(value))
            .GroupBy(value => value, StringComparer.OrdinalIgnoreCase)
            .Where(group => group.Count() > 1)
            .Select(group => group.Key))
        {
            output.Add($"Duplicate {label}: {duplicate}");
        }
    }

    private static void AddAliasConflictWarnings(IEnumerable<BotPersonaConfig> personas, ICollection<string> output)
    {
        var aliases = personas
            .SelectMany(persona => PersonaRegistry
                .GetMentionAliases(persona)
                .Select(alias => new
                {
                    PersonaId = persona.Id,
                    Alias = TextKey.NormalizeAlias(alias)
                }))
            .Where(item => item.Alias.Length > 0)
            .Distinct()
            .ToArray();

        foreach (var conflict in aliases
            .GroupBy(item => item.Alias, StringComparer.OrdinalIgnoreCase)
            .Where(group => group.Select(item => item.PersonaId).Distinct(StringComparer.OrdinalIgnoreCase).Count() > 1))
        {
            output.Add($"Duplicate persona alias across profiles: {conflict.Key}");
        }
    }
}

internal static class ConfigHelpers
{
    public static string[] NormalizeStringArray(IEnumerable<string>? values, string[] fallback)
    {
        var normalized = values?
            .Where(value => !string.IsNullOrWhiteSpace(value))
            .Select(value => value.Trim())
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToArray() ?? [];

        return normalized.Length == 0 ? fallback : normalized;
    }

    public static double ClampChance(double value)
    {
        if (double.IsNaN(value) || double.IsInfinity(value))
        {
            return 0;
        }

        return Math.Clamp(value, 0, 1);
    }
}
