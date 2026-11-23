# Organism design — specification v0.1 (draft)

The OS is designed like a living organism that repairs itself. Each rule borrows a strategy
observed in animals. The source of each rule is given.

## 1. Modules, not monoliths (segments and colonies: annelids, botryllid tunicates)

- The system is a colony of identical-shaped modules ("zooids"). Each module has the same structure:
  a role, a local genome, and a connection to its neighbours.
- A module can be lost without losing the whole, the same way an annelid regenerates segments.

## 2. Every module carries a copy of the recipe (reserve cells: planarians, neoblasts)

- Each module keeps a small copy of the genome of its neighbours.
- If a module is destroyed, the recipe survives in the others, as neoblasts are spread through the planarian body.

## 3. Reuse before creating (hydra: morphallaxis)

- After damage, the system first reorganises the modules still alive: it moves roles and load
  toward survivors.
- It creates new modules only when the survivors cannot cover the roles.

## 4. A growth zone for missing parts (axolotl: blastema)

- When a module is missing, a growth zone is created from the local genome and rebuilds it.
- Damage triggers a signal (the equivalent of reactive oxygen species in the axolotl) that starts the rebuild.
- The rebuilt module is checked against the genome before it is connected.

## 5. No central commander (sea star: distributed nervous system)

- Repair decisions are local. Each module decides for itself, using only its neighbours.
- The whole system does not wait for a central controller to start repairing.

## 6. Wound closure stays local (Sycon and Leucosolenia sponges)

- A fault is isolated and sealed around the damaged module, so the rest keeps running.

## 7. Every role keeps a representative (sponge reaggregation: needs all cell types)

- The system refuses to start with fewer than one live module for each essential role
  (storage, display, accessibility, repair).

## 8. Whole-body regeneration is the goal (planarian, hydra, botryllid, sea star larva)

- Any fragment that contains at least one module of each role must be able to rebuild the whole.

## Open questions

- How many copies of the genome must each module keep (proposed: its neighbours plus one extra).
- How a module proves it is genuine before it is connected (proposed: SHA-256 against the genome).
- Minimum size of a viable fragment (to be measured).
