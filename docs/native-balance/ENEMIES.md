# Native enemies and encounter scaling

Native-only balancing reference. The recorded client version is provenance, not a strict usage requirement. Refresh values only when explicitly requested.

Source: 0.135.1; SHA-256 `71e5f1767279db4bd428d1cd9c5d50d37759b72295386a69f9e3f6e70bc59249`.

## Reading the tables

HP and damage below are **definition inputs**, not final in-run values. Speed is native world units/second; intervals and times are seconds and simulation minutes respectively. Activation minutes indicate eligibility/scheduling, not a guaranteed ordinary spawn. A dash means scripted eligibility (infinite native threshold). XP is the raw definition input, before reward processing. Creature IDs and display IDs are retained in the JSON companion.

## Spawn formulas

Let `m` be elapsed simulation minutes; `e` elapsed endless minutes (zero outside endless); `A = map health coefficient × run multiplier`; `L` the map late-health coefficient. The additional run multiplier is the native run field at `+0xf14`; it is an explicit input, not assumed universally equal to 1. `H` and `D` are heroic multipliers. `S = 0.7` on the first three maps, otherwise `1`.

For ordinary enemies / guardians:

```text
P(m) = 1 + 0.1m + 0.036m² + c m³
c = 0.00704 ordinary; 0.00594 guardian
HP = baseHP × (4.95 ordinary; 4.8 guardian) × A × P(m)
     × (1 + L × clamp(m/20, 0, 1)²) × 1.22^e × H × partyHP
partyHP = 1 solo; 0.8 × (1 + 0.72 × (N−1) × S) cooperative host
Damage = baseDamage × (1.25 ordinary; 1 guardian) × sqrt(A)
         × (1 + 0.07m + 0.015m² + 0.0002m³) × 1.096^e × D
Movement speed = baseSpeed × 1.05
```

Boss constructor:

```text
HP = baseHP × 5.75 × A × H × 1.22^e × bossPartyHP × spawnScale
bossPartyHP = 1 solo; 0.8 × (1 + 1.04 × (N−1) × S) cooperative host
Damage = baseDamage × sqrt(A) × D × 1.096^e
Movement speed = baseSpeed
```

Scheduled bosses use spawnScale=1. A highest-difficulty extra-spawn path uses 0.6. Bosses do not use the ordinary elapsed-time polynomial. These are constructor baselines; abilities, phases, attacks and temporary effects may have their own multipliers. No party damage multiplier appears in these constructor paths. A one-member cooperative host still gets the native 0.8 HP factor; it is distinct from solo.

Ordinary population target: 50 solo; `round(50 × (1 + 0.312 × (N−1) × S))` in coop. This is not total entities including bosses, summons and scripted enemies, nor a complete model of spawn pacing.

Formula evidence: ordinary/guardian constructor RVA `0x5cdee20`; health polynomials `0x5d55e90` / `0x5d55ee0`; boss constructor `0x5cf4170`; schedule `0x5cf54a0`. Calculated examples are balancing estimates; native arithmetic uses float32.

## Heroic multipliers

| Level | HP | Damage |
| --- | ---: | ---: |
| 1 | 1.79 | 1.65 |
| 2 | 1.97 | 1.77 |
| 3 | 2.16 | 1.88 |
| 4 | 2.45 | 2.01 |
| 5 | 2.72 | 2.16 |

Level 0 uses 1/1. Source: difficulty table at RVA `0x7142088`, stride 48, health/damage at row −16/−12.

## Elwynn Forest

