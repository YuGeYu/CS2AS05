using CounterStrikeSharp.API;
using CounterStrikeSharp.API.Core;
using CounterStrikeSharp.API.Core.Attributes;
using CounterStrikeSharp.API.Core.Attributes.Registration;
using CounterStrikeSharp.API.Modules.Commands;
using CounterStrikeSharp.API.Modules.Extensions;
using CounterStrikeSharp.API.Modules.Utils;
using Microsoft.Extensions.Logging;
using System.Globalization;
using System.Diagnostics;

namespace CS2BotLlmChat;

[MinimumApiVersion(80)]
public sealed class CS2BotLlmChatPlugin : BasePlugin, IPluginConfig<BotChatConfig>
{
    private readonly HttpClient _httpClient = new();
    private readonly OpenAiChatClient _llmClient;
    private readonly MessageIntentParser _intentParser = new();
    private readonly ConversationMemory _memory = new();
    private readonly BotIdentityResolver _identityResolver = new();
    private readonly PromptBuilder _promptBuilder = new();
    private readonly ReplyPostProcessor _postProcessor = new();
    private readonly OwnBotOutputCache _ownOutputs = new();
    private readonly IRandomSource _random = new SystemRandomSource();
    private CancellationTokenSource _shutdown = new();
    private PersonaRegistry _personaRegistry = new([]);
    private BotHiderCapability? _botHider;
    private int _pendingRequests;
    private int _requestsThisRound;
    private int _requestsThisMatch;

    public CS2BotLlmChatPlugin()
    {
        _llmClient = new OpenAiChatClient(_httpClient);
        Config.Normalize();
        _personaRegistry = new PersonaRegistry([Config.GetSharedPersona()]);
    }

    public override string ModuleName => "CS2 Bot LLM Chat";

    public override string ModuleVersion => "0.2.0";

    public override string ModuleAuthor => "unicbm";

    public override string ModuleDescription => "Lightweight shared LLM chat for CS2 bots.";

    public BotChatConfig Config { get; set; } = new();

    public void OnConfigParsed(BotChatConfig config)
    {
        TryApplyConfig(config, "startup");
    }

    public override void Load(bool hotReload)
    {
        RegisterListener<Listeners.OnPlayerChat>(OnPlayerChat);
        RegisterListener<Listeners.OnMapStart>(_ => ResetMatchBudget());
        Logger.LogInformation(
            "{ModuleName} loaded. Shared bot profile: {Profile}",
            ModuleName,
            Config.GetSharedPersona().DisplayName);
    }

    private void ResetMatchBudget()
    {
        Interlocked.Exchange(ref _requestsThisMatch, 0);
        Interlocked.Exchange(ref _requestsThisRound, 0);
    }

    public override void OnAllPluginsLoaded(bool hotReload)
    {
        _botHider = BotHiderCapability.TryResolve(Logger);
    }

    public override void Unload(bool hotReload)
    {
        _shutdown.Cancel();
        _shutdown.Dispose();
        _httpClient.Dispose();
    }

    [GameEventHandler]
    public HookResult OnRoundStart(EventRoundStart @event, GameEventInfo info)
    {
        _memory.ResetRound();
        Interlocked.Exchange(ref _requestsThisRound, 0);
        return HookResult.Continue;
    }

    [GameEventHandler]
    public HookResult OnRoundMvp(EventRoundMvp @event, GameEventInfo info)
    {
        var player = @event.Userid;
        if (player is { IsValid: true, IsBot: true } && Random.Shared.NextDouble() < 0.40)
            Server.NextFrame(() => _ = RunEventChatAsync(player, "你刚刚拿到MVP。用一句简短、带竞技嘲讽但不过分的中文嘲讽对面。", Config.CloneForRuntime(), _shutdown.Token));
        return HookResult.Continue;
    }

