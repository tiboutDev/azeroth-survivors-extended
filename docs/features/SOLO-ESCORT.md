# Mechanical Chicken escort

AZSX adds one optional engineering escort in a solo or cooperative run. This page describes
current source behavior, not an already published release. Native assets only:
the Mechanical Chicken display and Goblin Teleporter object are selected from
the supported client. Missing assets or changed native contacts are blockers,
not a reason to ship replacement resources.

## Encounter

The first placement attempt is scheduled 30–90 simulation seconds after the
first completed native boss encounter. Elapsed time alone never opens the
escort. Unsuccessful attempts recur after a random 45–75 seconds. A successful
reservation is the only escort of that run. Each attempt examines up to four
independent map-wide native ground sites and eight headings per site. Failed
sites consume no escort quota. The route length follows the native map radius,
at 1.5 yards per segment with a 180-yard maximum: the reviewed 0.138.0 maps
range from about 42 yards in Gurubashi Arena (32-yard radius) and 58.5 yards
in Naxxramas (44-yard radius) to 180 yards on the largest maps. Every segment
must pass native terrain and clipping checks. The start and five-yard extraction
footprints also probe interior navigation cells, not just their centers and rims. Purely visual foliage outside native navigation remains a placement limit.
A map can still reject an attempt when
no safe route exists; placement is not guaranteed. The chicken is an
invulnerable scene actor, not an enemy or ordinary escort quest. The portal
marks the end of the route.

Every active participant must be alive, playing and within six yards. After a
two-second preparation, the chicken moves one segment every half second while
the whole group stays close. Leaving pauses progress; returning resumes it. Native pause freezes simulation time. The scene expires
after four simulation minutes. A downed cooperative participant suspends movement; run exit cleans the scene without reward.
Successful extraction removes all owned objects and reserves one independently drawn
gadget for each current participant, once. Gadgets are bound to slot and GUID;
a departed participant receives no reward.

## Crash-site scenery

Four small native gnomish parts are placed behind the chicken when their
individual ground samples are safe: steel plates (GameObject display IDs 450
and 454) at 45% scale, a screw (452) and a tool (449) at 60% scale. Their GameObject
definitions are also native; no resource is bundled. The chicken's route stays
clear. Each successful prop is owned by the escort and removed on success,
failure, death or run cleanup. A failed ground sample or prop creation leaves
the escort playable without that prop. The native Goblin Teleporter (2047)
remains at extraction.

The 0.138.0 isolated client contains GameObject display references for large
and small gnomish gears (451 and 5391) but no matching 3D files in its MPQs,
so those displays are not used. The verified plate, screw and tool files are
present as native M2 models. Their visual composition still requires an
isolated in-game review; DBC and file presence alone cannot prove the rendered
size, texture or clarity.

## Run-local gadget

One of six gadgets is drawn freely with no rank, inventory item or permanent
unlock. It is equipped for the current run and triggers automatically:

| Gadget | Effect |
| --- | --- |
| Net | Roots nearby ordinary enemies using native root state. |
| Rocket Boots | Temporarily add 35% movement speed through native stat reconstruction. |
| Goblin Sapper Charge | Native bounded explosion with native self-damage, never below 75% maximum HP. |
| Mortar | Native projectile; a bounded six-yard explosion follows its identified impact. |
| Shrink Ray | Cosmetic native player scale change only; no gameplay size/reach change. |
| Protection Belt | Native temporary shield. |

Gadget triggers and cooldowns use simulation time. A participant’s death or departure removes that participant’s temporary effects
and identity; run end removes all remaining effects. No uncertain native launch or grant is retried.

## Presentation and compatibility

The retained AZSX Lua provider exports structured escort and gadget data via
API 1.12. AZSXUI displays the objective beneath the minimap, a native-icon
`INV_Misc_EngGizmos_06` marker at the chicken while it is active, and a persistent gadget
badge beside the health/Fortune medallions after extraction. The marker uses
read-only live scene coordinates and disappears with the scene. Native aura
templates supply encounter debuff borders. AZSXUI does not own gameplay. The selected 0.138.0 descriptor guards native scene, spline,
projectile, combat and broadcast contacts. A future client needs a complete
native review and live solo and two-client cooperative trials; matching bytes alone do not prove behavior.

The requested native `INV_Egg_05` is absent from every 0.138.0 MPQ in the
reviewed isolated client. No replacement asset is bundled; the available
`INV_Misc_EngGizmos_06` remains the marker until a future native client supplies
the egg icon.
