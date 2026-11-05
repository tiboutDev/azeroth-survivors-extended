# Specialization weapon appearances

Weapon appearances are cosmetic. Equipment, stats, unlocks and native combat
rules still use the worn items. The unique highest talent tree selects the
variant; empty builds, ties, other specializations and unmapped sets stay native.

**Worn piece (spec)** adapts the worn set weapon. **This piece (spec)** adapts the
selected owned piece, independently of the worn set. Both work in the Armory,
main-menu portrait and runs, including cooperative participants. Explicit native
Look choices keep their native meaning for each hand. A manually selected off-hand
prevents a two-handed variant from hiding it. Explicit piece choices retain the
native collection L marker and Showing row. Melee animation grip follows the
visible hands, including Enhancement white hits, Rockbiter and Stormstrike.

## Tier 0.5 and progression adjustment

Scholomance sits after Blackrock Mountain and before the final raid tiers. Its
native set index is 7; the existing set indices are unchanged.

| Set | Specialization | Main hand | Off hand |
| --- | --- | --- | --- |
| Heroism | Fury | Silent Fang | Mirah’s Song |
| Heroism | Protection | Skullforge Reaver | Crest of Retribution |
| Soulforge | Holy | Hammer of Revitalization | Argent Defender |
| Soulforge | Protection | Argent Avenger | Argent Defender |
| The Five Thunders | Enhancement | Jeklik’s Crusher (two-handed) | Hidden |
| Lawbringer | Protection | Empyrean Demolisher | Drillborer Disk |

Arms, Retribution, Elemental and Restoration keep their native weapons. Other
existing variants are unchanged: Lightforge Protection retains Ironfoe, and
Earthshatter Enhancement retains Sulfuras. The full executable mapping lives in
`crates/azsx-game/src/official_native/weapon_appearance.rs`.

## Model references

These are original Classic items, not invented tiers or Season of Discovery
variants. Display IDs describe client presentation; they are not generated AZS
gear IDs. Metadata classes/subclasses, inventory slots and sheaths match the
weapon or shield type. Availability must be checked against the selected client’s
loaded ItemDisplayInfo catalog; a catalog row alone does not prove rendered assets.

| Classic item | Item ID | Display ID | Type |
| --- | --- | --- | --- |
| [Silent Fang](https://classicdb.ch/?item=13953) | 13953 | 24756 | One-handed sword |
| [Mirah’s Song](https://classicdb.ch/?item=15806) | 15806 | 26494 | One-handed sword |
| [Skullforge Reaver](https://classicdb.ch/?item=13361) | 13361 | 25036 | One-handed sword, back sheath |
| [Crest of Retribution](https://classicdb.ch/?item=13375) | 13375 | 23825 | Shield |
| [Hammer of Revitalization](https://classicdb.ch/?item=22315) | 22315 | 6798 | One-handed mace |
| [Argent Defender](https://classicdb.ch/?item=13243) | 13243 | 25133 | Shield |
| [Argent Avenger](https://classicdb.ch/?item=13246) | 13246 | 23836 | One-handed sword |
| [Jeklik’s Crusher](https://classicdb.ch/?item=19918) | 19918 | 32782 | Two-handed mace |
| [Empyrean Demolisher](https://classicdb.ch/?item=17112) | 17112 | 29171 | One-handed mace |

Focused selected-memory tests cover the new pairs, both specialization Look
modes, unchanged equipment, native precedence, unknown sets and grip metadata.
The diagnostic catalog probe and bounded native captures cover client availability
and rendering separately. Session evidence belongs in ignored local storage.
