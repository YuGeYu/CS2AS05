namespace PlayerSkinMod.Data;

public static class StaticData
{
    public static readonly int[] KnifePaints =
    {
        5, 12, 38, 40, 42, 43, 44, 59, 72, 77, 98, 143, 175,
        409, 413, 414, 415, 418, 420, 421, 568, 569, 570, 571, 572
    };

    /// <summary>
    /// Unique glove type DefIndices in panel display order (index matches panel's gloves[] array).
    /// Used for fallback when gloveDefIndex is not set in the loadout (pre-v1.5.6 configs).
    /// </summary>
    public static readonly ushort[] GloveTypes =
    {
        5027, 5030, 5031, 5032, 5033, 5034, 5035, 4725
    };

    public static readonly (ushort DefIndex, int PaintKit)[] Gloves =
    {
        (5027, 10006), (5027, 10007), (5027, 10008), (5027, 10039),
        (5030, 10018), (5030, 10019), (5030, 10037), (5030, 10038),
        (5030, 10045), (5030, 10046), (5030, 10047), (5030, 10048),
        (5030, 10073), (5030, 10074), (5030, 10075), (5030, 10076),
        (5031, 10013), (5031, 10015), (5031, 10016), (5031, 10040),
        (5031, 10041), (5031, 10042), (5031, 10043), (5031, 10044),
        (5031, 10069), (5031, 10070), (5031, 10071), (5031, 10072),
        (5032, 10009), (5032, 10010), (5032, 10021), (5032, 10036),
        (5032, 10053), (5032, 10054), (5032, 10055), (5032, 10056),
        (5032, 10081), (5032, 10082), (5032, 10083), (5032, 10084),
        (5033, 10024), (5033, 10026), (5033, 10027), (5033, 10028),
        (5033, 10049), (5033, 10050), (5033, 10051), (5033, 10052),
        (5033, 10077), (5033, 10078), (5033, 10079), (5033, 10080),
        (5034, 10030), (5034, 10033), (5034, 10034), (5034, 10035),
        (5034, 10061), (5034, 10062), (5034, 10063), (5034, 10064),
        (5034, 10065), (5034, 10066), (5034, 10067), (5034, 10068),
        (5035, 10057), (5035, 10058), (5035, 10059), (5035, 10060),
        (4725, 10085), (4725, 10086), (4725, 10087), (4725, 10088),
    };

