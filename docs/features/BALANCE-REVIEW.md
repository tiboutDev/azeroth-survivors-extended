# AZSX bonus balance review

This review compares current AZSX rules with the independent [native reference](../native-balance/NATIVE-BALANCE.md). The first pass, participant-weighted Sayge categories and the Alacrity/Renewal adjustments below are implemented. The native reference itself contains no AZSX balance policy.

## Implemented first pass

- Crystals: Red +12% damage, Blue +7% cooldown recovery input, Green +15% maximum health, Yellow +2 percentage points of critical strike chance. Permanent personal additive gains; existing free color draws, spawn budgets and claims remain.
- Supplies Recovery: +8/+12/+16%; Vitality: +20/+30/+40% maximum health. Other families, 60-second effects, 90-second reserves and encounter thresholds remain.
- Defense remains +3/+5/+7 points of damage reduction only. The earlier review incorrectly inferred hidden health from Archimtiros. Generic stat input 5 and jewelry definition kind 5 are distinct dispatch domains: the former adds reduction only, the latter also adds item-specific health. There is no hidden health to subtract. Warding enchantments also use the generic reduction input.
- Native input restoration, same-family refresh without power stacking, and no free maximum-health heal remain required.
- Lua API bonus totals, native tooltips and pickup text use the revised crystal values.

A first Common acquisition gives Accuria +25.5% damage, Quick Strike +13.5% recovery input, Heavy Dark Iron +25.2618% health and Spell Power +21% area. The revised crystals are approximately half these contributions, without consuming equipment slots. This is a budget comparison, not equal combat utility.

Recovery input H gives cooldown factor 1/(1+H). A +7% input alone shortens cooldowns by approximately 6.54%, and gives less relative reduction when H is already high. Existing character caps and additive baselines still apply.

## Implemented Sayge category weighting

Reward magnitudes, personal replacement and strongest-only shared XP/Gold remain unchanged. At each successful ritual, snapshot the N eligible participating identities and use category weights Luck:N, XP:1, Gold:1 for every personal draw. Power draws remain unchanged and independent. The automatic boss fallback uses the same category weighting. Do not reroll existing fortunes when membership changes.

| Participants | Personal Luck | Personal XP | Personal Gold | Group has XP after a fresh complete draw | Previous equal-draw group XP |
| --- | --- | --- | --- | --- | --- |
| 1 | 33.33% | 33.33% | 33.33% | 33.33% | 33.33% |
| 2 | 50% | 25% | 25% | 43.75% | 55.56% |
| 3 | 60% | 20% | 20% | 48.80% | 70.37% |
| 4 | 66.67% | 16.67% | 16.67% | 51.77% | 80.25% |
| 5 | 71.43% | 14.29% | 14.29% | 53.73% | 86.83% |

Gold has the same marginal coverage. Formula: 1-(1-1/(N+2))^N; previous coverage was 1-(2/3)^N. These are fresh full-group draw probabilities, not lifetime uptime, guaranteed joint XP-and-Gold coverage or predictions after departures. Maximum-of-several power selection still benefits groups. More personal Luck is the deliberate counterpart; the resulting balance still needs play evaluation.

An exact per-category solo-coverage target could instead use p(XP)=p(Gold)=1-(2/3)^(1/N). At five players this would put roughly 84.4% of personal draws into Luck. The moderate weighting above preserves more category variety and a measured cooperative advantage.

## Implemented enchantment adjustments

An enchantment locks on the first accepted ring upgrade. The five total-value tiers start at ring ranks 2/4/7/10/14; totals replace previous tier values rather than adding them. Two rings can contribute the same enchantment independently. Native acquisition rarity does not directly scale the enchantment.

| Enchantment | Five tier totals before this adjustment | First Common native comparison | Implemented decision |
| --- | --- | --- | --- |
| Might | 5/9/15/22/30% damage | Accuria 25.5% | Keep |
| Alacrity | 2/4/7/10/14% recovery input | Quick Strike 13.5% input | Updated to 3/5/8/11/15% |
| Fortitude | 5/10/17/25/35% health | Heavy Dark Iron 25.2618% | Keep |
| Precision | 1/2/4/7/10 critical-chance points | Dragonslayer 6 points | Keep |
| Ferocity | 10/20/32/48/65 critical-damage points | Blackhand 29.25 points | Keep; effect depends on crit chance |
| Warding | 1/2/3.5/5/7 reduction points | Archimtiros 4.4208 points plus 17.6832% health | Keep; no hidden health from Warding |
| Renewal | 0.3/0.6/1.2/2/3 health per second | Devoured 2.25 health per second | Updated to 0.5/1/1.5/2/3 |
| Reach | 5/9/15/22/30% area | Spell Power 21% | Keep |

First-acquisition figures are reference units, not the native value of a rank-14 ring. With all-Common acquisitions, rank-2/rank-7/rank-14 Accuria contributes 41.24/106.11/172.75% damage; its corresponding Might adds 5/15/30%. Quick Strike contributes 21.83/56.18/91.46% recovery input versus the previous Alacrity 2/7/14%. This supports a small Alacrity increase, not a general enchantment nerf.

Renewal's increase is concentrated at early tiers; its final tier remains 3 health/second. Native capped jewelry may stop being offered before high enchantment thresholds. Do not treat a theoretical native rank trajectory as an attainable build. Warding must be assessed against the character reduction cap and existing reduction; its defensive utility rises as that cap is approached, then saturates.

## Validation

The rules/crystals check profiles cover personal claims, stacking, refresh/expiry, Lua 5.1 totals/text, selected-adapter borrowed inputs and restoration, plus Clippy. An additional exact-image emulator test executes the native generic reduction and health branches with distinct accumulator sentinels: reduction changes only its own accumulator. The item arithmetic oracle remains separate.

This is source-level and isolated-machine-code validation, not a live gameplay balance trial. No installation, commit, push or publication is part of this pass.

Validation result: all seven steps passed in `test-results/targeted/20261101-174500-3a3fdd5d/`; the eight native-reference arithmetic tests, including the new generic-input isolation oracle, passed with Unicorn and the reviewed image supplied. No live client was launched.

Follow-up validation: all six rules/enchantments profile steps passed in `test-results/targeted/20261101-174943-5befef1e/`, covering exact category weights for one to five players, retained fortunes, duplicate completion, unchanged ritual power, enchantment text and rejection of previous ranked values. Windows-target check with tests/test-hooks also passed (11 warnings outside this adjustment). No DLL was built or deployed and no live client was launched.

## Cooperative global area/range

Approved first pass: retain 50% of positive global area/range bonuses in coop.
Apply once after the complete authoritative native rebuild, including every
participant context. Solo and base radii remain native; guest snapshots are not
adjusted again. No new native hook is introduced. This is a balancing hypothesis
for improving melee participation, not a measured guarantee across all classes.
