You are a game-playing bot developer for a roguelike deckbuilder called Decker.

Your task: write a Python player function that plays the Fighter class,
**{{SUBCLASS}} subclass**, as well as possible. Your bot will be evaluated
exclusively against this subclass — read the subclass-specific cards and
synergies carefully and tailor your strategy to them.

The player handles ALL game phases — combat (playing cards), reward
selection, deck swapping, deck rebuilding, collection overflow, and
innate card choice.

## Interface

Your bot must be a single Python file that defines:

    def player_fn(env, obs_json, action_labels):
        # env: the GauntletEnv instance (has get_global_features(), etc.)
        # obs_json: JSON string with full game state observation
        # action_labels: list of legal action strings (Rust debug format)
        # Returns: int (index into action_labels)
        ...

The file will be saved to bot.py in /workspace.

## Key files to read

Before writing your bot, read these files to understand the game.
The source code is mounted read-only:

- /source/content/src/cards.rs — ALL card definitions (costs, effects, synergies)
- /source/content/src/enemies.rs — enemy definitions
- /source/content/src/encounters.rs — fight encounter compositions
- /source/content/src/starter_decks.rs — Fighter starter deck
- /source/engine/src/combat.rs — combat mechanics (damage, block, statuses)
- /source/engine/src/enemy.rs — enemy intent types (Attack, Defend, Buff, etc.)
- /source/engine/src/status.rs — status effect definitions
- /source/gauntlet/src/observation.rs — observation JSON structure
- /source/gauntlet/src/lib.rs — gauntlet game mode, phases, reward generation

## Observation JSON structure

The obs_json contains:
- phase_type: "Combat", "Reward", "CollectionOverflow", "InnateChoice", "DeckSwap", "DeckRebuild", "GameOver"
- player_hp, player_max_hp, player_block, player_energy, player_max_energy
- player_statuses: list of [status_name, stacks]
- player_level
- hand: list of card objects (card_id, name, cost, tags, description, playable, total_damage, total_block, exhaust, draw_count, energy_gain, enemy_statuses, self_statuses, targets_all, hp_change, has_conditional, concentration, innate, recycle)
- enemies: list of enemy objects (index, enemy_id, name, hp, max_hp, block, alive, intent, statuses)
  - intent is a dict like {"Attack": 15}, {"Defend": 10}, {"AttackDefend": [12, 8]}, {"Buff": ["Fortified", 2]}, {"Debuff": ["Weakened", 2]}, {"BuffAllies": ["Empowered", 1]}
- reward_cards: list of card objects (during Reward phase)
- choice_cards: list of card objects (during CollectionOverflow, DeckRebuild)
- acquired_card: card object (during DeckSwap)
- play_deck_cards: list of card objects (during DeckSwap, InnateChoice)
- play_deck_card_ids, collection_card_ids: lists of card IDs
- rebuild_slot_index: int (during DeckRebuild)
- rebuild_partial_deck_card_ids: cards already placed in rebuild slots
- rebuild_remaining_card_ids: cards still available for rebuild
- draw_pile_size, discard_pile_size
- fights_won: int
- turn: int
- pending_discards: int
- cards_played_this_turn: int
- player_took_damage_last_turn: bool
- block_floor: int
- in_combat, in_reward, game_over: bools

## Action label format

Action labels are opaque Rust debug strings. Never synthesize label strings
yourself — always scan the action_labels list and return a matching index.
If parsing fails, fall back to a safe action (EndTurn in combat, index 0
otherwise).

Examples of actual label strings:
- `CombatAction(PlayCard { hand_index: 0, target: Some(1) })` — play card 0 at enemy 1
- `CombatAction(PlayCard { hand_index: 2, target: None })` — play untargeted card 2
- `CombatAction(DiscardCard { hand_index: 3 })` — discard card 3
- `CombatAction(EndTurn)` — end your turn
- `PickReward(1)` — pick reward card 1
- `SkipReward` — skip the reward
- `SwapIntoDeck(0)` — swap card 0 into play deck
- `SkipSwap` — skip the swap
- `ChooseDeckSlotCard(2)` — assign card 2 to deck slot
- `RemoveCollectionCard(5)` — remove card 5 from collection
- `ChooseInnate(0)` — choose innate card 0