Map coefficient **0.82**; late-health coefficient **0**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Kobold Vermin | ordinary | 0 | 24 | 3 | 1.6 | 5 | 1 |
| Young Wolf | ordinary | 0.5 | 18 | 3 | 1.3 | 7.4 | 1 |
| Murloc Streamrunner | ordinary | 1.2 | 30 | 4 | 1.6 | 5.6 | 1 |
| Kobold Geomancer | ordinary | 2 | 30 | 6 | 3.2 | 5 | 2 |
| Defias Thug | ordinary | 3 | 44 | 6 | 1.8 | 5.4 | 2 |
| Riverpaw Brute | ordinary | 4.5 | 120 | 9 | 2.2 | 4.6 | 4 |
| Prowler | ordinary | 6 | 50 | 7 | 1.3 | 7.6 | 2 |
| Defias Rogue Wizard | ordinary | 7.5 | 40 | 7 | 3.2 | 5 | 3 |
| Defias Enforcer | ordinary | 9 | 170 | 12 | 2.2 | 4.8 | 4 |
| Kobold Taskmaster | guardian | 3.5 | 600 | 18 | 2 | 5.4 | 25 |
| Gruff Swiftbite | guardian | 8 | 800 | 22 | 1.6 | 6.6 | 28 |
| Bellygrub | guardian | 13 | 1000 | 26 | 2.4 | 5 | 30 |
| Hogger | boss | 3 | 3600 | 22 | - | 5.6 | - |
| Mother Fang | boss | 6 | 9000 | 30 | - | 6 | - |
| Rhahk'Zor | boss | 9 | 17000 | 38 | - | 5.4 | - |
| Mr. Smite | boss | 12 | 28000 | 46 | - | 6.2 | - |
| Edwin VanCleef | boss | 16 | 52500 | 54 | - | 6.6 | - |
## Duskwood

Map coefficient **0.97**; late-health coefficient **0**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Skeletal Raider | ordinary | 0 | 28 | 4 | 1.6 | 5.2 | 1 |
| Plague Spreader | ordinary | 0.4 | 30 | 4 | 1.6 | 5.4 | 1 |
| Black Widow Hatchling | ordinary | 1 | 18 | 4 | 1.2 | 7.8 | 1 |
| Nightbane Shadow Weaver | ordinary | 2 | 36 | 7 | 3.2 | 5 | 2 |
| Rotting Horror | ordinary | 3 | 60 | 7 | 2 | 4.8 | 2 |
| Nightbane Dark Runner | ordinary | 4 | 55 | 8 | 1.4 | 7 | 2 |
| Fleshripper | ordinary | 5.5 | 40 | 6 | 1.4 | 7.4 | 2 |
| Restless Shade | ordinary | 7 | 46 | 8 | 3.2 | 5 | 3 |
| Bone Chewer | ordinary | 9 | 170 | 12 | 2.3 | 4.6 | 4 |
| Nightbane Worgen Alpha | guardian | 4 | 700 | 20 | 1.6 | 6.4 | 25 |
| Carrion Recluse | guardian | 8.5 | 850 | 24 | 2 | 5.6 | 28 |
| Flesh Eating Abomination | guardian | 13 | 1000 | 28 | 2.4 | 4.6 | 30 |
| Naraxis | boss | 3 | 3600 | 22 | - | 6 | - |
| Mor'Ladim | boss | 6 | 9000 | 30 | - | 5.4 | - |
| Stitches | boss | 9 | 17000 | 38 | - | 4.8 | - |
| Lupos | boss | 12 | 28000 | 46 | - | 7.4 | - |
| Morbent Fel | boss | 16 | 52500 | 54 | - | 5.6 | - |
## Warsong Gulch

Map coefficient **1.17**; late-health coefficient **0**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Burning Imp | ordinary | 0 | 26 | 4 | 1.6 | 5.2 | 1 |
| Felhound | ordinary | 0.7 | 22 | 4 | 1.4 | 7.6 | 1 |
| Fel Bat | ordinary | 1.2 | 22 | 5 | 1.3 | 8 | 1 |
| Satyr Shadowcaster | ordinary | 2 | 34 | 7 | 3.2 | 5 | 2 |
| Satyr Trickster | ordinary | 3 | 40 | 6 | 1.8 | 5.4 | 2 |
| Voidwalker | ordinary | 4.5 | 130 | 10 | 2.2 | 4.4 | 4 |
| Succubus | ordinary | 6 | 60 | 9 | 1.5 | 6.4 | 3 |
| Fire Imp | ordinary | 7.5 | 40 | 8 | 3.2 | 5 | 3 |
| Mana Fiend | ordinary | 9 | 190 | 13 | 2.2 | 5 | 5 |
| Felguard Elite | guardian | 4 | 650 | 20 | 2 | 5.6 | 25 |
| Infernal | guardian | 8.5 | 900 | 26 | 2.4 | 4.6 | 30 |
| Doomguard | guardian | 13 | 1100 | 30 | 2.4 | 5.4 | 32 |
| Doomguard Commander | boss | 3 | 3600 | 22 | - | 5.5 | - |
| Mannoroth's Herald | boss | 6 | 9000 | 30 | - | 6 | - |
| Varimathras | boss | 9 | 17000 | 38 | - | 6.2 | - |
| Lord Hel'nurath | boss | 12 | 28000 | 46 | - | 6.2 | - |
| Lord Kazzak | boss | 16 | 52500 | 54 | - | 6 | - |
## Un'Goro Crater

