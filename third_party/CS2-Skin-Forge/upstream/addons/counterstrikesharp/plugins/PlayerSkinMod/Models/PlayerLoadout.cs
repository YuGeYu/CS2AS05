namespace PlayerSkinMod.Models;

public class PlayerLoadout
{
    // Legacy shared knife fields (pre-v1.6.0 panels); kept for backward compat
    public int KnifeIndex { get; set; } = -1;
    public int KnifePaint { get; set; } = -1;
    public float KnifeWear { get; set; } = 0.01f;
    public int KnifeSeed { get; set; } = 0;
    // Per-team knives (v1.6.0+)
    public int KnifeIndexCt { get; set; } = -1;
    public int KnifePaintCt { get; set; } = -1;
    public float KnifeWearCt { get; set; } = 0.01f;
    public int KnifeSeedCt { get; set; } = 0;
    public int KnifeIndexT { get; set; } = -1;
    public int KnifePaintT { get; set; } = -1;
    public float KnifeWearT { get; set; } = 0.01f;
    public int KnifeSeedT { get; set; } = 0;
    // Per-team gloves
    public int GloveIndexCt { get; set; } = -1;
    public int GlovePaintCt { get; set; } = -1;
    public float GloveWearCt { get; set; } = 0.01f;
    public int GloveSeedCt { get; set; } = 0;
    public ushort GloveDefIndexCt { get; set; } = 0;
    public int GloveIndexT { get; set; } = -1;
    public int GlovePaintT { get; set; } = -1;
    public float GloveWearT { get; set; } = 0.01f;
    public int GloveSeedT { get; set; } = 0;
    public ushort GloveDefIndexT { get; set; } = 0;
    public int AgentModelCt { get; set; } = -1;
    public int AgentModelT { get; set; } = -1;
    public string AgentModelPathCt { get; set; } = "";
    public string AgentModelPathT { get; set; } = "";
    public int MusicKit { get; set; } = -1;
    // Legacy shared weapon maps (pre-v1.6.0 panels); kept for backward compat
    public Dictionary<ushort, int> WeaponPaints { get; set; } = new();
    public Dictionary<ushort, int> WeaponSeeds { get; set; } = new();
    public Dictionary<ushort, float> WeaponWears { get; set; } = new();
    // Per-team weapon skins (v1.6.0+)
    public Dictionary<ushort, int> WeaponPaintsCt { get; set; } = new();
    public Dictionary<ushort, int> WeaponSeedsCt { get; set; } = new();
    public Dictionary<ushort, float> WeaponWearsCt { get; set; } = new();
    public Dictionary<ushort, int> WeaponPaintsT { get; set; } = new();
    public Dictionary<ushort, int> WeaponSeedsT { get; set; } = new();
    public Dictionary<ushort, float> WeaponWearsT { get; set; } = new();
    public Dictionary<ushort, List<StickerInfo>> WeaponStickers { get; set; } = new();
    public Dictionary<ushort, KeychainInfo> WeaponKeychains { get; set; } = new();
    public Dictionary<ushort, string> WeaponNametags { get; set; } = new();
    public Dictionary<ushort, StatTrakInfo> WeaponStatTrak { get; set; } = new();
    public bool UseRandom { get; set; } = true;
}

public class StickerInfo
{
    public uint Id { get; set; }
    public uint Schema { get; set; } = 0;
    public float OffsetX { get; set; } = 0f;
    public float OffsetY { get; set; } = 0f;
    public float Wear { get; set; } = 0f;
    public float Scale { get; set; } = 1f;
    public float Rotation { get; set; } = 0f;
}

public class KeychainInfo
{
    public uint Id { get; set; }
    public float OffsetX { get; set; } = 0f;
    public float OffsetY { get; set; } = 0f;
    public float OffsetZ { get; set; } = 0f;
    public int Seed { get; set; } = 0;
}

public class StatTrakInfo
{
    public bool Enabled { get; set; } = false;
    public int Count { get; set; } = 0;
}