    [GameEventHandler]
    public HookResult OnPlayerDeath(EventPlayerDeath @event, GameEventInfo info)
    {
        var victim = @event.Userid;
        var attacker = @event.Attacker;
        if (victim is { IsValid: true, IsBot: true } && Random.Shared.NextDouble() < 0.10)
            Server.NextFrame(() => _ = RunEventChatAsync(victim, "你刚刚被击杀。用一句简短、带自嘲或嘲讽对面的中文回应。", Config.CloneForRuntime(), _shutdown.Token));
        if (attacker is { IsValid: true, IsBot: true } && victim is { IsValid: true, IsBot: false } && Random.Shared.NextDouble() < 0.20)
            Server.NextFrame(() => _ = RunEventChatAsync(attacker, "你刚刚击杀了一名真人玩家。用一句简短的中文嘲讽对面。", Config.CloneForRuntime(), _shutdown.Token));
        return HookResult.Continue;
    }

    private Task RunEventChatAsync(CCSPlayerController bot, string situation, BotChatConfig config, CancellationToken cancellationToken)
        => RunEventChatAsync(new ResolvedBot(config.GetSharedPersona().Id, config.GetSharedPersona().DisplayName, bot.Slot, false, bot.PlayerName, bot.Team), situation, config, cancellationToken);

    private async Task RunEventChatAsync(ResolvedBot bot, string situation, BotChatConfig config, CancellationToken cancellationToken)
    {
        try
        {
            if (!TryAcquireRequest(config)) return;
            var context = string.Join("\n", _memory.GetRecentContext(config.Global.Memory.RecentContextMessages, config.Global.Memory.RecentContextMaxChars));
            var team = bot.Team is CsTeam.CounterTerrorist ? "CT" : bot.Team is CsTeam.Terrorist ? "T" : "未知阵营";
            var reply = await _llmClient.CreateReplyAsync(config.Global.Provider, [
                new OpenAiChatMessage("system", $"你是CS2中的BOT，阵营是{team}。判断是否适合公开聊天：不适合只输出 SILENT；适合则只输出一句中文内容，不要前缀、引号或解释。"),
                new OpenAiChatMessage("user", $"公开聊天上下文:{context}\n战场情况:{situation}")], cancellationToken).ConfigureAwait(false);
            if (reply.Trim().Equals("SILENT", StringComparison.OrdinalIgnoreCase)) return;
            var text = _postProcessor.Process(reply, config.GetSharedPersona(), config.Global.Output);
            if (!text.ShouldSend) return;
            Server.NextFrame(() => SendProcessedReply(new ChatTarget(false, CsTeam.None), new ChatEvent(bot.Slot, bot.CurrentGameName ?? "BOT", false, true, situation, DateTimeOffset.UtcNow, ChatChannel.All, bot.Team), config.GetSharedPersona(), bot, text.Text, config));
        }
        catch { }
    }

    private void OnPlayerChat(CCSPlayerController player, string message, bool teamChat)
    {
        var config = Config.CloneForRuntime();
        var personaRegistry = new PersonaRegistry([config.GetSharedPersona()]);

        if (!config.Global.Enabled)
        {
            return;
        }

        var now = DateTimeOffset.UtcNow;
        RefreshBotIdentityBindings(config, personaRegistry);

        if (ShouldIgnoreAutomatedSpeaker(player, message, now, config))
        {
            return;
        }

        var target = new ChatTarget(teamChat, player.Team);
        var ev = new ChatEvent(
            player.Slot,
            player.PlayerName,
            IsHuman: !player.IsBot,
            IsKnownBot: player.IsBot,
            ChatReplyFormatter.Sanitize(message),
            now,
            teamChat ? ChatChannel.Team : ChatChannel.All,
            player.Team);

        var state = _memory.GetState(now);
        var intent = _intentParser.Parse(ev.Text, config.Global.Trigger, personaRegistry, state);
        if (intent.ShouldIgnore)
        {
            LogSelectionInfo("Ignoring chat from {PlayerName}: {Reason}", player.PlayerName, intent.IgnoreReason ?? "unknown");
            return;
        }

        var cleanEvent = ev with { Text = intent.CleanText };
        _memory.AddPlayerMessage(cleanEvent, config.Global.Memory);
        var sharedPersona = config.GetSharedPersona();
        var activeSpeakerBots = GetActiveSpeakerBots(target, sharedPersona, config);
        foreach (var bot in activeSpeakerBots.Take(1))
        {
            var situation = $"玩家/ BOT 发言人:{player.PlayerName}，阵营:{(player.Team is CsTeam.CounterTerrorist ? "CT" : player.Team is CsTeam.Terrorist ? "T" : "未知")}，频道:{(teamChat ? "队伍" : "公开")}，内容:{cleanEvent.Text}";
            _ = RunEventChatAsync(bot, situation, config, _shutdown.Token);
        }
    }