Map coefficient **1.32**; late-health coefficient **0**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Ravasaur Runner | ordinary | 0 | 26 | 4 | 1.3 | 7.6 | 1 |
| Young Ravasaur | ordinary | 0.6 | 24 | 4 | 1.3 | 7.8 | 1 |
| Fledgling Pterrordax | ordinary | 1.2 | 22 | 5 | 1.3 | 8 | 1 |
| Gorishi Stinger | ordinary | 2 | 34 | 7 | 3.2 | 5 | 2 |
| Bloodpetal Lasher | ordinary | 3 | 40 | 6 | 1.8 | 5.4 | 2 |
| Un'Goro Stomper | ordinary | 4.5 | 130 | 10 | 2.2 | 4.6 | 4 |
| Venomhide Ravasaur | ordinary | 6 | 60 | 9 | 1.4 | 7.2 | 3 |
| Frenzied Pterrordax | ordinary | 7.5 | 40 | 8 | 3.2 | 5 | 3 |
| Elder Diemetradon | ordinary | 9 | 190 | 13 | 2.2 | 5 | 5 |
| Ravasaur Hunter | guardian | 4 | 650 | 20 | 1.6 | 6.6 | 25 |
| Un'Goro Thunderer | guardian | 8.5 | 900 | 26 | 2.4 | 5 | 30 |
| Devilsaur | guardian | 13 | 1100 | 30 | 2.4 | 5.6 | 32 |
| Ravasaur Matriarch | boss | 3 | 3600 | 22 | - | 6.6 | - |
| Uhk'loc | boss | 6 | 9000 | 30 | - | 5.8 | - |
| Lar'korwi | boss | 9 | 17000 | 38 | - | 7 | - |
| Tyrant Devilsaur | boss | 12 | 28000 | 46 | - | 6.4 | - |
| King Mosh | boss | 16 | 52500 | 54 | - | 6.6 | - |
## Arathi Basin

Map coefficient **1.5**; late-health coefficient **0**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Witherbark Troll | ordinary | 0 | 28 | 4 | 1.6 | 5 | 1 |
| Plains Creeper | ordinary | 0.5 | 30 | 5 | 1.5 | 7.4 | 1 |
| Mesa Buzzard | ordinary | 1 | 24 | 5 | 1.4 | 7.8 | 1 |
| Witherbark Shadowcaster | ordinary | 2 | 32 | 7 | 3.2 | 5 | 2 |
| Syndicate Mercenary | ordinary | 3 | 36 | 6 | 1.4 | 6.6 | 2 |
| Boulderfist Ogre | ordinary | 4.5 | 150 | 11 | 2.4 | 4.2 | 4 |
| Feral Mountain Lion | ordinary | 6 | 46 | 8 | 1.3 | 7.4 | 3 |
| Boulderfist Magus | ordinary | 7.5 | 52 | 9 | 3.2 | 5 | 3 |
| Boulderfist Brute | ordinary | 9 | 200 | 13 | 2.2 | 5 | 5 |
| Witherbark Headhunter | guardian | 4 | 700 | 21 | 2 | 5.4 | 25 |
| Rumbling Exile | guardian | 8.5 | 950 | 27 | 2.4 | 4.6 | 30 |
| Boulderfist Enforcer | guardian | 13 | 1150 | 31 | 2.2 | 5.6 | 32 |
| Kovork | boss | 4 | 3800 | 24 | - | 5.4 | - |
| Nimar the Slayer | boss | 7 | 9400 | 30 | - | 6.4 | - |
| Marez Cowl | boss | 11 | 17500 | 40 | - | 6 | - |
| Lord Falconcrest | boss | 14 | 29000 | 46 | - | 6.6 | - |
| Lord Aliden Perenolde | boss | 18 | 54000 | 56 | - | 6 | - |
## Feralas