    public static readonly Dictionary<ushort, int[]> GunPaints = new()
    {
        [1] = [17, 37, 40, 61, 90, 114, 138, 185, 231, 232, 237, 273, 296, 328, 347, 351, 397, 425, 468, 469, 470, 509, 527, 603, 645, 711, 757, 764, 805, 841, 938, 945, 962, 992, 1006, 1050, 1054, 1056, 1090, 1189, 1257, 1318, 1360, 1430, 1458],
        [2] = [28, 43, 46, 47, 112, 139, 153, 190, 220, 249, 261, 276, 307, 330, 396, 447, 450, 453, 491, 528, 544, 625, 658, 710, 747, 824, 860, 895, 903, 978, 998, 1005, 1086, 1091, 1126, 1156, 1169, 1263, 1290, 1335, 1347, 1373, 1459],
        [3] = [3, 44, 46, 78, 141, 151, 210, 223, 252, 254, 265, 274, 352, 377, 387, 427, 464, 510, 530, 585, 605, 646, 660, 693, 729, 784, 831, 837, 906, 932, 979, 1002, 1062, 1082, 1093, 1128, 1168, 1262, 1336, 1380, 1429, 1457],
        [4] = [2, 3, 38, 40, 48, 84, 129, 152, 159, 208, 230, 278, 293, 353, 367, 381, 399, 437, 479, 495, 532, 586, 607, 623, 680, 694, 713, 732, 789, 799, 808, 832, 918, 957, 963, 988, 1016, 1039, 1079, 1100, 1119, 1120, 1121, 1122, 1123, 1158, 1167, 1200, 1208, 1227, 1240, 1265, 1282, 1312, 1348, 1357, 1421, 1450, 1460],
        [7] = [14, 44, 72, 113, 122, 142, 170, 172, 180, 226, 282, 300, 302, 316, 340, 341, 380, 394, 422, 456, 474, 490, 506, 524, 600, 639, 656, 675, 707, 724, 745, 795, 801, 836, 885, 912, 921, 941, 959, 1004, 1018, 1035, 1070, 1087, 1141, 1143, 1171, 1179, 1207, 1218, 1221, 1238, 1283, 1288, 1309, 1352, 1358, 1397, 1425, 1449, 1466],
        [8] = [9, 10, 33, 46, 47, 73, 100, 110, 121, 134, 173, 197, 246, 280, 305, 375, 444, 455, 507, 541, 583, 601, 674, 690, 708, 727, 740, 758, 779, 794, 823, 845, 886, 913, 927, 942, 995, 1033, 1088, 1198, 1249, 1308, 1339, 1362, 1452, 1464],
        [9] = [30, 51, 72, 84, 137, 163, 174, 181, 212, 227, 251, 259, 279, 344, 395, 424, 446, 451, 475, 525, 584, 640, 662, 691, 718, 736, 756, 788, 803, 819, 838, 887, 917, 943, 975, 1026, 1029, 1058, 1144, 1170, 1206, 1213, 1222, 1239, 1280, 1324, 1346, 1356, 1378, 1422, 1465, 1467],
        [10] = [22, 47, 60, 92, 154, 178, 194, 218, 240, 244, 260, 288, 371, 429, 461, 477, 492, 529, 604, 626, 659, 723, 835, 863, 869, 882, 904, 919, 999, 1053, 1066, 1092, 1127, 1146, 1184, 1202, 1219, 1241, 1302, 1321, 1365, 1393, 1461, 1477],
        [11] = [6, 8, 46, 72, 74, 147, 195, 229, 235, 294, 382, 438, 465, 493, 511, 545, 606, 628, 677, 712, 739, 806, 891, 930, 980, 1034, 1095, 1129, 1305, 1328],
        [13] = [76, 83, 101, 119, 192, 216, 235, 237, 239, 241, 246, 264, 294, 297, 308, 379, 398, 428, 460, 478, 494, 546, 629, 647, 661, 790, 807, 842, 939, 972, 981, 1013, 1032, 1038, 1071, 1147, 1178, 1185, 1264, 1275, 1296, 1314, 1383, 1434],
        [14] = [22, 75, 120, 151, 170, 202, 243, 266, 401, 452, 472, 496, 547, 648, 827, 875, 900, 902, 933, 983, 1042, 1148, 1242, 1298, 1370, 1435],
        [16] = [8, 16, 17, 101, 118, 155, 164, 167, 176, 187, 215, 255, 309, 336, 384, 400, 449, 471, 480, 512, 533, 588, 632, 664, 695, 730, 780, 793, 811, 844, 874, 926, 971, 985, 993, 1041, 1063, 1097, 1149, 1165, 1209, 1210, 1228, 1255, 1266, 1281, 1313, 1353, 1364, 1432, 1446, 1463],
        [17] = [3, 17, 32, 38, 44, 98, 101, 126, 140, 157, 188, 246, 284, 310, 333, 337, 343, 372, 402, 433, 498, 534, 589, 651, 665, 682, 742, 748, 761, 812, 826, 840, 871, 898, 908, 947, 965, 1009, 1025, 1045, 1067, 1075, 1098, 1131, 1150, 1164, 1204, 1229, 1244, 1269, 1285, 1295, 1334, 1349, 1367, 1448, 1454],
        [19] = [20, 67, 100, 111, 124, 127, 133, 156, 169, 175, 182, 228, 234, 244, 283, 311, 335, 342, 359, 486, 516, 593, 611, 636, 669, 717, 726, 744, 759, 776, 828, 849, 911, 925, 936, 969, 977, 1000, 1015, 1020, 1074, 1154, 1190, 1199, 1233, 1250, 1256, 1277, 1291, 1332, 1361, 1419],
        [23] = [161, 753, 768, 781, 798, 800, 810, 846, 872, 888, 915, 923, 949, 974, 986, 1061, 1137, 1180, 1231, 1274, 1294, 1344, 1366, 1385],
        [24] = [15, 17, 37, 70, 90, 93, 131, 169, 175, 193, 250, 281, 333, 362, 392, 412, 436, 441, 488, 556, 615, 652, 672, 688, 704, 725, 778, 802, 851, 879, 916, 990, 1003, 1008, 1049, 1085, 1157, 1175, 1194, 1203, 1236, 1303, 1351, 1387, 1426],
        [25] = [42, 95, 96, 135, 146, 166, 169, 205, 238, 240, 314, 320, 348, 370, 393, 407, 505, 521, 557, 616, 654, 689, 706, 731, 760, 821, 834, 850, 970, 994, 1021, 1046, 1078, 1103, 1135, 1174, 1182, 1201, 1215, 1254, 1267, 1287, 1333, 1381, 1471],
        [26] = [3, 13, 25, 70, 148, 149, 159, 164, 171, 203, 224, 236, 267, 293, 306, 349, 376, 457, 508, 526, 542, 594, 641, 676, 692, 770, 775, 829, 873, 884, 973, 1083, 1099, 1125, 1325, 1374, 1392, 1418, 1472],
        [27] = [32, 34, 39, 70, 99, 100, 171, 177, 198, 291, 327, 385, 431, 462, 473, 499, 535, 608, 633, 666, 703, 737, 754, 773, 787, 822, 909, 948, 961, 1072, 1089, 1132, 1188, 1220, 1245, 1306, 1355],
        [28] = [28, 144, 201, 240, 285, 298, 317, 355, 369, 432, 483, 514, 610, 698, 763, 783, 920, 950, 958, 1012, 1043, 1080, 1152, 1260, 1300],
        [29] = [5, 30, 41, 83, 119, 171, 204, 246, 250, 256, 323, 345, 390, 405, 434, 458, 517, 552, 596, 638, 655, 673, 720, 797, 814, 870, 880, 953, 1014, 1140, 1155, 1160, 1272, 1391, 1427, 1475],
        [30] = [2, 17, 36, 159, 179, 206, 216, 235, 242, 248, 272, 289, 303, 374, 439, 459, 463, 520, 539, 555, 599, 614, 671, 684, 722, 733, 738, 766, 791, 795, 816, 839, 889, 905, 964, 1010, 1024, 1159, 1214, 1235, 1252, 1279, 1286, 1299, 1322, 1384, 1447, 1462],
        [31] = [292, 1172, 1183, 1205, 1268, 1297, 1382],
        [32] = [21, 32, 71, 95, 104, 184, 211, 246, 275, 327, 338, 346, 357, 389, 443, 485, 515, 550, 591, 635, 667, 700, 878, 894, 951, 960, 997, 1019, 1055, 1138, 1181, 1224, 1259, 1292, 1342, 1359],
        [33] = [5, 11, 15, 28, 102, 141, 175, 209, 213, 245, 250, 354, 365, 423, 442, 481, 500, 536, 627, 649, 696, 719, 728, 752, 782, 847, 893, 935, 940, 1007, 1023, 1096, 1133, 1163, 1246, 1326, 1354, 1386, 1436, 1468],
        [34] = [33, 39, 61, 100, 141, 148, 199, 262, 298, 329, 331, 366, 368, 386, 403, 448, 482, 549, 609, 630, 679, 697, 715, 734, 755, 804, 820, 867, 910, 931, 1037, 1094, 1134, 1193, 1211, 1225, 1258, 1278, 1301, 1310, 1330, 1341, 1375, 1388, 1423, 1469, 1473],
        [35] = [3, 25, 62, 99, 107, 145, 158, 164, 166, 170, 191, 214, 225, 248, 263, 286, 294, 298, 299, 323, 324, 356, 450, 484, 537, 590, 634, 699, 716, 746, 785, 809, 890, 929, 987, 1051, 1077, 1162, 1192, 1247, 1261, 1331, 1337, 1350, 1368, 1442, 1474],
        [36] = [15, 27, 34, 77, 78, 99, 102, 125, 130, 162, 164, 168, 207, 219, 230, 258, 271, 295, 358, 373, 388, 404, 426, 466, 467, 501, 551, 592, 650, 668, 678, 741, 749, 774, 777, 786, 813, 825, 848, 907, 928, 968, 982, 1030, 1044, 1081, 1153, 1212, 1230, 1248, 1273, 1307, 1315, 1317, 1345, 1369, 1420, 1455],
        [38] = [46, 70, 100, 116, 117, 157, 159, 165, 196, 232, 298, 312, 391, 406, 502, 518, 597, 612, 642, 685, 865, 883, 896, 914, 954, 1028, 1139, 1226, 1327, 1343, 1371, 1453, 1470],
        [39] = [28, 39, 61, 98, 101, 136, 186, 243, 247, 287, 298, 363, 378, 487, 519, 553, 598, 613, 686, 702, 750, 765, 815, 861, 864, 897, 901, 934, 955, 966, 1022, 1048, 1084, 1151, 1234, 1270, 1320, 1394],
        [40] = [26, 60, 70, 96, 99, 128, 147, 200, 222, 233, 253, 304, 319, 361, 503, 513, 538, 554, 624, 670, 743, 751, 762, 868, 877, 899, 935, 956, 967, 989, 996, 1052, 1060, 1101, 1161, 1187, 1251, 1271, 1289, 1304, 1316, 1372, 1379],
        [60] = [60, 77, 106, 160, 189, 217, 235, 254, 257, 301, 321, 326, 360, 383, 430, 440, 445, 497, 548, 587, 631, 644, 663, 681, 714, 792, 862, 946, 984, 1001, 1017, 1059, 1073, 1130, 1166, 1177, 1216, 1223, 1243, 1311, 1319, 1338, 1340, 1376, 1433, 1476],
        [61] = [25, 60, 115, 183, 217, 221, 236, 277, 290, 313, 318, 332, 339, 364, 443, 454, 489, 504, 540, 637, 653, 657, 705, 796, 817, 818, 830, 922, 991, 1027, 1031, 1040, 1065, 1102, 1136, 1142, 1173, 1186, 1217, 1253, 1284, 1323, 1377, 1431, 1451],
        [63] = [12, 32, 147, 218, 268, 269, 270, 297, 298, 315, 322, 325, 333, 334, 350, 366, 435, 453, 476, 543, 602, 622, 643, 687, 709, 859, 933, 937, 944, 976, 1036, 1064, 1076, 1195, 1329, 1390, 1443],
        [64] = [12, 27, 37, 40, 123, 522, 523, 595, 683, 701, 721, 798, 843, 866, 892, 924, 952, 1011, 1047, 1145, 1232, 1237, 1276, 1293, 1363, 1389, 1445],
    };

