# Mechanical Chicken escort

AZSX adds one optional engineering escort in a solo run. This page describes
current source behavior, not an already published release. Native assets only:
the Mechanical Chicken display and Goblin Teleporter object are selected from
the supported client. Missing assets or changed native contacts are blockers,
not a reason to ship replacement resources.

## Encounter

The first placement attempt occurs between 90 and 150 simulation seconds;
unsuccessful attempts recur after a random 180–300 seconds. A successful
reservation is the only escort of that run. A 120-segment route spans about
180 yards over valid native ground. The chicken is an invulnerable scene actor,
not an enemy or ordinary escort quest. The portal marks the end of the route.

Stay within six yards. After a two-second preparation, the chicken moves one
segment every half second while the player stays close. Leaving pauses progress;
returning resumes it. Native pause freezes simulation time. The scene expires
after four simulation minutes; death and run exit clean it without reward.
Successful extraction removes both objects and awards one gadget once.

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

Gadget triggers and cooldowns use simulation time. Death or run end removes
temporary effects and identities. No uncertain native launch or grant is retried.

## Presentation and compatibility

The retained AZSX Lua provider exports structured escort and gadget data via
API 1.11. AZSXUI may display the objective and the reward, but does not own the
gameplay. The selected 0.138.0 descriptor guards native scene, spline,
projectile, combat and broadcast contacts. A future client needs a complete
native review and a live solo trial; matching bytes alone do not prove behavior.