Map coefficient **2.05**; late-health coefficient **0**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Woodpaw Mongrel | ordinary | 0 | 28 | 4 | 1.6 | 5.4 | 1 |
| Longtooth Runner | ordinary | 0.6 | 24 | 5 | 1.3 | 7.8 | 1 |
| Sprite Darter | ordinary | 1.2 | 22 | 5 | 1.4 | 8 | 1 |
| Woodpaw Mystic | ordinary | 2 | 36 | 7 | 3.2 | 5 | 2 |
| Gordunni Ogre | ordinary | 3 | 75 | 9 | 2 | 5 | 3 |
| Wildspawn Satyr | ordinary | 4 | 50 | 8 | 1.6 | 6.2 | 2 |
| Ironfur Bear | ordinary | 5.5 | 160 | 12 | 2.2 | 4.8 | 4 |
| Gordunni Ogre-Mage | ordinary | 7 | 55 | 9 | 3.2 | 5 | 3 |
| Feral Scar Yeti | ordinary | 9 | 200 | 14 | 2.4 | 4.6 | 5 |
| Gordunni Brute | guardian | 4 | 750 | 22 | 2 | 5.4 | 25 |
| Warpwood Treant | guardian | 8.5 | 950 | 26 | 2.4 | 4.4 | 28 |
| Ironbark Protector | guardian | 13 | 1150 | 30 | 2.6 | 4.4 | 32 |
| Zevrim Thornhoof | boss | 4 | 3800 | 24 | - | 5.8 | - |
| Alzzin the Wildshaper | boss | 7 | 9400 | 32 | - | 6 | - |
| Tendris Warpwood | boss | 11 | 18000 | 40 | - | 4.8 | - |
| Immol'thar | boss | 14 | 30000 | 48 | - | 5.6 | - |
| King Gordok | boss | 18 | 56250 | 58 | - | 6 | - |
## Gurubashi Arena

Map coefficient **2.95**; late-health coefficient **0.06**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Gurubashi Scout | ordinary | 0 | 30 | 4 | 1.6 | 5.6 | 1 |
| Razzashi Raptor | ordinary | 0.5 | 26 | 5 | 1.3 | 8 | 1 |
| Bloodseeker Bat | ordinary | 1.2 | 24 | 5 | 1.3 | 8.2 | 1 |
| Hakkari Shadowcaster | ordinary | 2 | 38 | 8 | 3.2 | 5 | 2 |
| Gurubashi Axe Thrower | ordinary | 3 | 60 | 9 | 1.8 | 5.6 | 2 |
| Zulian Tiger | ordinary | 4 | 55 | 9 | 1.3 | 7.6 | 2 |
| Razzashi Serpent | ordinary | 5.5 | 140 | 12 | 2 | 5 | 4 |
| Hakkari Witch Doctor | ordinary | 7 | 52 | 9 | 3.2 | 5 | 3 |
| Son of Hakkar | ordinary | 9 | 190 | 14 | 2 | 6 | 5 |
| Gurubashi Berserker | guardian | 4 | 800 | 24 | 2 | 5.6 | 25 |
| Zulian Crocolisk | guardian | 8.5 | 950 | 26 | 2.4 | 4.8 | 28 |
| Gurubashi Champion | guardian | 13 | 1150 | 32 | 2.2 | 5.4 | 32 |
| High Priestess Jeklik | boss | 4 | 3900 | 25 | - | 6 | - |
| High Priest Venoxis | boss | 7 | 9600 | 33 | - | 5.6 | - |
| Bloodlord Mandokir | boss | 11 | 18500 | 42 | - | 6.6 | - |
| Jin'do the Hexxer | boss | 14 | 30000 | 48 | - | 5.6 | - |
| Hakkar the Soulflayer | boss | 18 | 57000 | 58 | - | 6 | - |
## Scarlet Monastery

