using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;

internal sealed record NadeAuditRecord(
    float Time, int Round, float FreezeElapsed, int Team, uint Bot, string Type, string Reason,
    int Emergency, int Opening, string Decision, float BotGap, float TeamGap,
    int BotOpening, int BotOpeningLimit, int TeamOpening, int TeamOpeningLimit,
    int TeamOpeningSmoke, int TeamOpeningSmokeLimit,
    int BotFlash, int BotSmoke, int BotHE, int BotMolotov)
{
    internal const string Prefix = "[NadeAudit] ";
    private static readonly HashSet<string> RequiredKeys = new(StringComparer.Ordinal)
    {
        "event", "time", "round", "freeze_elapsed", "team", "bot", "type", "reason",
        "emergency", "opening", "decision", "bot_gap", "team_gap", "bot_opening",
        "team_opening", "team_opening_smoke", "bot_flash", "bot_smoke", "bot_he", "bot_molotov",
    };

    internal string Format() => string.Format(CultureInfo.InvariantCulture,
        Prefix + "event=throw time={0:F3} round={1} freeze_elapsed={2:F3} team={3} bot={4} type={5} reason={6} emergency={7} opening={8} decision={9} bot_gap={10:F3} team_gap={11:F3} bot_opening={12}/{13} team_opening={14}/{15} team_opening_smoke={16}/{17} bot_flash={18} bot_smoke={19} bot_he={20} bot_molotov={21}",
        Time, Round, FreezeElapsed, Team, Bot, Type, Reason, Emergency, Opening, Decision, BotGap, TeamGap,
        BotOpening, BotOpeningLimit, TeamOpening, TeamOpeningLimit, TeamOpeningSmoke, TeamOpeningSmokeLimit,
        BotFlash, BotSmoke, BotHE, BotMolotov);

    internal static NadeAuditRecord Parse(string line)
    {
        int prefixAt = line.IndexOf(Prefix, StringComparison.Ordinal);
        if (prefixAt < 0) throw new FormatException("Missing [NadeAudit] prefix.");
        var fields = new Dictionary<string, string>(StringComparer.Ordinal);
        foreach (string token in line[(prefixAt + Prefix.Length)..].Split(' ', StringSplitOptions.RemoveEmptyEntries))
        {
            int equals = token.IndexOf('=');
            if (equals <= 0 || equals == token.Length - 1) throw new FormatException($"Malformed token '{token}'.");
            string key = token[..equals];
            if (!RequiredKeys.Contains(key)) throw new FormatException($"Unknown field '{key}'.");
            if (!fields.TryAdd(key, token[(equals + 1)..])) throw new FormatException($"Duplicate field '{key}'.");
        }
        foreach (string key in RequiredKeys)
            if (!fields.ContainsKey(key)) throw new FormatException($"Missing field '{key}'.");
        if (fields["event"] != "throw") throw new FormatException("event must be throw.");
        string type = fields["type"];
        if (type is not ("flash" or "smoke" or "he" or "molotov")) throw new FormatException($"Invalid type '{type}'.");
        var botOpening = ParseRatio(fields["bot_opening"], "bot_opening");
        var teamOpening = ParseRatio(fields["team_opening"], "team_opening");
        var smokeOpening = ParseRatio(fields["team_opening_smoke"], "team_opening_smoke");
        return new NadeAuditRecord(
            Float(fields, "time"), Int(fields, "round"), Float(fields, "freeze_elapsed"), Int(fields, "team"), UInt(fields, "bot"),
            type, fields["reason"], Bit(fields, "emergency"), Bit(fields, "opening"), fields["decision"],
            Float(fields, "bot_gap"), Float(fields, "team_gap"), botOpening.Value, botOpening.Limit,
            teamOpening.Value, teamOpening.Limit, smokeOpening.Value, smokeOpening.Limit,
            Int(fields, "bot_flash"), Int(fields, "bot_smoke"), Int(fields, "bot_he"), Int(fields, "bot_molotov"));
    }

    private static (int Value, int Limit) ParseRatio(string value, string key)
    {
        string[] parts = value.Split('/');
        if (parts.Length != 2 || !int.TryParse(parts[0], NumberStyles.None, CultureInfo.InvariantCulture, out int current)
            || !int.TryParse(parts[1], NumberStyles.None, CultureInfo.InvariantCulture, out int limit))
            throw new FormatException($"Invalid ratio field '{key}'.");
        return (current, limit);
    }

    private static float Float(Dictionary<string, string> f, string key) =>
        float.TryParse(f[key], NumberStyles.Float, CultureInfo.InvariantCulture, out float value) && float.IsFinite(value)
            ? value : throw new FormatException($"Invalid float field '{key}'.");
    private static int Int(Dictionary<string, string> f, string key) =>
        int.TryParse(f[key], NumberStyles.Integer, CultureInfo.InvariantCulture, out int value)
            ? value : throw new FormatException($"Invalid integer field '{key}'.");
    private static uint UInt(Dictionary<string, string> f, string key) =>
        uint.TryParse(f[key], NumberStyles.None, CultureInfo.InvariantCulture, out uint value)
            ? value : throw new FormatException($"Invalid unsigned field '{key}'.");
    private static int Bit(Dictionary<string, string> f, string key)
    {
        int value = Int(f, key);
        return value is 0 or 1 ? value : throw new FormatException($"Field '{key}' must be 0 or 1.");
    }
}

