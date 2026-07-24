using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

internal static class Program
{
    private static int _assertions;

    private static int Main(string[] args)
    {
        if (args.Length > 0)
        {
            if (args.Length > 2) throw new ArgumentException("Usage: dotnet run -c Release -- <log-path> [output-directory]");
            string output = args.Length == 2 ? args[1] : Path.GetDirectoryName(Path.GetFullPath(args[0]))!;
            return NadeAuditAnalyzer.Analyze(args[0], output);
        }

        OpeningPerBotLimit();
        BotAndTeamGaps();
        OpeningTeamAndTypeBudgets();
        NormalPlannedSmokeCap();
        ModeSpecificSmokeCapAndEmergencies();
        CandidatePriorityAndStableTieBreak();
        SmokeRemainsEligibleWithoutOffensiveCandidate();
        EffectiveRadiusBoundariesAndProbabilities();
        FlashFallbackProbabilities();
        EmergencyStillUsesHardLimit();
        HardLimitsAndMolotovAlias();
        FailedCreationRollsBackReservation();
        AuditCommitAndRoundTrip();
        AuditParserRejectsMalformedFields();
        AuditAnalyzerRequiresExplicitNormalMode();
        RoundResetClearsState();
        Console.WriteLine($"Nade pacing/audit policy: {_assertions} assertions passed.");
        return 0;
    }

    private static void OpeningPerBotLimit()
    {
        var policy = new NadePacingPolicy();
        Commit(policy, Begin(policy, 1, 2, "flash", ThrowReason.PlannedReplay, 101f, 100f, 5), 101f);
        Reject(policy.TryBegin(1, 2, "he", ThrowReason.PlannedReplay, 107f, 100f, 5), "same bot opening limit");
        Accept(policy.TryBegin(1, 2, "he", ThrowReason.PlannedReplay, 116f, 100f, 5), "opening expires at 15 seconds");
    }

    private static void BotAndTeamGaps()
    {
        var policy = new NadePacingPolicy();
        Commit(policy, Begin(policy, 1, 2, "flash", ThrowReason.PlannedReplay, 20f, 0f, 5), 20f);
        Reject(policy.TryBegin(1, 2, "he", ThrowReason.PlannedReplay, 24.999f, 0f, 5), "bot gap");
        Reject(policy.TryBegin(2, 2, "he", ThrowReason.PlannedReplay, 20.499f, 0f, 5), "team gap");
        Accept(policy.TryBegin(2, 2, "he", ThrowReason.PlannedReplay, 20.5f, 0f, 5), "exact team gap");
    }

    private static void OpeningTeamAndTypeBudgets()
    {
        var policy = new NadePacingPolicy();
        string[] types = { "smoke", "flash", "he", "molotov" };
        for (int i = 0; i < types.Length; i++)
            Commit(policy, Begin(policy, (uint)i + 1, 2, types[i], ThrowReason.PlannedReplay, 101f + i, 100f, 6), 101f + i);
        Reject(policy.TryBegin(5, 2, "smoke", ThrowReason.PlannedReplay, 105f, 100f, 6), "second opening smoke rejected");
        Reject(policy.TryBegin(5, 2, "flash", ThrowReason.PlannedReplay, 105f, 100f, 6), "fourth opening offensive throw rejected");

        var smokePolicy = new NadePacingPolicy();
        Commit(smokePolicy, Begin(smokePolicy, 1, 2, "smoke", ThrowReason.PlannedReplay, 101f, 100f, 5), 101f);
        Reject(smokePolicy.TryBegin(2, 2, "smoke", ThrowReason.PlannedReplay, 102f, 100f, 5), "opening smoke cap is one");
    }

    private static void CandidatePriorityAndStableTieBreak()
    {
        var candidates = new[] { (Id: "z", Type: "smoke"), (Id: "h", Type: "he"), (Id: "m", Type: "molotov"), (Id: "f", Type: "flash") };
        var selected = candidates.OrderByDescending(x => NadeDecisionPolicy.Score(x.Type, 2f)).ThenBy(x => x.Id, StringComparer.Ordinal).First();
        Equal("molotov", selected.Type, "offensive priority");
        var tied = new[] { (Id: "b", Type: "he"), (Id: "a", Type: "he") }
            .OrderByDescending(x => NadeDecisionPolicy.Score(x.Type, 2f)).ThenBy(x => x.Id, StringComparer.Ordinal).First();
        Equal("a", tied.Id, "ordinal tie break");
    }