Map coefficient **2.33**; late-health coefficient **0.12**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Scarlet Monk | ordinary | 0 | 30 | 4 | 1.6 | 5.6 | 1 |
| Scarlet Tracking Hound | ordinary | 0.5 | 26 | 5 | 1.3 | 8 | 1 |
| Scarlet Scout | ordinary | 1.2 | 26 | 5 | 1.3 | 7.4 | 1 |
| Scarlet Adept | ordinary | 2 | 38 | 8 | 3.2 | 5 | 2 |
| Scarlet Centurion | ordinary | 3 | 60 | 9 | 1.8 | 5.6 | 2 |
| Scarlet Defender | ordinary | 4.5 | 140 | 12 | 2.2 | 4.6 | 4 |
| Scarlet Gallant | ordinary | 6 | 55 | 9 | 1.4 | 7.2 | 2 |
| Scarlet Wizard | ordinary | 7 | 52 | 9 | 3.2 | 5 | 3 |
| Scarlet Protector | ordinary | 9 | 190 | 14 | 2 | 5.4 | 5 |
| Scarlet Myrmidon | guardian | 4 | 800 | 24 | 2 | 5.6 | 25 |
| Scarlet Champion | guardian | 8.5 | 950 | 26 | 2.2 | 5.4 | 28 |
| Scarlet Abbot | guardian | 13 | 1150 | 32 | 2.2 | 5.2 | 32 |
| Houndmaster Loksey | boss | 4 | 3900 | 25 | - | 6.2 | - |
| Arcanist Doan | boss | 7 | 9600 | 33 | - | 5.6 | - |
| Herod | boss | 11 | 18500 | 42 | - | 6.4 | - |
| Scarlet Commander Mograine | boss | 14 | 30000 | 48 | - | 6.2 | - |
| High Inquisitor Whitemane | boss | 18 | 57000 | 58 | - | 6 | - |
## Alterac Valley

Map coefficient **2.95**; late-health coefficient **0.18**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Trogg Tunneler | ordinary | 0 | 30 | 4 | 1.6 | 5.2 | 1 |
| Starving Frostwolf | ordinary | 0.6 | 24 | 5 | 1.3 | 7.8 | 1 |
| Winterax Axe Thrower | ordinary | 1.2 | 34 | 6 | 1.7 | 5.6 | 1 |
| Crushridge Ogre Mage | ordinary | 2 | 48 | 8 | 3.2 | 5 | 3 |
| Snowfall Wendigo | ordinary | 3 | 70 | 8 | 2 | 5 | 2 |
| Crushridge Brute | ordinary | 4.5 | 170 | 12 | 2.4 | 4.4 | 4 |
| Ice Revenant | ordinary | 6 | 90 | 9 | 1.8 | 5.6 | 3 |
| Winterax Shadow Hunter | ordinary | 7.5 | 56 | 9 | 3.2 | 5 | 3 |
| Frostwolf Packleader | ordinary | 9 | 200 | 14 | 1.6 | 6.4 | 5 |
| Alterac Yeti Lord | guardian | 4 | 750 | 22 | 2 | 5.6 | 25 |
| Mountain Giant | guardian | 8.5 | 1000 | 28 | 2.6 | 4.6 | 30 |
| Winterax Warlord | guardian | 13 | 1200 | 32 | 2.2 | 5.6 | 32 |
| Korrak the Bloodrager | boss | 4 | 4000 | 25 | - | 5.6 | - |
| Ivus the Forest Lord | boss | 8 | 10000 | 33 | - | 5.2 | - |
| Lokholar the Ice Lord | boss | 12 | 19000 | 42 | - | 5.4 | - |
| Glacius the Frozen Titan | boss | 16 | 23250 | 48 | - | 5 | - |
| Azuregos | boss | 20 | 58500 | 60 | - | 6 | - |
## Blackrock Mountain