    [ConsoleCommand("css_llmbot_reload", "Reloads CS2 Bot LLM Chat config.")]
    public void OnReloadConfig(CCSPlayerController? player, CommandInfo command)
    {
        var previousConfig = Config.CloneForRuntime();
        try
        {
            Config.Reload();
        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Config reload failed.");
            Config = previousConfig;
            _personaRegistry = new PersonaRegistry([Config.GetSharedPersona()]);
            ReplyToCommand(player, command, "[LLMBot] Reload failed. Keeping previous config.");
            return;
        }

        if (!TryApplyConfig(Config, "reload"))
        {
            Config = previousConfig;
            _personaRegistry = new PersonaRegistry([Config.GetSharedPersona()]);
            RefreshBotIdentityBindings(Config, _personaRegistry);
            ReplyToCommand(player, command, "[LLMBot] Reload rejected. Keeping previous config.");
            return;
        }

        RefreshBotIdentityBindings(Config, _personaRegistry);
        ReplyToCommand(player, command, $"[LLMBot] Reloaded. SharedProfile={Config.GetSharedPersona().DisplayName}");
    }

    [ConsoleCommand("css_llmbot_test", "Sends a test message to the configured LLM.")]
    public void OnTestCommand(CCSPlayerController? player, CommandInfo command)
    {
        var message = command.ArgString.Trim();
        if (string.IsNullOrWhiteSpace(message))
        {
            ReplyToCommand(player, command, "[LLMBot] Usage: css_llmbot_test <message>");
            return;
        }

        ReplyToCommand(player, command, "[LLMBot] Sending test request...");
        _ = ReplyToCommandAsync(player, player?.PlayerName ?? "Server", message, Config.CloneForRuntime(), _shutdown.Token);
    }

    [ConsoleCommand("css_llmbot_saytest", "Tries to make any bot send a native chat message.")]
    public void OnSayTestCommand(CCSPlayerController? player, CommandInfo command)
    {
        var message = ChatReplyFormatter.Sanitize(command.ArgString);
        if (string.IsNullOrWhiteSpace(message))
        {
            ReplyToCommand(player, command, "[LLMBot] Usage: css_llmbot_saytest <message>");
            return;
        }

        RefreshBotIdentityBindings(Config, _personaRegistry);
        var result = TryNativeBotSay(new ChatTarget(false, CsTeam.None), message, Config.Global.Output.MaxReplyChars);
        if (result.Sent)
        {
            ReplyToCommand(player, command, $"[LLMBot] Called {result.Method} on bot {result.BotName}. If no chat appeared, CS2 ignored the command silently.");
            return;
        }

        ReplyToCommand(player, command, $"[LLMBot] Bot native say failed: {result.Error}");
    }

    [ConsoleCommand("css_llmbot_bhstatus", "Shows BotHider compatibility state.")]
    public void OnBotHiderStatusCommand(CCSPlayerController? player, CommandInfo command)
    {
        if (_botHider is null)
        {
            ReplyToCommand(player, command, "[LLMBot] BotHider capability: not available.");
            return;
        }

        var slots = _botHider.GetManagedSlots();
        ReplyToCommand(player, command, $"[LLMBot] BotHider capability: available; managed slots={slots.Length}.");
        foreach (var slot in slots.Take(8))
        {
            ReplyToCommand(player, command, $"[LLMBot] slot={slot} name='{_botHider.GetPersonaName(slot)}'");
        }
    }