    private static void SmokeRemainsEligibleWithoutOffensiveCandidate()
    {
        Equal(100, NadeDecisionPolicy.Score("smoke", 2f), "smoke eligible before eight seconds");
        Equal(100, NadeDecisionPolicy.Score("smoke", 10f), "smoke eligible after eight seconds");
    }

    private static void EffectiveRadiusBoundariesAndProbabilities()
    {
        True(NadeDecisionPolicy.IsEnemyInEffectiveRadius("he", 450f), "HE boundary");
        False(NadeDecisionPolicy.IsEnemyInEffectiveRadius("he", 451f), "HE outside");
        True(NadeDecisionPolicy.IsEnemyInEffectiveRadius("molotov", 450f), "molotov boundary");
        False(NadeDecisionPolicy.IsEnemyInEffectiveRadius("molotov", 451f), "molotov outside");
        Equal(1f, NadeDecisionPolicy.NoInfoProbability("normal", "he"), "normal HE probability");
        Equal(1f, NadeDecisionPolicy.NoInfoProbability("normal", "molotov"), "normal fire probability");
        Equal(1f, NadeDecisionPolicy.NoInfoProbability("more", "he"), "more HE probability");
        Equal(1f, NadeDecisionPolicy.NoInfoProbability("more", "molotov"), "more fire probability");
        Equal(1f, NadeDecisionPolicy.NoInfoProbability("max", "he"), "max probability");
        Equal(1f, NadeDecisionPolicy.NoInfoProbability("max", "molotov"), "max fire probability");
    }

    private static void FlashFallbackProbabilities()
    {
        Equal(1600f, NadeDecisionPolicy.FlashFallbackRadius, "fallback radius");
        Equal(0.45f, NadeDecisionPolicy.FlashFallbackProbability("normal"), "normal fallback");
        Equal(0.70f, NadeDecisionPolicy.FlashFallbackProbability("more"), "more fallback");
        Equal(1f, NadeDecisionPolicy.FlashFallbackProbability("max"), "max fallback");
        True(NadeDecisionPolicy.CanUseFlashFallback("normal", true, true, 1600f, 0.449), "fallback valid boundary");
        False(NadeDecisionPolicy.CanUseFlashFallback("normal", false, true, 100f, 0), "fallback requires team tag");
        False(NadeDecisionPolicy.CanUseFlashFallback("normal", true, false, 100f, 0), "fallback requires schedule");
        False(NadeDecisionPolicy.CanUseFlashFallback("normal", true, true, 1601f, 0), "fallback radius boundary");
        False(NadeDecisionPolicy.CanUseFlashFallback("normal", true, true, 100f, 0.45), "deterministic probability boundary");
    }

    private static void EmergencyStillUsesHardLimit()
    {
        var policy = new NadePacingPolicy();
        Commit(policy, Begin(policy, 1, 2, "flash", ThrowReason.PlannedReplay, 101f, 100f, 5), 101f);
        Commit(policy, Begin(policy, 1, 2, "flash", ThrowReason.DefuseCover, 101.1f, 100f, 5), 101.1f);
        Reject(policy.TryBegin(1, 2, "flash", ThrowReason.DefuseCover, 101.2f, 100f, 5), "flash hard limit");
        Commit(policy, Begin(policy, 1, 2, "smoke", ThrowReason.ExtinguishFire, 101.2f, 100f, 5), 101.2f);
        Reject(policy.TryBegin(1, 2, "smoke", ThrowReason.DefuseCover, 101.3f, 100f, 5), "smoke hard limit");
    }

    private static void HardLimitsAndMolotovAlias()
    {
        foreach (string type in new[] { "he", "smoke" })
        {
            var policy = new NadePacingPolicy();
            Commit(policy, Begin(policy, 1, 2, type, ThrowReason.DefuseCover, 1f, 0f, 5), 1f);
            Reject(policy.TryBegin(1, 2, type, ThrowReason.DefuseCover, 2f, 0f, 5), type + " hard limit");
        }
        var fire = new NadePacingPolicy();
        Commit(fire, Begin(fire, 1, 2, "molotov", ThrowReason.DefuseCover, 1f, 0f, 5), 1f);
        Reject(fire.TryBegin(1, 2, "incgrenade", ThrowReason.DefuseCover, 2f, 0f, 5), "shared fire bucket");
    }

