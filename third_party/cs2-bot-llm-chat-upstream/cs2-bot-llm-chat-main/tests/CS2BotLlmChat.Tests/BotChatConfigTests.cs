using CS2BotLlmChat;
using Xunit;

namespace CS2BotLlmChat.Tests;

public sealed class BotChatConfigTests
{
    [Fact]
    public void Normalize_DefaultsBlankModelToDeepSeekChat()
    {
        var config = new BotChatConfig
        {
            Provider = new LlmProviderConfig
            {
                Model = " "
            }
        };

        config.Normalize();

        Assert.Equal("deepseek-chat", config.Provider.Model);
    }

    [Fact]
    public void ValidateRaw_ReportsDuplicatesBeforeNormalizeDropsThem()
    {
        var config = new BotChatConfig
        {
            Personas =
            [
                new BotPersonaConfig { Id = "shared-bot", DisplayName = "bot" },
                new BotPersonaConfig { Id = " SHARED-BOT ", DisplayName = "bot two" }
            ],
            BotHider = new BotHiderIntegrationConfig
            {
                ManagedSlots =
                [
                    new BotHiderManagedSlotConfig { Slot = 3, PersonaId = "shared-bot" },
                    new BotHiderManagedSlotConfig { Slot = 3, PersonaId = "other-bot" }
                ]
            }
        };

        var validation = BotChatConfigValidator.ValidateRaw(config);

        Assert.False(validation.IsValid);
        Assert.Contains(validation.Errors, error => error.Contains("Duplicate persona id", StringComparison.Ordinal));
        Assert.Contains(validation.Errors, error => error.Contains("Duplicate BotHider managed slot", StringComparison.Ordinal));
    }

    [Fact]
    public void CloneForRuntime_ReturnsIndependentNormalizedCopy()
    {
        var config = new BotChatConfig
        {
            Global = new GlobalBotConfig
            {
                Provider = new LlmProviderConfig
                {
                    Model = "test-model"
                }
            },
            Personas =
            [
                new BotPersonaConfig
                {
                    Id = "shared-bot",
                    DisplayName = "bot",
                    Aliases = ["@bot"]
                }
            ]
        };
        config.Normalize();

        var clone = config.CloneForRuntime();
        config.Global.Provider.Model = "changed-model";
        config.Personas[0].DisplayName = "changed";

        Assert.Equal("test-model", clone.Global.Provider.Model);
        Assert.Equal("bot", clone.GetSharedPersona().DisplayName);
        Assert.NotSame(config.Global.Provider, clone.Global.Provider);
        Assert.NotSame(config.Personas[0], clone.Personas[0]);
    }
}