    public static readonly (string DesignerName, ushort DefIndex, string ModelPath)[] Knives =
    {
        ("weapon_bayonet",               500, "weapons/models/knife/bayonet/weapon_bayonet.vmdl"),
        ("weapon_knife_css",             503, "weapons/models/knife/css/weapon_knife_css.vmdl"),
        ("weapon_knife_flip",            505, "weapons/models/knife/flip/weapon_knife_flip.vmdl"),
        ("weapon_knife_gut",             506, "weapons/models/knife/gut/weapon_knife_gut.vmdl"),
        ("weapon_knife_karambit",        507, "weapons/models/knife/karambit/weapon_knife_karambit.vmdl"),
        ("weapon_knife_m9_bayonet",      508, "weapons/models/knife/m9_bayonet/weapon_knife_m9_bayonet.vmdl"),
        ("weapon_knife_tactical",        509, "weapons/models/knife/tactical/weapon_knife_tactical.vmdl"),
        ("weapon_knife_falchion",        512, "weapons/models/knife/falchion/weapon_knife_falchion.vmdl"),
        ("weapon_knife_survival_bowie",  514, "weapons/models/knife/survival_bowie/weapon_knife_survival_bowie.vmdl"),
        ("weapon_knife_butterfly",       515, "weapons/models/knife/butterfly/weapon_knife_butterfly.vmdl"),
        ("weapon_knife_push",            516, "weapons/models/knife/push/weapon_knife_push.vmdl"),
        ("weapon_knife_cord",            517, "weapons/models/knife/cord/weapon_knife_cord.vmdl"),
        ("weapon_knife_canis",           518, "weapons/models/knife/canis/weapon_knife_canis.vmdl"),
        ("weapon_knife_ursus",           519, "weapons/models/knife/ursus/weapon_knife_ursus.vmdl"),
        ("weapon_knife_gypsy_jackknife", 520, "weapons/models/knife/gypsy_jackknife/weapon_knife_gypsy_jackknife.vmdl"),
        ("weapon_knife_outdoor",         521, "weapons/models/knife/outdoor/weapon_knife_outdoor.vmdl"),
        ("weapon_knife_stiletto",        522, "weapons/models/knife/stiletto/weapon_knife_stiletto.vmdl"),
        ("weapon_knife_widowmaker",      523, "weapons/models/knife/widowmaker/weapon_knife_widowmaker.vmdl"),
        ("weapon_knife_skeleton",        525, "weapons/models/knife/skeleton/weapon_knife_skeleton.vmdl"),
        ("weapon_knife_kukri",           526, "weapons/models/knife/kukri/weapon_knife_kukri.vmdl"),
    };

