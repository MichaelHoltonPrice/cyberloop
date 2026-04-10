"""Action label parser for Decker bot development.

Parses the Rust debug-format action labels into structured tuples.
Available inside the eval container for bots to import:

    from action_parser import parse_action_label
"""

import re

_PLAY_RE = re.compile(
    r"^CombatAction\(PlayCard \{ hand_index: (\d+), "
    r"target: (None|Some\((\d+)\)) \}\)$"
)
_DISCARD_RE = re.compile(
    r"^CombatAction\(DiscardCard \{ hand_index: (\d+) \}\)$"
)
_PICK_RE = re.compile(r"^PickReward\((\d+)\)$")
_SWAP_RE = re.compile(r"^SwapIntoDeck\((\d+)\)$")
_DECK_RE = re.compile(r"^ChooseDeckSlotCard\((\d+)\)$")
_REMOVE_RE = re.compile(r"^RemoveCollectionCard\((\d+)\)$")
_INNATE_RE = re.compile(r"^ChooseInnate\((\d+)\)$")


def parse_action_label(label):
    """Parse a Rust debug-format action label into a structured tuple.

    Returns:
        ("end_turn",)
        ("skip_reward",)
        ("skip_swap",)
        ("play_card", hand_index: int, target: int | None)
        ("discard_card", hand_index: int)
        ("pick_reward", index: int)
        ("swap_into_deck", index: int)
        ("choose_deck_slot", index: int)
        ("remove_collection_card", index: int)
        ("choose_innate", index: int)
        ("unknown", original_label: str)
    """
    if label == "CombatAction(EndTurn)":
        return ("end_turn",)
    if label == "SkipReward":
        return ("skip_reward",)
    if label == "SkipSwap":
        return ("skip_swap",)

    m = _PLAY_RE.match(label)
    if m:
        target = None if m.group(2) == "None" else int(m.group(3))
        return ("play_card", int(m.group(1)), target)

    m = _DISCARD_RE.match(label)
    if m:
        return ("discard_card", int(m.group(1)))

    m = _PICK_RE.match(label)
    if m:
        return ("pick_reward", int(m.group(1)))

    m = _SWAP_RE.match(label)
    if m:
        return ("swap_into_deck", int(m.group(1)))

    m = _DECK_RE.match(label)
    if m:
        return ("choose_deck_slot", int(m.group(1)))

    m = _REMOVE_RE.match(label)
    if m:
        return ("remove_collection_card", int(m.group(1)))

    m = _INNATE_RE.match(label)
    if m:
        return ("choose_innate", int(m.group(1)))

    return ("unknown", label)
