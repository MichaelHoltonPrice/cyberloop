You are improving a Python bot for the Cyberloop Decker gauntlet.

Start by copying `/input/bot/bot.py` to `/scratch/bot.py`. Edit only
the scratch copy. The bot module must define:

```python
def player_fn(env, obs_json: str, action_labels: list[str]) -> int:
    ...
```

When you want Flywheel to evaluate the candidate, write the current
candidate to `/output/bot/bot.py`, write `eval_requested` to
`/flywheel/control/termination_reason`, and finish your turn. Flywheel
will run the EvalBot block after this execution commits.

When the bot is good enough to keep without another evaluation, write
the final bot to `/output/bot/bot.py`, write `done` to
`/flywheel/control/termination_reason`, and finish your turn.
