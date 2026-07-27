using System;
using System.Collections.Generic;

internal static class NadeDecisionPolicy
{
    internal const float EffectiveHERadius = 450f;
    internal const float EffectiveMolotovRadius = 450f;
    internal const float FlashFallbackRadius = 1600f;
    internal const float OpeningOffensivePrioritySeconds = 8f;

    internal static int Score(string type, float freezeElapsed) => type switch
    {
        "molotov" => 400,
        "he" => 300,
        "flash" => 200,
        "smoke" => 100,
        _ => 0,
    } + (freezeElapsed >= 0f && freezeElapsed < OpeningOffensivePrioritySeconds && type != "smoke" ? 10 : 0);

    internal static bool IsEnemyInEffectiveRadius(string type, float distance) =>
        distance <= (type == "he" ? EffectiveHERadius : EffectiveMolotovRadius);

    internal static bool CanUseFlashFallback(string mode, bool hasTeamTag, bool inSchedule,
        float nearestEnemyDistance, double roll) => hasTeamTag && inSchedule
        && nearestEnemyDistance <= FlashFallbackRadius
        && roll < FlashFallbackProbability(mode);

    internal static float NoInfoProbability(string mode, string type) => mode switch
    {
        "normal" => 1f,
        "more" => 1f,
        _ => 1f,
    };

    internal static float FlashFallbackProbability(string mode) => mode switch
    {
        "normal" => 0.45f,
        "more" => 0.70f,
        _ => 1f,
    };
}

internal enum ThrowReason
{
    PlannedReplay,
    Retaliation,
    PlantCover,
    DefuseCover,
    ExtinguishFire,
}

internal sealed class ThrowReservation
{
    public uint BotIndex { get; }
    public int TeamNum { get; }
    public string GrenadeType { get; }
    public ThrowReason Reason { get; }
    public bool IsNonEmergency { get; }
    public bool IsOpening { get; }
    public int OpeningTeamLimit { get; }
    public bool UsesNormalPlannedSmokeCap { get; }

    public ThrowReservation(uint botIndex, int teamNum, string grenadeType,
        ThrowReason reason, bool isNonEmergency, bool isOpening, int openingTeamLimit,
        bool usesNormalPlannedSmokeCap)
    {
        BotIndex = botIndex;
        TeamNum = teamNum;
        GrenadeType = grenadeType;
        Reason = reason;
        IsNonEmergency = isNonEmergency;
        IsOpening = isOpening;
        OpeningTeamLimit = openingTeamLimit;
        UsesNormalPlannedSmokeCap = usesNormalPlannedSmokeCap;
    }
}

internal readonly record struct PacingAuditSnapshot(
    float BotGap, float TeamGap, int BotOpening, int TeamOpening, int TeamOpeningSmoke,
    int BotFlash, int BotSmoke, int BotHE, int BotMolotov, int OpeningTeamLimit);

internal sealed class NadePacingPolicy
{
    internal const float OpeningWindowSeconds = 15f;
    internal const int OpeningThrowsPerBot = 1;
    internal const int OpeningSmokeCap = 1;
    internal const int OpeningOffensiveCap = 3;
    internal const float PerBotThrowGapSeconds = 5f;
    internal const float PerTeamThrowGapSeconds = 0.5f;
    internal const int NormalPlannedSmokeCapPerTeam = 1;

    private sealed class GrenadeCounts
    {
        public int Flash;
        public int Smoke;
        public int HE;
        public int Molotov;

        public GrenadeCounts Clone() => new()
        {
            Flash = Flash,
            Smoke = Smoke,
            HE = HE,
            Molotov = Molotov,
        };
    }

    private readonly Dictionary<uint, GrenadeCounts> _committedByBot = new();
    private readonly Dictionary<uint, GrenadeCounts> _pendingByBot = new();
    private readonly Dictionary<uint, float> _lastBotThrowAt = new();
    private readonly Dictionary<int, float> _lastTeamThrowAt = new();
    private readonly Dictionary<uint, int> _openingByBot = new();
    private readonly Dictionary<int, int> _openingByTeam = new();
    private readonly Dictionary<int, int> _openingSmokeByTeam = new();
    private readonly Dictionary<int, int> _openingOffensiveByTeam = new();
    private readonly Dictionary<uint, int> _pendingOpeningByBot = new();
    private readonly Dictionary<int, int> _pendingOpeningByTeam = new();
    private readonly Dictionary<int, int> _pendingOpeningSmokeByTeam = new();
    private readonly Dictionary<int, int> _pendingOpeningOffensiveByTeam = new();
    private readonly Dictionary<int, int> _normalPlannedSmokeByTeam = new();
    private readonly Dictionary<int, int> _pendingNormalPlannedSmokeByTeam = new();
    private readonly HashSet<uint> _pendingNonEmergencyBots = new();
    private readonly HashSet<int> _pendingNonEmergencyTeams = new();

