# Design Conclusion

This document records the final design direction settled on after exploring several alternatives. **All previous experimental code was reverted** — nothing from the TWAG-dice, card-hands, or hunt-loop experiments remains in the working tree. This is the plan going forward.

## The core idea: FTL blue options

The game is a Witcher-inspired terminal adventure built on **self-contained encounter cards** drawn from a shuffled deck (FTL / Pathway style). There are no fixed links between encounters.

The final mechanic: **cards are permanent capabilities you own**, and they unlock **blue options** in encounters — exactly like FTL's Teleporter or Cloaking systems unlock special options at beacons.

```
Forest Road

A shape moves between the trees.

> Confront the beast                     [white — always available]
> Follow its prints          (Track)     [blue — requires Track card]
> Burn the brush to flush it (Burn)      [blue — requires Burn card]
> Turn back                              [white]
```

- A card is **not played or consumed**. It is a system in your loadout.
- Blue options are **authored, static choices** with better effects — not dynamically computed modifiers.
- The blue text signals "you own the thing that makes this possible."
- This is the FTL progression model in a Witcher setting.

## Why this direction was chosen

Other directions were explored and rejected:
- **TWAG dice/wounds/actions** — added systems without fixing the unsatisfying loop. Reverted.
- **Slay the Spire card hands** — hand/draw/discard/energy machinery too complex for the current scope.
- **Griftlands dual decks** — good inspiration (multiple solutions, social encounters), but full implementation is large.
- **Caves of Qud** — a different *kind* of game (simulation, not loop), massive scope.
- **Hunt loop (investigate/prepare/exploit)** — solid but a separate, larger addition.

The blue-options model gives the "multiple solutions" payoff of Griftlands with the smallest possible implementation: one flag on a choice, one styled line in the UI.

## Cards = skills/verbs (class-less)

There are **no classes**. Cards are capabilities acquired through play. Two runs diverge based on which cards you picked up.

Starting card set to build first (six, each works in 3+ encounters):
- **Track** — information / navigation
- **Pickpocket** — theft / gold
- **Sneak** — avoidance / surprise
- **Charm** — social / de-escalation
- **Burn** — arcane / aggression
- **Scavenge** — resources / loot

Card domains matter: Charm works on human enemies (Highwaymen), not monsters (Beast/Wraith). Not every card works everywhere — that's intentional.

### Card acquisition
- Merchant (buy a card)
- Event rewards (the old man teaches you Track)
- Level-up (pick 1 of 3)
- Contract rewards (card instead of / in addition to gold)

## Traits = passive modifiers (separate concept)

Traits are always-on rules changes, distinct from cards:
- Cards gate *options*; traits change *numbers*.

Examples: Scavenger (+50% gold), Thick Skin (+1 max health), Negotiator (cheaper bribes).

Traits use the same `requires_trait` flag pattern. Keep them independent — no trait combination interactions.

## Implementation plan

Small and contained (~100 lines across 4 files):

1. `Card` enum — Track, Pickpocket, Sneak, Charm, Burn, Scavenge, Strike
2. `player.cards: Vec<Card>` — the "deck" of owned capabilities
3. `Choice.requires_card: Option<Card>` — the blue-option gate
4. Blue styling in `ui/game.rs` — `Color::Cyan` for card-gated options
5. `Effect::GainItem(Item)` — new effect variant for "find herbs" style options (3-line change in `effect.rs` + one match arm in `event_choice`)
6. `Action::EndCombat { victory, follow_up }` — for Charm options like "talk them down" that end a fight without violence, reusing `leave_combat()`

### What works today with just `requires_card` + static options
- Pickpocket → +8 gold
- Scavenge → +9 gold (vs +6 base)
- Sneak → skip encounter (no effects, like "Turn back")
- Charm → "talk them down" (fight skipped)
- Burn → check with better success effects
- Forage → needs `Effect::GainItem`

### Explicitly deferred (need real new systems)
- "Reveal weakness" — weakness is already shown; would need to hide it first + track `weakness_known`
- "Peek at next card" — needs card-selection UI
- "Haggle 25% off" — works only as a hardcoded cheaper parallel option
- "First strike" — combat has no initiative state
- "Resist the cold" — no damage-modification / status system
- "Refill energy" — no energy system (Vigor was considered, not built)
- "Reputation cost" — no reputation state

## Combat example (Charm)

```
Highwaymen (human enemies):
  > Strike                          [white]
  > Sign                            [white]
  > "Talk them down"   (Charm)      [blue] → they surrender, fight ends

Beast / Wraith (monsters):
  > Strike                          [white]
  > Sign                            [white]
  > (no Charm options — it's a beast)
```

"Talk them down" = `Action::EndCombat`, ~10 lines, no new state.

## Current state (as of this document)

- All experimental code changes reverted; working tree matches the last committed state.
- Nothing committed from the design explorations.
- Existing code: event deck, dice-based checks/combat, wounds, 4 Witcher heroes, TWAG action economy — all still present and unchanged.
- The blue-options system above is the next thing to build.

## First build step

Add `Card` enum + `player.cards` + `requires_card` on `Choice`, wire ONE encounter fully (e.g. Forest Road with Track and Burn blue options), and show blue styling — so the whole chain is visible end-to-end before expanding.
