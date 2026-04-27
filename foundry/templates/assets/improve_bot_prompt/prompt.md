You are improving a Python bot for the Cyberloop Decker gauntlet.

Your task: write a Python `player_fn` that plays the Fighter class,
dueling subclass, as well as possible. The bot controls all game
phases: combat, rewards, deck swapping, deck rebuilding, collection
overflow, and innate-card choice.

## Required Interface

The output must be a single Python file defining:

```python
def player_fn(env, obs_json: str, action_labels: list[str]) -> int:
    ...
```

`player_fn` is called once per game action and must return an index into
`action_labels`. Never synthesize action labels. Always choose one of the
provided labels.

Start by copying `/input/bot/bot.py` to `/scratch/bot.py`. Edit only the
scratch copy. If `/input/score/scores.json` exists, read it for results
from the previous evaluation in this lane.

`/scratch/.flywheel_scratchpad` is a directory for notes you want to
preserve across work segments in this lane. Store durable notes in files
inside that directory, for example
`/scratch/.flywheel_scratchpad/notes.md`. Do not delete or replace the
scratchpad directory itself. Do not store candidate bots there; the
candidate to evaluate must still be saved to `/output/bot/bot.py`.

## Source Files

Read the game source before making strategy changes. The engine is
mounted read-only at `/source`:

- `/source/content/src/cards.rs`: card definitions, costs, effects, tags
- `/source/content/src/enemies.rs`: enemy definitions
- `/source/content/src/encounters.rs`: fight encounter compositions
- `/source/content/src/starter_decks.rs`: Fighter starter deck
- `/source/engine/src/combat.rs`: combat mechanics
- `/source/engine/src/enemy.rs`: enemy intent types
- `/source/engine/src/status.rs`: status effects
- `/source/gauntlet/src/observation.rs`: observation JSON structure
- `/source/gauntlet/src/lib.rs`: gauntlet phases and reward generation

## Observation Shape

`obs_json` contains structured game state, including:

- `phase_type`: `Combat`, `Reward`, `CollectionOverflow`,
  `InnateChoice`, `DeckSwap`, `DeckRebuild`, or `GameOver`
- player HP, block, energy, statuses, level, deck and collection IDs
- combat hand, draw pile size, discard pile size, turn, fights won
- enemies with HP, block, visible intents, and statuses
- reward cards, choice cards, acquired card, rebuild slots, innate choices

## Action Parser

Use the installed parser instead of string-parsing action labels yourself:

```python
from action_parser import parse_action_label
```

Typical parsed shapes:

- `("play_card", hand_index, target)`
- `("discard_card", hand_index)`
- `("end_turn",)`
- `("pick_reward", index)`
- `("skip_reward",)`
- `("swap_into_deck", index)`
- `("skip_swap",)`
- `("choose_deck_slot", index)`
- `("remove_collection_card", index)`
- `("choose_innate", index)`
- `("unknown", original_label)`

If parsing fails or no preferred action is available, choose a safe legal
fallback: end turn in combat if possible, otherwise index 0.

## Performance Rules

The evaluator runs many episodes. Keep `player_fn` fast.

- Do not import heavy libraries inside `player_fn`.
- Avoid expensive search over long histories.
- Avoid loops that can repeatedly play recycle cards without progress.
- If `cards_played_this_turn` is high, force `EndTurn`.

## Evaluation Protocol

You have 5 evaluations available across this lane. Each time you want
to use one, save your current candidate to `/output/bot/bot.py` and call
`request_eval`. The tool response only acknowledges that evaluation was
requested; it is not the score. Stop work after requesting evaluation.
On the next work segment, read `/input/score/scores.json` for the actual
evaluation result.

Do not call `request_eval` until `/output/bot/bot.py` contains the
candidate you want evaluated. After all 5 evaluations are used, your
last saved candidate is final.

Call `request_eval` at most once per assistant turn. Any additional
request in the same turn will be denied without running another
evaluation.

If the evaluation result reports an abort, inspect the abort reason in
`/input/score/scores.json`, then change the bot to avoid that failure
mode before requesting another evaluation.