    internal static string? NormalizeGrenadeType(string grenadeType) => grenadeType.ToLowerInvariant() switch
    {
        "flash" => "flash",
        "smoke" => "smoke",
        "he" => "he",
        "molotov" or "incgrenade" => "molotov",
        _ => null,
    };

    internal ThrowReservation? TryBegin(uint botIndex, int teamNum, string grenadeType,
        ThrowReason reason, float now, float freezeEndTime, int aliveBotCount, string mode = "normal")
    {
        string? normalized = NormalizeGrenadeType(grenadeType);
        if (normalized == null) return null;
        bool isNonEmergency = reason is not ThrowReason.DefuseCover and not ThrowReason.ExtinguishFire;
        bool isOpening = freezeEndTime > 0f
            && now >= freezeEndTime
            && now - freezeEndTime < OpeningWindowSeconds;
        int openingBudget = Math.Min(Math.Max(aliveBotCount, 0), 5);
        if (!CanBegin(botIndex, teamNum, normalized, reason, now, freezeEndTime, aliveBotCount, mode)) return null;
        bool usesNormalPlannedSmokeCap = mode == "normal" && reason == ThrowReason.PlannedReplay && normalized == "smoke";
        if (usesNormalPlannedSmokeCap) AddValue(_pendingNormalPlannedSmokeByTeam, teamNum, 1);
        if (isNonEmergency)
        {
            _pendingNonEmergencyBots.Add(botIndex);
            _pendingNonEmergencyTeams.Add(teamNum);
            if (isOpening)
            {
                AddValue(_pendingOpeningByBot, botIndex, 1);
                if (reason != ThrowReason.PlantCover)
                    AddValue(_pendingOpeningByTeam, teamNum, 1);
                if (reason != ThrowReason.PlantCover)
                    AddOpeningTypePending(teamNum, normalized, 1);
            }
        }

        AddCount(_pendingByBot, botIndex, normalized);
        return new ThrowReservation(botIndex, teamNum, normalized, reason, isNonEmergency, isOpening, openingBudget, usesNormalPlannedSmokeCap);
    }

    internal bool CanBegin(uint botIndex, int teamNum, string grenadeType, ThrowReason reason,
        float now, float freezeEndTime, int aliveBotCount, string mode = "normal")
    {
        string? normalized = NormalizeGrenadeType(grenadeType);
        if (normalized == null || !CanReserveHardLimit(botIndex, normalized, mode)) return false;
        bool usesNormalPlannedSmokeCap = mode == "normal" && reason == ThrowReason.PlannedReplay && normalized == "smoke";
        if (usesNormalPlannedSmokeCap
            && GetValue(_normalPlannedSmokeByTeam, teamNum) + GetValue(_pendingNormalPlannedSmokeByTeam, teamNum) >= NormalPlannedSmokeCapPerTeam)
            return false;
        bool isNonEmergency = reason is not ThrowReason.DefuseCover and not ThrowReason.ExtinguishFire;
        if (!isNonEmergency) return true;
        if (_pendingNonEmergencyBots.Contains(botIndex) || _pendingNonEmergencyTeams.Contains(teamNum)) return false;
        if (_lastBotThrowAt.TryGetValue(botIndex, out float lastBot) && now - lastBot < PerBotThrowGapSeconds) return false;
        if (_lastTeamThrowAt.TryGetValue(teamNum, out float lastTeam) && now - lastTeam < PerTeamThrowGapSeconds) return false;
        bool isOpening = freezeEndTime > 0f && now >= freezeEndTime && now - freezeEndTime < OpeningWindowSeconds;
        if (!isOpening) return true;
        int openingBudget = Math.Min(Math.Max(aliveBotCount, 0), 5);
        if (GetValue(_openingByBot, botIndex) + GetValue(_pendingOpeningByBot, botIndex) >= OpeningThrowsPerBot) return false;
        if (reason != ThrowReason.PlantCover
            && GetValue(_openingByTeam, teamNum) + GetValue(_pendingOpeningByTeam, teamNum) >= openingBudget) return false;
        return reason == ThrowReason.PlantCover || CanBeginOpeningType(teamNum, normalized, true);
    }