Map coefficient **4.14**; late-health coefficient **0.25**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Dark Iron Slaver | ordinary | 0 | 30 | 5 | 1.6 | 5.2 | 1 |
| Black Broodling | ordinary | 0.5 | 24 | 5 | 1.3 | 7.8 | 1 |
| Blackrock Grunt | ordinary | 1.2 | 36 | 6 | 1.7 | 5.6 | 1 |
| Dark Iron Geomancer | ordinary | 2 | 38 | 8 | 3.2 | 5 | 2 |
| Anvilrage Guardsman | ordinary | 3 | 80 | 10 | 2 | 5 | 3 |
| Blazing Elemental | ordinary | 4 | 55 | 9 | 1.4 | 7 | 2 |
| Firegut Ogre | ordinary | 5.5 | 170 | 13 | 2.2 | 4.8 | 4 |
| Black Wyrmkin | ordinary | 7 | 60 | 10 | 3.2 | 5 | 3 |
| Fireguard Destroyer | ordinary | 9 | 210 | 15 | 2.4 | 4.6 | 5 |
| Core Hound | guardian | 4 | 850 | 26 | 1.8 | 6 | 25 |
| Black Drake | guardian | 8.5 | 1000 | 28 | 2.2 | 6.2 | 28 |
| Molten Giant | guardian | 13 | 1250 | 34 | 2.6 | 4.4 | 32 |
| General Angerforge | boss | 4 | 4200 | 26 | - | 6 | - |
| Warchief Rend Blackhand | boss | 8 | 10500 | 34 | - | 6.4 | - |
| Emperor Dagran Thaurissan | boss | 12 | 19500 | 42 | - | 5.8 | - |
| Majordomo Executus | boss | 16 | 24000 | 50 | - | 5.8 | - |
| Nefarian | boss | 20 | 60000 | 62 | - | 6.2 | - |
## Gates of Ahn'Qiraj

Map coefficient **3.26**; late-health coefficient **0.32**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Silithid Swarmer | ordinary | 0 | 24 | 5 | 1.2 | 7.6 | 1 |
| Silithid Creeper | ordinary | 0.5 | 32 | 6 | 1.6 | 5.6 | 1 |
| Hive'Zara Stinger | ordinary | 1.2 | 26 | 6 | 1.3 | 8.2 | 1 |
| Twilight Geolord | ordinary | 2 | 40 | 8 | 3.2 | 5 | 2 |
| Qiraji Gladiator | ordinary | 3 | 90 | 11 | 2 | 5.4 | 3 |
| Stonelash Scorpid | ordinary | 4 | 60 | 10 | 1.4 | 6.8 | 2 |
| Silithid Ravager | ordinary | 5.5 | 190 | 14 | 2.2 | 4.8 | 4 |
| Qiraji Mindslayer | ordinary | 7 | 60 | 11 | 3.2 | 5 | 3 |
| Dust Stormer | ordinary | 9 | 230 | 16 | 2.2 | 5.6 | 5 |
| Qiraji Champion | guardian | 4 | 900 | 28 | 2 | 5.6 | 25 |
| Anubisath Sentinel | guardian | 8.5 | 1100 | 30 | 2.4 | 4.8 | 28 |
| Obsidian Destroyer | guardian | 13 | 1350 | 36 | 2.6 | 4.6 | 32 |
| Kurinnaxx | boss | 4 | 4400 | 27 | - | 6 | - |
| General Rajaxx | boss | 8 | 11000 | 35 | - | 6.2 | - |
| Princess Huhuran | boss | 12 | 20000 | 44 | - | 6.4 | - |
| Ossirian the Unscarred | boss | 16 | 24750 | 52 | - | 6 | - |
| C'Thun | boss | 20 | 55500 | 64 | - | 2.4 | - |
## Naxxramas

Map coefficient **6.25**; late-health coefficient **0.4**.

