# Native economy and kill rewards

Native-only reference. Recorded client: **0.135.1**; the version is provenance,
not a requirement for consulting these values. Refresh only on explicit request.
Image identity and enemy definitions: [ENEMIES.md](ENEMIES.md).

## Kill rewards

| Kind | Raw XP pickup amount | Gold pickup probability | Raw gold amount on success |
| --- | --- | ---: | ---: |
| Ordinary enemy | Stochastic rounding of `definition XP × 1.65` | 16.5% | Uniform integer 2, 3 or 4 |
| Guardian | Definition XP unchanged | 100% | 15 |

Ordinary XP uses floor plus a Bernoulli draw for the fractional remainder:
XP input 1 gives 1 or 2, with mean 1.65; input 2 gives 3 or 4, with mean 3.3.
These are generated pickup amounts, before player XP/gold multipliers and shared
receipt processing. An ordinary kill therefore produces **0.495 raw gold on
average**, not a guaranteed fractional credit. Rounding each pickup means that
multiplying 0.495 by a bonus is not generally the expected final credited gold.
Pickup availability/collection and boss rewards are separate paths.

Evidence: death reward branch RVA `0x5ce3177`–`0x5ce33dc`, pickup constructor
`0x5ce8b60`, bounded integer RNG `0x5d782c0`. Guardians also create a type-2 pickup;
this reference does not reinterpret that object as an additional gold reward.
The enemy definition field +68 is **not** the gold amount: it feeds a separate
decaying accumulator whose complete purpose remains unresolved.

## Gold credit

For raw credit `G`, player gold multiplier `B`, participant count `N`, and
`S = 0.7` on the first three maps, otherwise `1`:

```text
normalization = 1 solo
normalization = 1 / (1 + 0.312 × (N−1) × S) on the cooperative host
credited = saturating_u32(roundf(float32(G × B × normalization)))
```

Native multiplication is float32 at each step, followed by one round-to-nearest
with half away from zero. Normalization is evaluated on the host; it must not be
applied again to a forwarded receipt. The native function also appends raw coop
receipts before credit. This is the credit routine's rule, not an assertion about
all higher-level recipient policies.

| Participants | First three maps: normalization | Later maps: normalization |
| --- | ---: | ---: |
| 1 | 1 | 1 |
| 2 | 0.82075 | 0.76220 |
| 3 | 0.69599 | 0.61576 |
| 4 | 0.60416 | 0.51653 |
| 5 | 0.53373 | 0.44484 |

Wallet and earned-gold counters increase by the credited integer. Gold displayed
through the native currency API is converted to copper with `gold × 10000`.
Evidence: RVA `0x5cdd580`, gold multiplier field `+0x5fc`, wallet `+0x578`.

## Caches

A successful purchase debits the current integer price `C` once. Insufficient
gold leaves the wallet and price unchanged. The next price is:

```text
next = saturating_u32(truncate(float32(float32(C × 1.3) + increment)))
increment = 5 for difficulty 0–2; 6.25 for difficulty 3–5
```

Positive truncation is floor, not round-to-nearest. Gold-gain bonuses do not
reduce the debit. A successful purchase increments the native opened-cache and
pending-choice counters. Evidence: RVA `0x5cfbbb0`.

For illustration, **given an input price of 20** (not a claim about every run's
initial price), successive prices are:

| Difficulty | Prices |
| --- | --- |
| 0–2 | 20, 31, 45, 63, 86, 116 |
| 3–5 | 20, 32, 47, 67, 93, 127 |

## Reward skip and reroll

Skipping an eligible choice produces raw gold `max(10, floor(current cache price / 4))`,
then calls the gold-credit routine above. It is not a fixed fraction of the
player's wallet. Evidence: RVA `0x5d45840`.

Reroll consumes an available free reroll first. Otherwise the native paid cost is
`100 × 3^k`, saturated at uint32 maximum, where `k` counts prior paid rerolls:
100, 300, 900, 2700, 8100, 24300, etc. A failed payment does not increase `k`.
A successful payment debits the wallet and regenerates choices. Evidence:
RVA `0x5d49610`. Free-reroll replenishment and resets depend on reward lifecycle;
this price formula alone does not imply a per-run reset policy.

## Scope and maintenance

Enemy HP/damage scaling, map coefficients and comparison examples are in
[ENEMIES.md](ENEMIES.md). Generic stats and common items are in
[NATIVE-BALANCE.md](NATIVE-BALANCE.md).

This is not a complete economic simulation: initial cache-price initialization,
boss loot tables, item sale values, XP thresholds, pickup loss/collection rates,
class-specific bonuses and full spawn pacing are not yet characterized here.
Do not invent a gold-per-minute or time-to-kill estimate from the table alone.

Refresh enemy definitions explicitly with:

```sh
uv run --with pefile python3 tools/dev/native_world_balance.py \
  --exe /path/to/reviewed/AzerothSurvivors.exe --output docs/native-balance
```

The extractor checks the source hash because its layout is image-specific;
the saved documents have no automatic expiry or runtime version gate.