    internal bool CanBeginOpeningType(int teamNum, string grenadeType, bool isOpening)
    {
        if (!isOpening) return true;
        if (grenadeType == "smoke")
            return GetValue(_openingSmokeByTeam, teamNum) + GetValue(_pendingOpeningSmokeByTeam, teamNum) < OpeningSmokeCap;
        return GetValue(_openingOffensiveByTeam, teamNum) + GetValue(_pendingOpeningOffensiveByTeam, teamNum) < OpeningOffensiveCap;
    }

    internal PacingAuditSnapshot Commit(ThrowReservation reservation, float now)
    {
        float botGap = reservation.IsNonEmergency && _lastBotThrowAt.TryGetValue(reservation.BotIndex, out float lastBot)
            ? now - lastBot : -1f;
        float teamGap = reservation.IsNonEmergency && _lastTeamThrowAt.TryGetValue(reservation.TeamNum, out float lastTeam)
            ? now - lastTeam : -1f;
        RemoveCount(_pendingByBot, reservation.BotIndex, reservation.GrenadeType);
        AddCount(_committedByBot, reservation.BotIndex, reservation.GrenadeType);
        if (reservation.UsesNormalPlannedSmokeCap)
        {
            RemoveValue(_pendingNormalPlannedSmokeByTeam, reservation.TeamNum, 1);
            AddValue(_normalPlannedSmokeByTeam, reservation.TeamNum, 1);
        }
        if (!reservation.IsNonEmergency) return Snapshot(reservation, botGap, teamGap);

        _pendingNonEmergencyBots.Remove(reservation.BotIndex);
        _pendingNonEmergencyTeams.Remove(reservation.TeamNum);
        _lastBotThrowAt[reservation.BotIndex] = now;
        _lastTeamThrowAt[reservation.TeamNum] = now;
        if (!reservation.IsOpening) return Snapshot(reservation, botGap, teamGap);

        AddValue(_openingByBot, reservation.BotIndex, 1);
        if (reservation.Reason != ThrowReason.PlantCover)
            AddValue(_openingByTeam, reservation.TeamNum, 1);
        RemoveValue(_pendingOpeningByBot, reservation.BotIndex, 1);
        if (reservation.Reason != ThrowReason.PlantCover)
            RemoveValue(_pendingOpeningByTeam, reservation.TeamNum, 1);
        if (reservation.Reason != ThrowReason.PlantCover)
            AddOpeningTypeCommitted(reservation.TeamNum, reservation.GrenadeType, 1);
        if (reservation.Reason != ThrowReason.PlantCover)
            AddOpeningTypePending(reservation.TeamNum, reservation.GrenadeType, -1);
        return Snapshot(reservation, botGap, teamGap);
    }

    internal void Cancel(ThrowReservation reservation)
    {
        RemoveCount(_pendingByBot, reservation.BotIndex, reservation.GrenadeType);
        if (reservation.UsesNormalPlannedSmokeCap)
            RemoveValue(_pendingNormalPlannedSmokeByTeam, reservation.TeamNum, 1);
        if (!reservation.IsNonEmergency) return;

        _pendingNonEmergencyBots.Remove(reservation.BotIndex);
        _pendingNonEmergencyTeams.Remove(reservation.TeamNum);
        if (!reservation.IsOpening) return;
        RemoveValue(_pendingOpeningByBot, reservation.BotIndex, 1);
        if (reservation.Reason != ThrowReason.PlantCover)
            RemoveValue(_pendingOpeningByTeam, reservation.TeamNum, 1);
        if (reservation.Reason != ThrowReason.PlantCover)
            AddOpeningTypePending(reservation.TeamNum, reservation.GrenadeType, -1);
    }

    internal void Reset()
    {
        _committedByBot.Clear();
        _pendingByBot.Clear();
        _lastBotThrowAt.Clear();
        _lastTeamThrowAt.Clear();
        _openingByBot.Clear();
        _openingByTeam.Clear();
        _openingSmokeByTeam.Clear();
        _openingOffensiveByTeam.Clear();
        _pendingOpeningByBot.Clear();
        _pendingOpeningByTeam.Clear();
        _pendingOpeningSmokeByTeam.Clear();
        _pendingOpeningOffensiveByTeam.Clear();
        _normalPlannedSmokeByTeam.Clear();
        _pendingNormalPlannedSmokeByTeam.Clear();
        _pendingNonEmergencyBots.Clear();
        _pendingNonEmergencyTeams.Clear();
    }