    private static void FailedCreationRollsBackReservation()
    {
        var policy = new NadePacingPolicy();
        var failed = Begin(policy, 1, 2, "smoke", ThrowReason.PlannedReplay, 101f, 100f, 5);
        policy.Cancel(failed);
        Accept(policy.TryBegin(1, 2, "smoke", ThrowReason.PlannedReplay, 101f, 100f, 5), "cancel releases all pending state");
    }

    private static void NormalPlannedSmokeCap()
    {
        var opening = new NadePacingPolicy();
        Commit(opening, Begin(opening, 1, 2, "smoke", ThrowReason.PlannedReplay, 101f, 100f, 5), 101f);
        Reject(opening.TryBegin(2, 2, "smoke", ThrowReason.PlannedReplay, 116f, 100f, 5), "opening smoke consumes whole-round cap");
        var noOpeningSmoke = new NadePacingPolicy();
        Commit(noOpeningSmoke, Begin(noOpeningSmoke, 1, 2, "flash", ThrowReason.PlannedReplay, 101f, 100f, 5), 101f);
        Accept(noOpeningSmoke.TryBegin(2, 2, "smoke", ThrowReason.PlannedReplay, 116f, 100f, 5), "late first smoke remains available");

        var policy = new NadePacingPolicy();
        Commit(policy, Begin(policy, 1, 2, "smoke", ThrowReason.PlannedReplay, 20f, 0f, 5), 20f);
        Reject(policy.TryBegin(2, 2, "smoke", ThrowReason.PlannedReplay, 20.5f, 0f, 5), "normal team smoke cap");
        var pending = Begin(policy, 3, 3, "smoke", ThrowReason.PlannedReplay, 20f, 0f, 5);
        Reject(policy.TryBegin(4, 3, "smoke", ThrowReason.PlannedReplay, 20.5f, 0f, 5), "pending smoke cap");
        policy.Cancel(pending);
        Accept(policy.TryBegin(4, 3, "smoke", ThrowReason.PlannedReplay, 20.5f, 0f, 5), "cancel restores smoke cap");
    }

    private static void ModeSpecificSmokeCapAndEmergencies()
    {
        var more = new NadePacingPolicy();
        Commit(more, Begin(more, 1, 2, "smoke", ThrowReason.PlannedReplay, 20f, 0f, 5, "more"), 20f);
        Accept(more.TryBegin(2, 2, "smoke", ThrowReason.PlannedReplay, 20.5f, 0f, 5, "more"), "more bypasses normal balance cap");
        var max = new NadePacingPolicy();
        Commit(max, Begin(max, 1, 2, "smoke", ThrowReason.PlannedReplay, 20f, 0f, 5, "max"), 20f);
        Commit(max, Begin(max, 2, 2, "smoke", ThrowReason.PlannedReplay, 20.5f, 0f, 5, "max"), 20.5f);
        Accept(max.TryBegin(3, 2, "smoke", ThrowReason.PlannedReplay, 21f, 0f, 5, "max"), "max third smoke bypasses normal balance cap");
        foreach (ThrowReason reason in new[] { ThrowReason.DefuseCover, ThrowReason.ExtinguishFire, ThrowReason.PlantCover })
        {
            var exempt = new NadePacingPolicy();
            Commit(exempt, Begin(exempt, 1, 2, "smoke", reason, 1f, 0f, 5), 1f);
            Accept(exempt.TryBegin(2, 2, "smoke", ThrowReason.PlannedReplay, 1.5f, 0f, 5), reason + " does not consume normal planned smoke cap");
        }
        var special = new NadePacingPolicy();
        Commit(special, Begin(special, 1, 2, "smoke", ThrowReason.DefuseCover, 1f, 0f, 5), 1f);
        Reject(special.TryBegin(1, 2, "smoke", ThrowReason.DefuseCover, 2f, 0f, 5), "special smoke remains hard limited per bot");
    }

    private static void AuditCommitAndRoundTrip()
    {
        var policy = new NadePacingPolicy();
        var cancelled = Begin(policy, 9, 2, "he", ThrowReason.PlannedReplay, 1f, 0f, 5);
        policy.Cancel(cancelled);
        int emitted = 0;
        var first = Begin(policy, 1, 2, "flash", ThrowReason.PlannedReplay, 101f, 100f, 5);
        var state = policy.Commit(first, 101f); emitted++;
        Equal(-1f, state.BotGap, "first bot gap"); Equal(-1f, state.TeamGap, "first team gap"); Equal(1, emitted, "only commit emits");
        var record = new NadeAuditRecord(101f, 7, 1f, 2, 1, "flash", "PlannedReplay", 0, 1, "flash_direct",
            state.BotGap, state.TeamGap, state.BotOpening, 1, state.TeamOpening, state.OpeningTeamLimit,
            state.TeamOpeningSmoke, NadePacingPolicy.OpeningSmokeCap, state.BotFlash, state.BotSmoke, state.BotHE, state.BotMolotov);
        string text = record.Format();
        Equal(text, NadeAuditRecord.Parse(text).Format(), "invariant audit round trip");
    }