    public static readonly Dictionary<string, ushort> KnifeDefIndexByName = new()
    {
        ["weapon_bayonet"]               = 500,
        ["weapon_knife_css"]             = 503,
        ["weapon_knife_flip"]            = 505,
        ["weapon_knife_gut"]             = 506,
        ["weapon_knife_karambit"]        = 507,
        ["weapon_knife_m9_bayonet"]      = 508,
        ["weapon_knife_tactical"]        = 509,
        ["weapon_knife_falchion"]        = 512,
        ["weapon_knife_survival_bowie"]  = 514,
        ["weapon_knife_butterfly"]       = 515,
        ["weapon_knife_push"]            = 516,
        ["weapon_knife_cord"]            = 517,
        ["weapon_knife_canis"]           = 518,
        ["weapon_knife_ursus"]           = 519,
        ["weapon_knife_gypsy_jackknife"] = 520,
        ["weapon_knife_outdoor"]         = 521,
        ["weapon_knife_stiletto"]        = 522,
        ["weapon_knife_widowmaker"]      = 523,
        ["weapon_knife_skeleton"]        = 525,
        ["weapon_knife_kukri"]           = 526,
    };

    public static readonly string[] CtModels =
    {
        "agents\\models\\ctm_diver\\ctm_diver_varianta.vmdl",
        "agents\\models\\ctm_diver\\ctm_diver_variantb.vmdl",
        "agents\\models\\ctm_diver\\ctm_diver_variantc.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_varianta.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_variantb.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_variantc.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_variantd.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_variante.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_variantf.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_variantg.vmdl",
        "agents\\models\\ctm_fbi\\ctm_fbi_varianth.vmdl",
        "agents\\models\\ctm_gendarmerie\\ctm_gendarmerie_varianta.vmdl",
        "agents\\models\\ctm_gendarmerie\\ctm_gendarmerie_variantb.vmdl",
        "agents\\models\\ctm_gendarmerie\\ctm_gendarmerie_variantc.vmdl",
        "agents\\models\\ctm_gendarmerie\\ctm_gendarmerie_variantd.vmdl",
        "agents\\models\\ctm_gendarmerie\\ctm_gendarmerie_variante.vmdl",
        "agents\\models\\ctm_sas\\ctm_sas.vmdl",
        "agents\\models\\ctm_sas\\ctm_sas_variantf.vmdl",
        "agents\\models\\ctm_sas\\ctm_sas_variantg.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_variante.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_variantg.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_varianti.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_variantj.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_variantk.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_variantl.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_variantm.vmdl",
        "agents\\models\\ctm_st6\\ctm_st6_variantn.vmdl",
        "agents\\models\\ctm_swat\\ctm_swat_variante.vmdl",
        "agents\\models\\ctm_swat\\ctm_swat_variantf.vmdl",
        "agents\\models\\ctm_swat\\ctm_swat_variantg.vmdl",
        "agents\\models\\ctm_swat\\ctm_swat_varianth.vmdl",
        "agents\\models\\ctm_swat\\ctm_swat_varianti.vmdl",
        "agents\\models\\ctm_swat\\ctm_swat_variantj.vmdl",
        "agents\\models\\ctm_swat\\ctm_swat_variantk.vmdl",
    };

