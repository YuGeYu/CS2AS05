using System.Text.Json;
using CounterStrikeSharp.API.Modules.Utils;
using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class SharedBotBehaviorTests
{
    [Fact]
    public void Config_ParsesSharedBotProfile()
    {
        const string json = """
        {
          "version": 2,
          "global": {
            "provider": {
              "BaseUrl": "https://example.test",
              "Model": "test-model",
              "ApiKey": "ENV:TEST_KEY",
              "TimeoutMs": 3500,
              "MaxOutputTokens": 80
            },
            "memory": {
              "perBotShortMemoryEnabled": false
            }
          },
          "personas": [
            {
              "id": "shared-bot",
              "displayName": "bot",
              "aliases": ["@bot", "机器人"]
            }
          ]
        }
        """;

        var config = JsonSerializer.Deserialize<BotChatConfig>(json, JsonSerialization.Default)!;
        config.Normalize();

        Assert.Equal("test-model", config.Global.Provider.Model);
        Assert.Equal("shared-bot", config.GetSharedPersona().Id);
        Assert.False(config.Global.Memory.PerBotShortMemoryEnabled);
    }

    [Fact]
    public void PersonaRegistry_MatchesSharedBotAlias()
    {
        var config = CreateSharedConfig();
        var registry = new PersonaRegistry([config.GetSharedPersona()]);

        var match = registry.MatchMention("@bot 你怎么看这把？");

        Assert.NotNull(match);
        Assert.Equal("shared-bot", match.Persona.Id);
        Assert.Equal("你怎么看这把？", match.UserMessage);
    }

    [Fact]
    public void Selector_PicksFromMultipleBotsUsingSameSharedProfile()
    {
        var config = CreateSharedConfig();
        config.Global.Selection.RandomJitter = 0;
        config.Global.Selection.NoMentionBaseReplyChance = 1;
        config.Global.RateLimit.GlobalCooldownMs = 0;
        config.Global.RateLimit.PerPlayerCooldownMs = 0;
        config.Global.RateLimit.PerBotCooldownMs = 0;
        config.Normalize();

        var registry = new PersonaRegistry([config.GetSharedPersona()]);
        var now = DateTimeOffset.UtcNow;
        var state = new ConversationMemory().GetState(now);
        var ev = new ChatEvent(1, "uni", true, false, "这把有点怪", now, ChatChannel.All, CsTeam.Terrorist);
        var intent = new MessageIntentParser().Parse(ev.Text, config.Global.Trigger, registry, state);
        var activeBots = new[]
        {
            new ResolvedBot("shared-bot", "bot", 3, true, "Rex", CsTeam.Terrorist),
            new ResolvedBot("shared-bot", "bot", 4, true, "Mika", CsTeam.CounterTerrorist)
        };

        var first = new BotReplySelector(registry, config.Global, new FixedRandomSource(0, 0))
            .Select(ev, new SelectionContext(intent, activeBots, state, now, "1"));
        var second = new BotReplySelector(registry, config.Global, new FixedRandomSource(0, 0.9))
            .Select(ev, new SelectionContext(intent, activeBots, state, now, "1"));

        Assert.True(first.ShouldReply);
        Assert.True(second.ShouldReply);
        Assert.Equal("shared-bot", first.PersonaId);
        Assert.Equal("shared-bot", second.PersonaId);
        Assert.Equal(3, first.Bot?.Slot);
        Assert.Equal(4, second.Bot?.Slot);
    }

    [Fact]
    public void Selector_DoesNotLockWholeSpeakerPoolWhenOneBotIsCoolingDown()
    {
        var config = CreateSharedConfig();
        config.Global.Selection.RandomJitter = 0;
        config.Global.Selection.NoMentionBaseReplyChance = 1;
        config.Global.RateLimit.GlobalCooldownMs = 0;
        config.Global.RateLimit.PerPlayerCooldownMs = 0;
        config.Global.RateLimit.PerBotCooldownMs = 6000;
        config.Global.RateLimit.SuppressAfterConsecutiveBotReplies = 0;
        config.Normalize();

        var registry = new PersonaRegistry([config.GetSharedPersona()]);
        var memory = new ConversationMemory();
        var now = DateTimeOffset.UtcNow;
        memory.AddBotMessage("shared-bot", 3, "Rex", "刚才我说过了", now - TimeSpan.FromSeconds(2), config.Global.Memory);

        var state = memory.GetState(now);
        var ev = new ChatEvent(1, "uni", true, false, "再说一句", now, ChatChannel.All, CsTeam.Terrorist);
        var intent = new MessageIntentParser().Parse(ev.Text, config.Global.Trigger, registry, state);
        var activeBots = new[]
        {
            new ResolvedBot("shared-bot", "bot", 3, true, "Rex", CsTeam.Terrorist),
            new ResolvedBot("shared-bot", "bot", 4, true, "Mika", CsTeam.CounterTerrorist)
        };

        var selection = new BotReplySelector(registry, config.Global, new FixedRandomSource(0, 0))
            .Select(ev, new SelectionContext(intent, activeBots, state, now, "1"));

        Assert.True(selection.ShouldReply);
        Assert.Equal(4, selection.Bot?.Slot);
    }

    [Fact]
    public void PromptBuilder_UsesRecentContextWithoutOtherPersonaDetails()
    {
        var config = CreateSharedConfig();
        var persona = config.GetSharedPersona();
        var prompt = new PromptBuilder().Build(
            persona,
            new ResolvedBot("shared-bot", "bot", 4, true, "Mika", CsTeam.Terrorist),
            new ChatEvent(1, "uni", true, false, "你怎么看？", DateTimeOffset.UtcNow, ChatChannel.All, CsTeam.Terrorist),
            new PromptContext(
                [new ChatMemoryLine("abc", false, "这把谁指挥", DateTimeOffset.UtcNow)],
                [],
                [],
                config.Global.TokenBudget,
                config.Global.Output));

        var content = string.Join("\n", prompt.Select(message => message.Content));

        Assert.Contains("当前实际发言 bot：Mika", content);
        Assert.Contains("[玩家 abc] 这把谁指挥", content);
        Assert.Contains("不要声称自己是某个职业选手", content);
        Assert.Contains("不可信输入", content);
        Assert.DoesNotContain("队里嘴硬但可靠", content);
    }

    [Fact]
    public void PromptBuilder_IncludesPerBotShortMemoryWhenProvided()
    {
        var config = CreateSharedConfig();
        config.Global.Memory.PerBotShortMemoryEnabled = true;
        config.Global.Memory.PerBotShortMemoryMaxItems = 3;
        config.Global.Memory.PerBotShortMemoryMaxChars = 240;
        config.Normalize();

        var persona = config.GetSharedPersona();
        var memory = new ConversationMemory();
        var now = DateTimeOffset.UtcNow;
        memory.UpdateBotMemoryAfterReply(
            persona.Id,
            new ChatEvent(1, "uni", true, false, "刚才怎么打", now, ChatChannel.All, CsTeam.Terrorist),
            "慢慢清点，别急着送",
            config.Global.Memory);

        var messages = new PromptBuilder().Build(
            persona,
            new ResolvedBot(persona.Id, persona.DisplayName, 4, true, "Mika", CsTeam.Terrorist),
            new ChatEvent(1, "uni", true, false, "再说下", now, ChatChannel.All, CsTeam.Terrorist),
            new PromptContext(
                [],
                memory.GetBotMemory(persona.Id, config.Global.Memory.PerBotShortMemoryMaxItems, config.Global.Memory.PerBotShortMemoryMaxChars),
                [],
                config.Global.TokenBudget,
                config.Global.Output));
        var content = string.Join("\n", messages.Select(message => message.Content));

        Assert.Contains("你自己的短记忆", content);
        Assert.Contains("慢慢清点", content);
    }

    [Fact]
    public void Selector_AllowsRepliesAgainAfterRoundMemoryReset()
    {
        var config = CreateSharedConfig();
        config.Global.Selection.RandomJitter = 0;
        config.Global.Selection.NoMentionBaseReplyChance = 1;
        config.Global.RateLimit.GlobalCooldownMs = 0;
        config.Global.RateLimit.PerPlayerCooldownMs = 0;
        config.Global.RateLimit.PerBotCooldownMs = 0;
        config.Global.RateLimit.MaxRepliesPerRound = 1;
        config.Global.RateLimit.SuppressAfterConsecutiveBotReplies = 0;
        config.Personas[0].Selection.CooldownMs = 0;
        config.Normalize();

        var registry = new PersonaRegistry([config.GetSharedPersona()]);
        var memory = new ConversationMemory();
        var now = DateTimeOffset.UtcNow;
        memory.AddBotMessage("shared-bot", 3, "Rex", "上一条", now - TimeSpan.FromSeconds(5), config.Global.Memory);
        var ev = new ChatEvent(1, "uni", true, false, "这回合还能聊吗", now, ChatChannel.All, CsTeam.Terrorist);
        var intent = new MessageIntentParser().Parse(ev.Text, config.Global.Trigger, registry, memory.GetState(now));
        var activeBots = new[]
        {
            new ResolvedBot("shared-bot", "bot", 3, true, "Rex", CsTeam.Terrorist)
        };

        var blocked = new BotReplySelector(registry, config.Global, new FixedRandomSource(0, 0))
            .Select(ev, new SelectionContext(intent, activeBots, memory.GetState(now), now, "1"));
        memory.ResetRound();
        var allowed = new BotReplySelector(registry, config.Global, new FixedRandomSource(0, 0))
            .Select(ev, new SelectionContext(intent, activeBots, memory.GetState(now), now, "1"));

        Assert.False(blocked.ShouldReply);
        Assert.Equal("round reply budget reached", blocked.Reason);
        Assert.True(allowed.ShouldReply);
    }

    [Fact]
    public void BotIdentityResolver_MarksBotHiderManagedSlotAsKnownBot()
    {
        var config = CreateSharedConfig();
        var registry = new PersonaRegistry([config.GetSharedPersona()]);
        var resolver = new BotIdentityResolver();

        resolver.RefreshBindings(
            registry,
            config.BotHider,
            [
                new PlayerSnapshot(3, "Rex", false, true, true, "Rex", CsTeam.Terrorist),
                new PlayerSnapshot(10, "uni", false, true, false, null, CsTeam.CounterTerrorist)
            ]);

        Assert.True(resolver.IsKnownBotSlot(3));
        Assert.False(resolver.IsKnownBotSlot(10));
    }

    [Fact]
    public void OwnBotOutputCache_MatchesRecentOutputEvenIfSlotLooksHuman()
    {
        var cache = new OwnBotOutputCache();
        var now = DateTimeOffset.UtcNow;
        cache.Add(3, "Rex", "别慌，这波能打", now, TimeSpan.FromSeconds(15));

        Assert.True(cache.Matches(10, "Rex", "别慌，这波能打", now + TimeSpan.FromSeconds(1)));
    }

    [Fact]
    public void ReplyPostProcessor_StripsAiPrefixAndTruncates()
    {
        var persona = CreateSharedConfig().GetSharedPersona();
        var output = new OutputConfig
        {
            MaxReplyChars = 12,
            ForbiddenPrefixes = ["作为一个AI"],
            StripQuotes = true
        };
        output.Normalize();

        var processed = new ReplyPostProcessor().Process("作为一个AI：这波有点意思，但是先别急", persona, output);

        Assert.True(processed.ShouldSend);
        Assert.DoesNotContain("AI", processed.Text);
        Assert.True(processed.Text.Length <= 12);
    }

    private static BotChatConfig CreateSharedConfig()
    {
        var config = new BotChatConfig
        {
            Global = new GlobalBotConfig
            {
                Trigger = new TriggerConfig
                {
                    RequireMention = false
                },
                Memory = new MemoryConfig
                {
                    PerBotShortMemoryEnabled = false,
                    PerBotShortMemoryMaxItems = 0,
                    PerBotShortMemoryMaxChars = 0
                }
            },
            Personas =
            [
                new BotPersonaConfig
                {
                    Id = "shared-bot",
                    DisplayName = "bot",
                    Aliases = ["@bot", "bot", "机器人"],
                    Binding = new PersonaBindingConfig
                    {
                        Type = "pool",
                        AllowBotHiderManagedSlot = true
                    },
                    Style = new PersonaStyleConfig
                    {
                        Tone = "普通游戏内 bot，短句、自然、可以接闲聊，不要装成真人选手",
                        Avoid = ["复杂人设", "长期记忆口吻", "AI 助手口吻"],
                        MaxSentences = 2
                    },
                    Character = new PersonaCharacterConfig
                    {
                        Role = "随机被选中接一句话的普通 CS2 bot",
                        Traits = ["自然", "短句"],
                        Interests = ["最近几句聊天", "普通闲聊"],
                        Boundaries = ["不刷屏", "不假装有复杂身份"]
                    },
                    Selection = new PersonaSelectionConfig
                    {
                        BaseReplyChance = 0.16,
                        MentionReplyChance = 0.95,
                        QuestionReplyChance = 0.34,
                        CooldownMs = 18000,
                        MaxRepliesPerMinute = 5,
                        ActivityWeight = 1
                    }
                }
            ]
        };

        config.Normalize();
        return config;
    }
}