    private PacingAuditSnapshot Snapshot(ThrowReservation reservation, float botGap, float teamGap) => new(
        botGap, teamGap,
        GetValue(_openingByBot, reservation.BotIndex), GetValue(_openingByTeam, reservation.TeamNum),
        GetValue(_openingSmokeByTeam, reservation.TeamNum),
        GetCount(_committedByBot, reservation.BotIndex, "flash"), GetCount(_committedByBot, reservation.BotIndex, "smoke"),
        GetCount(_committedByBot, reservation.BotIndex, "he"), GetCount(_committedByBot, reservation.BotIndex, "molotov"),
        reservation.OpeningTeamLimit);

    private void AddOpeningTypeCommitted(int teamNum, string type, int amount)
    {
        if (type == "smoke") AddValue(_openingSmokeByTeam, teamNum, amount);
        else AddValue(_openingOffensiveByTeam, teamNum, amount);
    }

    private void AddOpeningTypePending(int teamNum, string type, int amount)
    {
        if (type == "smoke") AddValue(_pendingOpeningSmokeByTeam, teamNum, amount);
        else AddValue(_pendingOpeningOffensiveByTeam, teamNum, amount);
    }

    private bool CanReserveHardLimit(uint botIndex, string grenadeType, string mode)
    {
        int count = GetCount(_committedByBot, botIndex, grenadeType)
            + GetCount(_pendingByBot, botIndex, grenadeType);
        if (mode == "less")
        {
            int total = GetCount(_committedByBot, botIndex, "flash") + GetCount(_committedByBot, botIndex, "smoke")
                + GetCount(_committedByBot, botIndex, "he") + GetCount(_committedByBot, botIndex, "molotov")
                + GetCount(_pendingByBot, botIndex, "flash") + GetCount(_pendingByBot, botIndex, "smoke")
                + GetCount(_pendingByBot, botIndex, "he") + GetCount(_pendingByBot, botIndex, "molotov");
            if (total >= 4) return false;
        }
        return count < (grenadeType == "flash" ? 2 : 1);
    }

    private static int GetCount(Dictionary<uint, GrenadeCounts> map, uint key, string type)
    {
        if (!map.TryGetValue(key, out var counts)) return 0;
        return type switch
        {
            "flash" => counts.Flash,
            "smoke" => counts.Smoke,
            "he" => counts.HE,
            _ => counts.Molotov,
        };
    }

    private static void AddCount(Dictionary<uint, GrenadeCounts> map, uint key, string type)
    {
        if (!map.TryGetValue(key, out var counts))
            map[key] = counts = new GrenadeCounts();
        switch (type)
        {
            case "flash": counts.Flash++; break;
            case "smoke": counts.Smoke++; break;
            case "he": counts.HE++; break;
            default: counts.Molotov++; break;
        }
    }

    private static void RemoveCount(Dictionary<uint, GrenadeCounts> map, uint key, string type)
    {
        if (!map.TryGetValue(key, out var counts)) return;
        switch (type)
        {
            case "flash": counts.Flash--; break;
            case "smoke": counts.Smoke--; break;
            case "he": counts.HE--; break;
            default: counts.Molotov--; break;
        }
        if (counts.Flash <= 0 && counts.Smoke <= 0 && counts.HE <= 0 && counts.Molotov <= 0)
            map.Remove(key);
    }

    private static int GetValue(Dictionary<uint, int> map, uint key) => map.TryGetValue(key, out int value) ? value : 0;
    private static int GetValue(Dictionary<int, int> map, int key) => map.TryGetValue(key, out int value) ? value : 0;
    private static void AddValue(Dictionary<uint, int> map, uint key, int amount) => map[key] = GetValue(map, key) + amount;
    private static void AddValue(Dictionary<int, int> map, int key, int amount) => map[key] = GetValue(map, key) + amount;
    private static void RemoveValue(Dictionary<uint, int> map, uint key, int amount) { if (map.TryGetValue(key, out int value) && (value -= amount) > 0) map[key] = value; else map.Remove(key); }
    private static void RemoveValue(Dictionary<int, int> map, int key, int amount) { if (map.TryGetValue(key, out int value) && (value -= amount) > 0) map[key] = value; else map.Remove(key); }
}