| Enemy | Kind | Minute | Base HP | Base damage | Interval | Speed | XP input |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plagued Ghoul | ordinary | 0 | 30 | 5 | 1.6 | 5.4 | 1 |
| Dread Creeper | ordinary | 0.5 | 28 | 6 | 1.3 | 7.8 | 1 |
| Soldier of the Frozen Wastes | ordinary | 1.2 | 26 | 6 | 1.4 | 8 | 1 |
| Naxxramas Cultist | ordinary | 2 | 42 | 9 | 3.2 | 5 | 2 |
| Carrion Spinner | ordinary | 3 | 90 | 11 | 2 | 5.4 | 3 |
| Bile Retcher | ordinary | 5.5 | 190 | 14 | 2.2 | 4.6 | 4 |
| Stoneskin Gargoyle | ordinary | 4 | 62 | 10 | 1.4 | 7 | 2 |
| Naxxramas Acolyte | ordinary | 7 | 62 | 11 | 3.2 | 5 | 3 |
| Living Monstrosity | ordinary | 9 | 240 | 16 | 2.2 | 5.6 | 5 |
| Soul Weaver | guardian | 4 | 950 | 28 | 2 | 5.6 | 25 |
| Unstoppable Abomination | guardian | 8.5 | 1150 | 31 | 2.4 | 4.8 | 28 |
| Guardian of Icecrown | guardian | 13 | 1400 | 37 | 2.6 | 4.8 | 32 |
| Highlord Mograine | guardian | - | 1400 | 37 | 2.4 | 5.6 | 32 |
| Thane Korth'azz | guardian | - | 1400 | 37 | 2.4 | 5.6 | 32 |
| Lady Blaumeux | guardian | - | 1400 | 37 | 2.4 | 5.6 | 32 |
| Sir Zeliek | guardian | - | 1400 | 37 | 2.4 | 5.6 | 32 |
| Anub'Rekhan | boss | 4 | 4600 | 28 | - | 5.8 | - |
| Noth the Plaguebringer | boss | 8 | 11500 | 36 | - | 5.6 | - |
| Patchwerk | boss | 12 | 21000 | 46 | - | 5 | - |
| Sapphiron | boss | 16 | 26000 | 54 | - | 5.8 | - |
| Kel'Thuzad | boss | 20 | 58500 | 66 | - | 5.8 | - |

## Worked comparisons

Normal difficulty, solo, run multiplier=1, no endless. Guardians evaluated at their scheduled minute; bosses at their scheduled arrival. Values rounded for reading.

| Map / enemy | Minute | Final HP estimate | Base attack damage estimate |
| --- | ---: | ---: | ---: |
| Elwynn Forest / Kobold Taskmaster | 3.5 | 4831.1 | 23.4 |
| Elwynn Forest / Hogger | 3 | 16974.0 | 19.9 |
| Duskwood / Nightbane Worgen Alpha | 4 | 7679.2 | 30.2 |
| Duskwood / Naraxis | 3 | 20079.0 | 21.7 |
| Warsong Gulch / Felguard Elite | 4 | 8600.9 | 33.2 |
| Warsong Gulch / Doomguard Commander | 3 | 24219.0 | 23.8 |
| Un'Goro Crater / Ravasaur Hunter | 4 | 9703.6 | 35.2 |
| Un'Goro Crater / Ravasaur Matriarch | 3 | 27324.0 | 25.3 |
| Arathi Basin / Witherbark Headhunter | 4 | 11875.0 | 39.4 |
| Arathi Basin / Kovork | 4 | 32775.0 | 29.4 |
| Feralas / Gordunni Brute | 4 | 17388.5 | 48.3 |
| Feralas / Zevrim Thornhoof | 4 | 44792.5 | 34.4 |
| Gurubashi Arena / Gurubashi Berserker | 4 | 26754.6 | 63.2 |
| Gurubashi Arena / High Priestess Jeklik | 4 | 66153.8 | 42.9 |
| Scarlet Monastery / Scarlet Myrmidon | 4 | 21182.2 | 56.2 |
| Scarlet Monastery / Houndmaster Loksey | 4 | 52250.2 | 38.2 |
| Alterac Valley / Alterac Yeti Lord | 4 | 25202.6 | 57.9 |
| Alterac Valley / Korrak the Bloodrager | 4 | 67850.0 | 42.9 |
| Blackrock Mountain / Core Hound | 4 | 40196.3 | 81.1 |
| Blackrock Mountain / General Angerforge | 4 | 99981.0 | 52.9 |
| Gates of Ahn'Qiraj / Qiraji Champion | 4 | 33607.0 | 77.5 |
| Gates of Ahn'Qiraj / Kurinnaxx | 4 | 82478.0 | 48.7 |
| Naxxramas / Soul Weaver | 4 | 68225.0 | 107.3 |
| Naxxramas / Anub'Rekhan | 4 | 165312.5 | 70.0 |

## Limits

No class-specific damage model, boss ability catalog, phase mechanics, spawn-location probability or time-to-kill prediction is implied. The raw enemy field +68 feeds a decaying death accumulator; its full downstream meaning is unresolved and it must not be treated as gold per kill. Complete raw definitions and addresses are in [native-world-balance.json](native-world-balance.json). Economy and XP reward processing are described in [ECONOMY.md](ECONOMY.md).
