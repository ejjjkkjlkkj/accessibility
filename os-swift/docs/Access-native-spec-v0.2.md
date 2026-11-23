# Native Accessibility — specification v0.2

## Principle

Accessibility is part of the core, not a layer on top. It is not an agent, not an AI feature,
and not an optional framework. It exists the moment an object exists.

## Rule 1 — Every system object is accessible by construction

Every object the OS creates implements `AccessibleObject`:
- `name`: what it is, in plain words.
- `role`: button, list, item, text, window, dialog, status, alert.
- `state`: enabled, focused, selected, busy, error.
- `actions`: the list of things it can do, each reachable from the keyboard.

A type that does not implement it cannot be placed on screen or in the shell. The compiler enforces this.

## Rule 2 — One output bus for everything

All output goes through `OutputBus`. Nothing prints directly.
The bus sends each message to all active channels at once:
- text (always on, works with any screen reader or terminal),
- speech (always available, can be toggled),
- braille (reserved, same text in braille form).

Speech is a channel of the bus, not a feature that some apps have.

## Rule 3 — Focus is a core service

`FocusManager` holds the one focused object. Moving focus announces the new object through the bus.
Keyboard navigation is the only navigation model: next, previous, activate, back.

## Rule 4 — Interruption is always possible

A single core key, `stop`, cancels speech, cancels any running action, and returns focus to the top.

## Rule 5 — Destructive actions are explicit

Any action with a `destructive` flag asks for confirmation through the bus. It is never silent.

## What this means for the code

- `Access.swift`: `AccessibleObject`, `Role`, `State`, `Action`, `OutputBus`, `FocusManager`.
- The shell's entries and commands implement `AccessibleObject`.
- The main loop writes through `OutputBus`, not `print`.
- Agents and AI are clients of this core. They do not provide accessibility.

## Out of scope for v0.2

Screen capture, vision models, and per-app adapters.