    [ConsoleCommand("css_llmbot_diag", "Shows CS2 Bot LLM Chat runtime diagnostics.")]
    public void OnDiagCommand(CCSPlayerController? player, CommandInfo command)
    {
        RefreshBotIdentityBindings(Config, _personaRegistry);
        var sharedPersona = Config.GetSharedPersona();
        var activeAll = GetActiveSpeakerBots(new ChatTarget(false, CsTeam.None), sharedPersona, Config);
        var state = _memory.GetState(DateTimeOffset.UtcNow);

        ReplyToCommand(player, command, $"[LLMBot] pending={_pendingRequests}, activeBots={activeAll.Count}, profile={sharedPersona.Id}/{sharedPersona.DisplayName}");
        ReplyToCommand(player, command, $"[LLMBot] cooldowns global={Config.Global.RateLimit.GlobalCooldownMs}ms player={Config.Global.RateLimit.PerPlayerCooldownMs}ms bot={Config.Global.RateLimit.PerBotCooldownMs}ms persona={sharedPersona.Selection.CooldownMs}ms");
        ReplyToCommand(player, command, $"[LLMBot] trigger requireMention={Config.Global.Trigger.RequireMention}, noMentionChance={Config.Global.Selection.NoMentionBaseReplyChance:0.00}, maxPerMinute={Config.Global.RateLimit.MaxRepliesPerMinute}, repliesLastMinute={state.RepliesLastMinute}");
        foreach (var bot in activeAll.Take(8))
        {
            ReplyToCommand(player, command, $"[LLMBot] bot slot={bot.Slot} name='{bot.CurrentGameName}' team={bot.Team} botHider={bot.IsBotHiderManaged}");
        }
    }

