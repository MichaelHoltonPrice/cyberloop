"""Baseline semantic-action to GUI-operation mapper for Decker.

The StepThroughGui block imports this file and calls ``plan_actions`` once per
bot-selected engine action. Return a list of operation dictionaries; the block
applies them through the desktop API and compares the exported game state to
the Rust-engine prediction.
"""

from __future__ import annotations


CARD_X_BY_HAND_SIZE = {
    1: [614],
    2: [547, 679],
    3: [479, 614, 745],
    4: [413, 547, 679, 811],
    5: [347, 479, 614, 745, 879],
    6: [300, 415, 530, 645, 760, 875],
    7: [270, 370, 470, 570, 670, 770, 870],
    8: [250, 335, 420, 505, 590, 675, 760, 845],
}

ENEMY_X_BY_COUNT = {
    1: [476],
    2: [476, 828],
    3: [476, 706, 940],
    4: [476, 647, 830, 1005],
    5: [380, 510, 640, 770, 900],
}

CARD_NAME_Y = 584
ENEMY_Y = 258
END_TURN = (1155, 639)
SKIP_BUTTON = (637, 527)
SKIP_SWAP = (637, 648)
PLAYER_TARGET = (224, 220)
REWARD_CARDS = [(476, 385), (637, 385), (797, 385)]
DISMISS_POPUP = (640, 365)


def plan_actions(context):
    """Translate one parsed semantic action into desktop operations.

    ``context["parsed_action"]`` is the tuple-like result of
    ``scripts/action_parser.py``. Coordinates are calibrated for the standard
    1280x720 Decker desktop.
    """
    parsed = tuple(context["parsed_action"])
    observation = context.get("observation") or {}
    kind = parsed[0] if parsed else "unknown"

    if kind == "play_card":
        _, hand_index, target = parsed
        hand = observation.get("hand") or []
        enemies = observation.get("enemies") or []
        clicks = [(_card_x(len(hand), int(hand_index)), CARD_NAME_Y)]
        if target is not None:
            clicks.append((_enemy_x(len(enemies), int(target)), ENEMY_Y))
        else:
            clicks.append(PLAYER_TARGET)
        return _clicks(clicks)

    if kind == "discard_card":
        _, hand_index = parsed
        hand = observation.get("hand") or []
        return _clicks([(_card_x(len(hand), int(hand_index)), CARD_NAME_Y)])

    if kind == "end_turn":
        return _clicks([END_TURN])

    if kind == "pick_reward":
        _, index = parsed
        idx = int(index)
        if 0 <= idx < len(REWARD_CARDS):
            return _clicks([REWARD_CARDS[idx]])
        return _clicks([SKIP_BUTTON])

    if kind == "skip_reward":
        return _clicks([SKIP_BUTTON])

    if kind == "skip_swap":
        return _clicks([SKIP_SWAP])

    if kind in {"swap_into_deck", "choose_deck_slot", "remove_collection_card", "choose_innate"}:
        _, index = parsed
        return _clicks([_deck_choice_position(int(index))])

    raise ValueError(f"unknown semantic action: {parsed!r}")


def _clicks(points):
    operations = []
    for index, (x, y) in enumerate(points):
        operations.append({"type": "click", "x": int(x), "y": int(y)})
        if index < len(points) - 1:
            operations.append({"type": "wait", "duration_ms": 300})
    return operations


def _card_x(hand_size, hand_index):
    if hand_size <= 0:
        return CARD_X_BY_HAND_SIZE[1][0]
    positions = CARD_X_BY_HAND_SIZE.get(hand_size)
    if positions is None:
        span = 580
        start = 614 - span // 2
        step = span // max(hand_size - 1, 1)
        positions = [start + i * step for i in range(hand_size)]
    index = max(0, min(int(hand_index), len(positions) - 1))
    return positions[index]


def _enemy_x(enemy_count, enemy_index):
    if enemy_count <= 0:
        return ENEMY_X_BY_COUNT[1][0]
    positions = ENEMY_X_BY_COUNT.get(enemy_count)
    if positions is None:
        span = 530
        start = 476
        step = span // max(enemy_count - 1, 1)
        positions = [start + i * step for i in range(enemy_count)]
    index = max(0, min(int(enemy_index), len(positions) - 1))
    return positions[index]


def _deck_choice_position(index):
    col = int(index) % 4
    row = int(index) // 4
    return 350 + col * 190, 250 + row * 80