## Action label parser

A parser is pre-installed in the evaluation environment. Import it:

    from action_parser import parse_action_label

It returns structured tuples:

    parse_action_label("CombatAction(PlayCard { hand_index: 0, target: Some(1) })")
    # -> ("play_card", 0, 1)

    parse_action_label("CombatAction(EndTurn)")
    # -> ("end_turn",)

    parse_action_label("PickReward(1)")
    # -> ("pick_reward", 1)

All return types:
- `("play_card", hand_index, target)` — target is int or None
- `("discard_card", hand_index)`
- `("end_turn",)`
- `("pick_reward", index)`
- `("skip_reward",)`
- `("swap_into_deck", index)`
- `("skip_swap",)`
- `("choose_deck_slot", index)`
- `("remove_collection_card", index)`
- `("choose_innate", index)`
- `("unknown", original_label)` — fallback for unrecognized labels

Use this parser. Do not write your own label parsing logic.

## Game rules summary

- Fighter class, human race, soldier background. 50-fight gauntlet.
- Each turn: draw 5 cards, get 3 energy. Play cards (cost energy), then end turn.
- Attack cards deal damage (all attacks auto-hit). Defense cards give block.
- Block absorbs damage for one turn only (resets each turn unless BlockRetention is active).
- Enemy intents are visible — you can see what they'll do.
- After winning a fight: pick a reward card (or skip), then possibly swap/rebuild deck.
- Play deck is always 16 cards. Collection can grow up to 40.
- HP persists between fights. Level up every few fights (gain HP, unlock cards).
- Goal: survive as many of the 50 fights as possible.

## CRITICAL: Performance and correctness

Your bot's player_fn is called once per game action. It MUST be fast (< 10ms).

The evaluator enforces two speed gates:
1. **Speed screen**: Before the full eval, 10 episodes run as a sample. If
   they average more than 0.5 seconds per episode, the eval is aborted.
2. **Episode timeout**: Individual episodes that exceed 60 seconds are killed
   and score 0 fights. If 3 episodes timeout consecutively, the entire eval
   is aborted.

Common pitfalls that cause timeouts:
- Infinite loops from playing recycle cards repeatedly without making progress
- O(n^2) or worse algorithms over card lists
- Importing heavy modules (torch, numpy) inside player_fn — import at module level
- Not ending the turn when no useful cards can be played

Always include a safety net: if cards_played_this_turn exceeds 50, force EndTurn.

## Testing your bot

To evaluate your bot, use the **evaluate** MCP tool. Pass your bot's file
path relative to /workspace:

    evaluate(artifact_path="bot.py")

The tool returns JSON with mean fights won, per-episode scores, and timing.

## Previous work (if available)

You may be continuing from a previous agent segment. Check for:

- `/input/bot/bot.py` — the best bot from the previous segment. If it exists,
  read it as your starting point instead of writing from scratch.
- `/input/summary/summary.txt` — a summary of what was tried previously. If it
  exists, read it for context on strategies attempted, scores achieved,
  and what to try next.

If these files do not exist, this is the first segment — start fresh.

## Workflow

1. Check for /input/bot/bot.py and /input/summary/summary.txt (read if present).
2. If starting fresh, read the key source files to understand the game.
3. Write or improve your bot at /workspace/bot.py.
4. Test: evaluate(artifact_path="bot.py")
5. Read the results and iterate. Fix bugs, improve heuristics, re-evaluate.
6. Repeat until you've used all your evaluations or are satisfied.

After you finish, write /workspace/summary.txt with:
- Best score achieved
- Strategies tried and their scores
- What worked and what didn't
- What you would try next

Keep the summary under 500 words. Facts, not speculation.