    public static readonly string[] TModels =
    {
        "agents\\models\\tm_balkan\\tm_balkan_variantf.vmdl",
        "agents\\models\\tm_balkan\\tm_balkan_variantg.vmdl",
        "agents\\models\\tm_balkan\\tm_balkan_varianth.vmdl",
        "agents\\models\\tm_balkan\\tm_balkan_varianti.vmdl",
        "agents\\models\\tm_balkan\\tm_balkan_variantj.vmdl",
        "agents\\models\\tm_balkan\\tm_balkan_variantk.vmdl",
        "agents\\models\\tm_balkan\\tm_balkan_variantl.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_varianta.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_variantb.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_variantb2.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_variantc.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_variantd.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_variante.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_variantf.vmdl",
        "agents\\models\\tm_jungle_raider\\tm_jungle_raider_variantf2.vmdl",
        "agents\\models\\tm_leet\\tm_leet_varianta.vmdl",
        "agents\\models\\tm_leet\\tm_leet_variantb.vmdl",
        "agents\\models\\tm_leet\\tm_leet_variantc.vmdl",
        "agents\\models\\tm_leet\\tm_leet_variantd.vmdl",
        "agents\\models\\tm_leet\\tm_leet_variante.vmdl",
        "agents\\models\\tm_leet\\tm_leet_variantf.vmdl",
        "agents\\models\\tm_leet\\tm_leet_variantg.vmdl",
        "agents\\models\\tm_leet\\tm_leet_varianth.vmdl",
        "agents\\models\\tm_leet\\tm_leet_varianti.vmdl",
        "agents\\models\\tm_leet\\tm_leet_variantj.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_varianta.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_variantb.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_variantc.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_variantd.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_variantf.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_variantg.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_varianth.vmdl",
        "agents\\models\\tm_phoenix\\tm_phoenix_varianti.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varf.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varf1.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varf2.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varf3.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varf4.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varf5.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varg.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varh.vmdl",
        "agents\\models\\tm_professional\\tm_professional_vari.vmdl",
        "agents\\models\\tm_professional\\tm_professional_varj.vmdl",
    };

    public static readonly int[] KitIds =
    {
         2,   3,   4,   5,   6,   7,   8,   9,  10,  11,
        12,  13,  14,  15,  16,  17,  18,  19,  20,  21,
        22,  23,  24,  25,  26,  27,  28,  29,  30,  31,
        32,  33,  34,  35,  36,  37,  38,  39,  40,  41,
        42,  43,  44,  45,  46,  47,  48,  49,  50,  51,
        52,  53,  54,  55,  56,  57,  58,  59,  60,  61,
        62,  63,  64,  65,  66,  67,  68,  69,  70,  71,
        72,  73,  74,  75,  76,  78,  79,  80,  81,  82,
        83,  84,  85,  86,  87,  88,  89,  90,  91,  92,
        93,  94,  95,  96,  98,  99, 100, 101, 102, 103,
    };
}
