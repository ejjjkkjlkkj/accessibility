# AccessModel — specification v0.1 (draft)

## Idea

Existing platforms ask every application to describe itself:
- Apple: the Accessibility framework (apps must expose elements).
- Windows: UI Automation (apps must implement providers).
- Linux: AT-SPI (apps must register with the bus).

AccessModel inverts this. A specialised model reads the screen and builds its own
understanding of it. Applications do not need to be changed, and they do not need
to cooperate. The model is the accessibility layer, not a framework.

## Inputs

1. Pixels: the current screen image, captured by the system.
2. Structure when available: UI tree from the platform (optional, used to check the model).
3. Context: the user's request, the active agent, the recent history.

## Outputs

1. A short spoken description of the screen (what matters now).
2. A list of actionable elements, each with: name, role, what it does, and how to reach it by keyboard.
3. Proposed actions, with a risk level. Destructive actions require spoken confirmation.

## Rules

- Text and speech are always produced together. Speech is never the only channel.
- Keyboard access to everything the model describes.
- No action runs without a visible plan, and an interrupt key stops everything at once.
- The model cannot act outside the permissions the user has granted.

## Training (later)

- Pairs of (screenshot, expected description and elements), built from apps we control first.
- Fine-tuning of a small vision-language model, run natively in the OS engine.

## Open questions

- Which vision-language base model to start from, and its licence.
- How to capture the screen on Windows from Swift (BitBlt or Windows Graphics Capture).
- Latency target for a description (proposed: under 2 seconds on this PC).