    private async Task ReplyToChatAsync(
        ChatTarget target,
        ChatEvent ev,
        BotReplySelection selection,
        BotChatConfig config,
        CancellationToken cancellationToken)
    {
        try
        {
            var persona = config.GetSharedPersona();
            var bot = selection.Bot;
            if (bot is null)
            {
                return;
            }

            var messages = BuildPromptMessages(persona, bot, ev, config);
            if (config.Global.Debug.LogPromptStats)
            {
                Logger.LogDebug(
                    "Prompt for {PersonaId}: messages={MessageCount}, chars={CharCount}",
                    persona.Id,
                    messages.Count,
                    messages.Sum(message => message.Content.Length));
            }

            _memory.AddPlayerMessage(ev, config.Global.Memory);

            var stopwatch = Stopwatch.StartNew();
            var reply = await _llmClient
                .CreateReplyAsync(config.Global.Provider, messages, cancellationToken)
                .ConfigureAwait(false);
            stopwatch.Stop();

            if (config.Global.Debug.LogLlmLatency)
            {
                Logger.LogInformation("LLM reply for {PersonaId} took {ElapsedMs}ms.", persona.Id, stopwatch.ElapsedMilliseconds);
            }

            var processed = _postProcessor.Process(reply, persona, config.Global.Output);
            if (!processed.ShouldSend)
            {
                Logger.LogDebug("Dropping LLM reply for {PersonaId}: {Reason}", persona.Id, processed.Reason);
                return;
            }

            if (config.Global.AntiLoop.MarkOwnBotMessages)
            {
                _ownOutputs.Add(
                    bot.Slot,
                    bot.DisplayName,
                    processed.Text,
                    DateTimeOffset.UtcNow,
                    TimeSpan.FromMilliseconds(config.Global.AntiLoop.OwnBotMessageTtlMs));
            }

            Server.NextFrame(() => SendProcessedReply(target, ev, persona, bot, processed.Text, config));
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
        }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "LLM chat reply failed.");
        }
        finally
        {
            Interlocked.Decrement(ref _pendingRequests);
        }
    }

    private async Task ReplyToCommandAsync(
        CCSPlayerController? player,
        string playerName,
        string userMessage,
        BotChatConfig config,
        CancellationToken cancellationToken)
    {
        try
        {
            if (!TryAcquireRequest(config))
            {
                Server.NextFrame(() => PrintCommandLine(player, "[LLMBot] 本场或本回合请求额度已用尽。"));
                return;
            }
            var persona = config.GetSharedPersona();
            var bot = new ResolvedBot(persona.Id, persona.DisplayName, -1, false, persona.DisplayName, CsTeam.None);
            var ev = new ChatEvent(-1, playerName, true, false, ChatReplyFormatter.Sanitize(userMessage), DateTimeOffset.UtcNow, ChatChannel.All, CsTeam.None);
            var messages = BuildPromptMessages(persona, bot, ev, config);
            var reply = await _llmClient.CreateReplyAsync(config.Global.Provider, messages, cancellationToken).ConfigureAwait(false);
            var processed = _postProcessor.Process(reply, persona, config.Global.Output);
            if (!processed.ShouldSend)
            {
                Server.NextFrame(() => PrintCommandLine(player, $"[LLMBot] Empty response from model {config.Global.Provider.Model}."));
                return;
            }

            Server.NextFrame(() => PrintCommandLine(player, $"{persona.DisplayName}: {processed.Text}"));
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
        }
        catch (Exception ex)
        {
            Logger.LogWarning(ex, "LLM test command failed.");
            Server.NextFrame(() => PrintCommandLine(player, $"[LLMBot] Request failed: {ex.Message}"));
        }
    }

    private bool TryAcquireRequest(BotChatConfig config)
    {
        var roundLimit = config.Global.RateLimit.MaxRepliesPerRound;
        var matchLimit = config.Global.RateLimit.MaxRepliesPerMatch;
        if (roundLimit > 0 && Volatile.Read(ref _requestsThisRound) >= roundLimit) return false;
        if (matchLimit > 0 && Volatile.Read(ref _requestsThisMatch) >= matchLimit) return false;
        if (roundLimit > 0 && Interlocked.Increment(ref _requestsThisRound) > roundLimit) { Interlocked.Decrement(ref _requestsThisRound); return false; }
        if (matchLimit > 0 && Interlocked.Increment(ref _requestsThisMatch) > matchLimit) { Interlocked.Decrement(ref _requestsThisMatch); Interlocked.Decrement(ref _requestsThisRound); return false; }
        return true;
    }

    private IReadOnlyList<OpenAiChatMessage> BuildPromptMessages(
        BotPersonaConfig persona,
        ResolvedBot bot,
        ChatEvent ev,
        BotChatConfig config)
    {
        var memoryConfig = config.Global.Memory;
        var recentLines = _memory.GetRecentContext(memoryConfig.RecentContextMessages, memoryConfig.RecentContextMaxChars);
        var botMemory = memoryConfig.PerBotShortMemoryEnabled
            ? _memory.GetBotMemory(persona.Id, memoryConfig.PerBotShortMemoryMaxItems, memoryConfig.PerBotShortMemoryMaxChars)
            : Array.Empty<MemoryItem>();
        var otherBotNames = Array.Empty<string>();

        return _promptBuilder.Build(
            persona,
            bot,
            ev,
            new PromptContext(
                recentLines,
                botMemory,
                otherBotNames,
                config.Global.TokenBudget,
                config.Global.Output));
    }

    private void SendProcessedReply(
        ChatTarget target,
        ChatEvent ev,
        BotPersonaConfig persona,
        ResolvedBot bot,
        string text,
        BotChatConfig config)
    {
        var sent = false;
        if (!config.Global.Debug.DryRunNoSay && config.Global.Output.PreferBotSay)
        {
            var result = TryNativeBotSay(target, bot, text, config.Global.Output.MaxReplyChars);
            sent = result.Sent;
            if (!sent)
            {
                Logger.LogDebug("Falling back to plugin chat print. Bot native say failed: {Error}", result.Error);
            }
        }

        if (!sent)
        {
            PrintChatLine(target, $"{bot.CurrentGameName ?? persona.DisplayName}: {text}");
        }

        var now = DateTimeOffset.UtcNow;
        _memory.AddBotMessage(persona.Id, bot.Slot, bot.CurrentGameName ?? persona.DisplayName, text, now, config.Global.Memory);
        if (config.Global.Memory.PerBotShortMemoryEnabled)
        {
            _memory.UpdateBotMemoryAfterReply(persona.Id, ev, text, config.Global.Memory);
        }
    }

    private static void PrintChatLine(ChatTarget target, string line)
    {
        if (!target.TeamOnly)
        {
            Server.PrintToChatAll(line);
            return;
        }

        foreach (var targetPlayer in Utilities.GetPlayers().Where(candidate => candidate.IsValid && candidate.Team == target.Team))
        {
            targetPlayer.PrintToChat(line);
        }
    }

    private static void PrintCommandLine(CCSPlayerController? player, string line)
    {
        if (player is { IsValid: true })
        {
            player.PrintToChat(line);
            return;
        }

        Server.PrintToConsole(line);
    }

    private static void ReplyToCommand(CCSPlayerController? player, CommandInfo command, string line)
    {
        if (player is null)
        {
            command.ReplyToCommand(line);
            return;
        }

        player.PrintToChat(line);
    }

    private bool TryApplyConfig(BotChatConfig config, string source)
    {
        var rawValidation = BotChatConfigValidator.ValidateRaw(config);
        foreach (var warning in rawValidation.Warnings)
        {
            Logger.LogWarning("Config warning: {Warning}", warning);
        }

        foreach (var error in rawValidation.Errors)
        {
            Logger.LogError("Config error: {Error}", error);
        }

        if (!rawValidation.IsValid)
        {
            Logger.LogError("Invalid {Source} config. Keeping previous runtime config.", source);
            return false;
        }

        config.Normalize();

        var validation = BotChatConfigValidator.Validate(config);
        foreach (var warning in validation.Warnings)
        {
            Logger.LogWarning("Config warning: {Warning}", warning);
        }

        foreach (var error in validation.Errors)
        {
            Logger.LogError("Config error: {Error}", error);
        }

        if (!validation.IsValid)
        {
            Logger.LogError("Invalid normalized {Source} config. Keeping previous runtime config.", source);
            return false;
        }

        Config = config;
        _personaRegistry = new PersonaRegistry([config.GetSharedPersona()]);

        if (config.Personas.Count(persona => persona.Enabled) > 1)
        {
            Logger.LogWarning("Multiple personas are configured, but current runtime uses only the first enabled persona as a shared bot profile.");
        }

        return true;
    }

    private void RefreshBotIdentityBindings(BotChatConfig config, PersonaRegistry personaRegistry)
    {
        var snapshots = Utilities.GetPlayers()
            .Where(player => player is not null)
            .Select(player =>
            {
                var isManaged = config.BotHider.Enabled && _botHider?.IsManagedBot(player) == true;
                var botHiderName = isManaged ? _botHider?.GetPersonaName(player.Slot) : null;
                return new PlayerSnapshot(
                    player.Slot,
                    player.PlayerName,
                    player.IsBot,
                    player.IsValid,
                    isManaged,
                    botHiderName,
                    player.Team);
            })
            .ToArray();

        _identityResolver.RefreshBindings(personaRegistry, config.BotHider, snapshots);

        if (config.Global.Debug.LogBotIdentityResolution)
        {
            Logger.LogInformation(
                "Resolved active persona bots: {Bindings}",
                string.Join(", ", _identityResolver.GetActivePersonaBots().Select(bot => $"{bot.PersonaId}@{bot.Slot}:{bot.CurrentGameName}")));
        }
    }

    private IReadOnlyList<ResolvedBot> GetActiveSpeakerBots(ChatTarget target, BotPersonaConfig sharedPersona, BotChatConfig config)
    {
        var candidates = new List<CCSPlayerController>();
        if (config.BotHider.Enabled && _botHider is not null)
        {
            candidates.AddRange(_botHider
                .GetManagedSlots()
                .Select(Utilities.GetPlayerFromSlot)
                .Where(player => player is { IsValid: true })
                .Cast<CCSPlayerController>());
        }

        candidates.AddRange(Utilities.GetPlayers()
            .Where(player => player is { IsValid: true, IsBot: true }));

        var deduped = candidates
            .GroupBy(player => player.Slot)
            .Select(group => group.First())
            .Where(player => player.Team is CsTeam.Terrorist or CsTeam.CounterTerrorist)
            .ToArray();

        if (target.TeamOnly)
        {
            var sameTeam = deduped.Where(player => player.Team == target.Team).ToArray();
            if (sameTeam.Length > 0)
            {
                deduped = sameTeam;
            }
        }

        return deduped
            .Select(player => new ResolvedBot(
                sharedPersona.Id,
                sharedPersona.DisplayName,
                player.Slot,
                config.BotHider.Enabled && _botHider?.IsManagedBot(player) == true,
                player.PlayerName,
                player.Team))
            .ToArray();
    }

    private bool ShouldIgnoreAutomatedSpeaker(CCSPlayerController player, string message, DateTimeOffset now, BotChatConfig config)
    {
        if (config.Global.AntiLoop.IgnoreMessagesMatchingRecentBotOutputs
            && _ownOutputs.Matches(player.Slot, player.PlayerName, message, now))
        {
            LogSelectionInfo("Ignoring own bot output from {PlayerName}.", player.PlayerName);
            return true;
        }

        var isKnownBotSlot = _identityResolver.IsKnownBotSlot(player.Slot)
            || (config.BotHider.Enabled && _botHider?.IsManagedBot(player) == true);

        if (config.Global.AntiLoop.IgnoreBotMessages && player.IsBot)
        {
            LogSelectionInfo("Ignoring native bot chat from {PlayerName} slot {Slot}.", player.PlayerName, player.Slot);
            return true;
        }

        if (config.Global.AntiLoop.IgnoreMessagesFromKnownBotSlots && isKnownBotSlot)
        {
            LogSelectionInfo("Ignoring known bot slot chat from {PlayerName} slot {Slot}.", player.PlayerName, player.Slot);
            return true;
        }

        var isKnownBotName = _identityResolver.IsKnownBotName(player.PlayerName);

        if (config.Global.AntiLoop.IgnoreBotMessages && isKnownBotName)
        {
            LogSelectionInfo("Ignoring known bot chat from {PlayerName} slot {Slot}.", player.PlayerName, player.Slot);
            return true;
        }

        return false;
    }

    private void LogSelectionInfo(string message, params object?[] args)
    {
        if (Config.Global.Debug.LogSelection)
        {
            Logger.LogInformation(message, args);
        }
    }

    private NativeBotSayResult TryNativeBotSay(ChatTarget target, string line, int maxChars)
    {
        var bot = FindReplyBot(target);
        if (bot is null)
        {
            return NativeBotSayResult.Failed("No valid bot found.");
        }

        var commandName = target.TeamOnly ? "say_team" : "say";
        return ExecuteBotSay(bot, commandName, line, maxChars);
    }

    private NativeBotSayResult TryNativeBotSay(ChatTarget target, ResolvedBot resolvedBot, string line, int maxChars)
    {
        if (target.TeamOnly && resolvedBot.Team != target.Team)
        {
            return NativeBotSayResult.Failed("Selected bot is not on the triggering team.");
        }

        var bot = Utilities.GetPlayerFromSlot(resolvedBot.Slot);
        if (bot is not { IsValid: true })
        {
            return NativeBotSayResult.Failed($"Slot {resolvedBot.Slot} is not a valid bot controller.");
        }

        var commandName = target.TeamOnly ? "say_team" : "say";
        return ExecuteBotSay(bot, commandName, line, maxChars);
    }

    private NativeBotSayResult ExecuteBotSay(CCSPlayerController bot, string commandName, string line, int maxChars)
    {
        var safeLine = BotCommandText.SanitizeForClientCommand(line, maxChars);
        if (safeLine.Length == 0)
        {
            return NativeBotSayResult.Failed("Sanitized bot chat line is empty.");
        }

        var command = $"{commandName} {QuoteConsoleArgument(safeLine)}";

        try
        {
            bot.ExecuteClientCommandFromServer(command);
            Logger.LogInformation("Called ExecuteClientCommandFromServer on bot {BotName}: {Command}", bot.PlayerName, commandName);
            return NativeBotSayResult.Succeeded(bot.PlayerName, "ExecuteClientCommandFromServer");
        }
        catch (Exception fromServerEx)
        {
            try
            {
                bot.ExecuteClientCommand(command);
                Logger.LogInformation("Called ExecuteClientCommand on bot {BotName}: {Command}", bot.PlayerName, commandName);
                return NativeBotSayResult.Succeeded(bot.PlayerName, "ExecuteClientCommand");
            }
            catch (Exception clientEx)
            {
                Logger.LogWarning(clientEx, "Bot native say failed after ExecuteClientCommandFromServer also failed: {Error}", fromServerEx.Message);
                return NativeBotSayResult.Failed(clientEx.Message);
            }
        }
    }

    private CCSPlayerController? FindReplyBot(ChatTarget target)
    {
        var botHiderBot = FindBotHiderReplyBot(target);
        if (botHiderBot is not null)
        {
            return botHiderBot;
        }

        var bots = Utilities.GetPlayers()
            .Where(candidate => candidate is { IsValid: true, IsBot: true })
            .ToArray();

        if (target.TeamOnly)
        {
            return bots.FirstOrDefault(candidate => candidate.Team == target.Team) ?? bots.FirstOrDefault();
        }

        return bots.FirstOrDefault(candidate => candidate.Team is CsTeam.Terrorist or CsTeam.CounterTerrorist)
            ?? bots.FirstOrDefault();
    }

    private CCSPlayerController? FindBotHiderReplyBot(ChatTarget target)
    {
        if (_botHider is null)
        {
            return null;
        }

        var managedBots = _botHider.GetManagedSlots()
            .Select(Utilities.GetPlayerFromSlot)
            .Where(player => player is { IsValid: true })
            .Cast<CCSPlayerController>()
            .ToArray();

        if (target.TeamOnly)
        {
            return managedBots.FirstOrDefault(candidate => candidate.Team == target.Team)
                ?? managedBots.FirstOrDefault();
        }

        return managedBots.FirstOrDefault(candidate => candidate.Team is CsTeam.Terrorist or CsTeam.CounterTerrorist)
            ?? managedBots.FirstOrDefault();
    }

    private static string QuoteConsoleArgument(string value)
    {
        return $"\"{value.Replace("\\", "\\\\").Replace("\"", "'")}\"";
    }

    private static string GetPlayerKey(CCSPlayerController player)
    {
        return player.UserId?.ToString(CultureInfo.InvariantCulture) ?? player.PlayerName;
    }

    private readonly record struct ChatTarget(bool TeamOnly, CsTeam Team);

    private readonly record struct NativeBotSayResult(bool Sent, string? BotName, string? Method, string Error)
    {
        public static NativeBotSayResult Succeeded(string botName, string method) => new(true, botName, method, string.Empty);

        public static NativeBotSayResult Failed(string error) => new(false, null, null, error);
    }
}