    private static void AuditParserRejectsMalformedFields()
    {
        string valid = new NadeAuditRecord(1, 1, 0, 2, 1, "he", "PlannedReplay", 0, 1, "he_effective", -1, -1, 1, 1, 1, 5, 0, NadePacingPolicy.OpeningSmokeCap, 0, 0, 1, 0).Format();
        Throws(() => NadeAuditRecord.Parse(valid + " unknown=1"), "unknown field");
        Throws(() => NadeAuditRecord.Parse(valid + " time=2.000"), "duplicate field");
        Throws(() => NadeAuditRecord.Parse(valid.Replace(" type=he", " type=bad", StringComparison.Ordinal)), "invalid type");
    }

    private static void RoundResetClearsState()
    {
        var policy = new NadePacingPolicy();
        Commit(policy, Begin(policy, 1, 2, "he", ThrowReason.PlannedReplay, 101f, 100f, 5), 101f);
        policy.Reset();
        Accept(policy.TryBegin(1, 2, "he", ThrowReason.PlannedReplay, 102f, 100f, 5), "reset");
    }

    private static void AuditAnalyzerRequiresExplicitNormalMode()
    {
        string root = Path.Combine(Path.GetTempPath(), "nade-audit-mode-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(root);
        try
        {
            string first = AuditSmoke(1f, 8, 2, 1);
            string second = AuditSmoke(2f, 8, 2, 2);
            string noMode = Path.Combine(root, "no-mode.log");
            File.WriteAllLines(noMode, new[] { first, second });
            Equal(0, NadeAuditAnalyzer.Analyze(noMode, Path.Combine(root, "no-mode-out")), "missing mode evidence is not normal");

            string normal = Path.Combine(root, "normal.log");
            File.WriteAllLines(normal, new[] { "bot_nades set to normal", first, second });
            Equal(2, NadeAuditAnalyzer.Analyze(normal, Path.Combine(root, "normal-out")), "explicit normal mode enforces team smoke cap");
            string report = File.ReadAllText(Path.Combine(root, "normal-out", "nade-audit-summary.txt"));
            True(report.Contains("normal planned smoke team > 1", StringComparison.Ordinal), "normal violation includes source rows");
        }
        finally
        {
            Directory.Delete(root, true);
        }
    }

    private static string AuditSmoke(float time, int round, int team, uint bot) =>
        new NadeAuditRecord(time, round, 20f, team, bot, "smoke", "PlannedReplay", 0, 0, "special",
            -1, -1, 0, 1, 0, 5, 0, NadePacingPolicy.OpeningSmokeCap, 0, 1, 0, 0).Format();

    private static ThrowReservation Begin(NadePacingPolicy p, uint bot, int team, string type, ThrowReason reason, float now, float freeze, int alive, string mode = "normal") =>
        p.TryBegin(bot, team, type, reason, now, freeze, alive, mode) ?? throw new InvalidOperationException($"Expected reservation for {type} at {now}.");
    private static PacingAuditSnapshot Commit(NadePacingPolicy p, ThrowReservation r, float now) => p.Commit(r, now);
    private static void Accept(ThrowReservation? value, string message) { _assertions++; if (value == null) throw new InvalidOperationException("Expected allow: " + message); }
    private static void Reject(ThrowReservation? value, string message) { _assertions++; if (value != null) throw new InvalidOperationException("Expected reject: " + message); }
    private static void True(bool value, string message) { _assertions++; if (!value) throw new InvalidOperationException("Expected true: " + message); }
    private static void False(bool value, string message) { _assertions++; if (value) throw new InvalidOperationException("Expected false: " + message); }
    private static void Equal<T>(T expected, T actual, string message) where T : notnull { _assertions++; if (!EqualityComparer<T>.Default.Equals(expected, actual)) throw new InvalidOperationException($"{message}: expected {expected}, actual {actual}"); }
    private static void Throws(Action action, string message) { _assertions++; try { action(); } catch (FormatException) { return; } throw new InvalidOperationException("Expected format error: " + message); }
}