internal static class NadeAuditAnalyzer
{
    internal static int Analyze(string inputPath, string outputDirectory)
    {
        Directory.CreateDirectory(outputDirectory);
        var records = new List<(int Line, string Raw, NadeAuditRecord Record)>();
        var modeByRound = new Dictionary<int, string>();
        string? currentMode = null;
        int lineNumber = 0;
        foreach (string line in File.ReadLines(inputPath))
        {
            lineNumber++;
            string lower = line.ToLowerInvariant();
            foreach (string mode in new[] { "normal", "more", "max" })
                if (lower.Contains($"bot_nades set to {mode}", StringComparison.Ordinal)) currentMode = mode;
            if (!line.Contains(NadeAuditRecord.Prefix, StringComparison.Ordinal)) continue;
            try
            {
                var record = NadeAuditRecord.Parse(line);
                records.Add((lineNumber, line, record));
                if (currentMode != null)
                {
                    if (!modeByRound.TryGetValue(record.Round, out string? existing)) modeByRound[record.Round] = currentMode;
                    else if (existing != currentMode) modeByRound[record.Round] = "mixed";
                }
            }
            catch (Exception ex) { throw new FormatException($"Line {lineNumber}: {ex.Message}\n{line}", ex); }
        }
        if (records.Count == 0) throw new FormatException("No [NadeAudit] records found.");

        var violations = new List<string>();
        foreach (var group in records.Where(x => x.Record.Emergency == 0).GroupBy(x => (x.Record.Round, x.Record.Bot)))
            CheckGaps(group.OrderBy(x => x.Record.Time).ToList(), 5f, "bot", violations);
        foreach (var group in records.Where(x => x.Record.Emergency == 0).GroupBy(x => (x.Record.Round, x.Record.Team)))
            CheckGaps(group.OrderBy(x => x.Record.Time).ToList(), 0.5f, "team", violations);

        var opening = records.Where(x => x.Record.Emergency == 0 && x.Record.FreezeElapsed >= 0f && x.Record.FreezeElapsed < 15f).ToList();
        foreach (var g in opening.GroupBy(x => (x.Record.Round, x.Record.Bot)).Where(g => g.Count() > 1)) AddGroupViolation("opening bot > 1", g, violations);
        foreach (var g in opening.GroupBy(x => (x.Record.Round, x.Record.Team)).Where(g => g.Count() > g.Min(x => x.Record.TeamOpeningLimit))) AddGroupViolation("opening team exceeds limit", g, violations);
        foreach (var g in opening.Where(x => x.Record.Type == "smoke").GroupBy(x => (x.Record.Round, x.Record.Team)).Where(g => g.Count() > g.Min(x => x.Record.TeamOpeningSmokeLimit))) AddGroupViolation("opening smoke exceeds limit", g, violations);

        foreach (var g in records.GroupBy(x => (x.Record.Round, x.Record.Bot)))
        {
            CheckHard(g, "flash", 2, violations); CheckHard(g, "smoke", 1, violations);
            CheckHard(g, "he", 1, violations); CheckHard(g, "molotov", 1, violations);
        }

        var normalPlannedSmokes = records.Where(x => modeByRound.TryGetValue(x.Record.Round, out string? mode)
            && mode == "normal" && x.Record.Reason == nameof(ThrowReason.PlannedReplay) && x.Record.Type == "smoke");
        foreach (var g in normalPlannedSmokes.GroupBy(x => (x.Record.Round, x.Record.Team)).Where(g => g.Count() > NadePacingPolicy.NormalPlannedSmokeCapPerTeam))
            AddGroupViolation("normal planned smoke team > 1", g, violations);

        string csvPath = Path.Combine(outputDirectory, "nade-audit-summary.csv");
        using (var csv = new StreamWriter(csvPath, false))
        {
            csv.WriteLine("category,key,count");
            foreach (var g in records.GroupBy(x => x.Record.Type).OrderBy(g => g.Key)) csv.WriteLine($"type,{g.Key},{g.Count()}");
            foreach (var g in records.GroupBy(x => x.Record.Round).OrderBy(g => g.Key)) csv.WriteLine($"round,{g.Key},{g.Count()}");
            foreach (var g in records.GroupBy(x => x.Record.Decision).OrderBy(g => g.Key)) csv.WriteLine($"decision,{g.Key},{g.Count()}");
        }
        string txtPath = Path.Combine(outputDirectory, "nade-audit-summary.txt");
        using (var txt = new StreamWriter(txtPath, false))
        {
            txt.WriteLine($"records={records.Count}");
            foreach (var g in records.GroupBy(x => x.Record.Type).OrderBy(g => g.Key)) txt.WriteLine($"type.{g.Key}={g.Count()}");
            foreach (var g in records.GroupBy(x => x.Record.Round).OrderBy(g => g.Key)) txt.WriteLine($"round.{g.Key}={g.Count()}");
            foreach (var g in records.GroupBy(x => x.Record.Decision).OrderBy(g => g.Key)) txt.WriteLine($"decision.{g.Key}={g.Count()}");
            txt.WriteLine($"violations={violations.Count}");
            foreach (string violation in violations) txt.WriteLine(violation);
            txt.WriteLine(violations.Count == 0 ? "RESULT=PASS" : "RESULT=FAIL");
        }
        return violations.Count == 0 ? 0 : 2;
    }

    private static void CheckGaps(List<(int Line, string Raw, NadeAuditRecord Record)> rows, float minimum, string kind, List<string> violations)
    {
        for (int i = 1; i < rows.Count; i++)
            if (rows[i].Record.Time - rows[i - 1].Record.Time + 0.0001f < minimum)
                violations.Add($"{kind}_gap line={rows[i].Line} raw={rows[i].Raw}");
    }
    private static void CheckHard(IEnumerable<(int Line, string Raw, NadeAuditRecord Record)> rows, string type, int limit, List<string> violations)
    {
        var matching = rows.Where(x => x.Record.Type == type).ToList();
        if (matching.Count > limit) AddGroupViolation($"hard {type} > {limit}", matching, violations);
    }
    private static void AddGroupViolation(string name, IEnumerable<(int Line, string Raw, NadeAuditRecord Record)> rows, List<string> violations)
    {
        foreach (var row in rows) violations.Add($"{name} line={row.Line} raw={row.Raw}");
    }
}
