# Cooperative pause

AZSX retains the native Paused menu, buttons and waiting panel. It does not add a
parallel menu, network listener or desktop/CPU control mechanism.

The native `paused` notification sets the input gate; `unpaused` releases it.
`resume` completes one reward choice and cannot release the whole group's pause.
The host derives its barrier from all current choices and participant-owned
menus. World delta and movement remain blocked while that barrier is active;
choice inputs still pass through. A partial HUD has no group-resume authority.

In active Coop, Escape reaches the native pause producer. Escape while waiting
opens that participant's menu; Resume/Escape closes only that ownership. Another
menu or reward choice continues holding the barrier. Native options remain in
their native flow.

Guest ownership changes use the reliable native input stream with the AZSX menu
opcode. Validate payload extent, current run epoch and connected peer/actor
identity. Departed/inactive actors and reused slots cannot retain another
character's menu. Teardown and a new run discard old ownership.

`pause.rs` owns pause authority and UI trampolines; `menu_pause.rs` owns personal
menu state. Session callbacks use bounded ports. Native offsets, enum/frame
translation and call shapes stay selected in `azsx-game`; portable menu policy
lives in `azsx-gameplay::official::MenuOwners`.

Portable policy, selected-memory fixtures and Windows stand-ins cover distinct
risks. Read [testing](../development/TESTING.md) for the current check scope; synthetic callback
evidence does not establish every visible menu flow in a running game.

At entry to a host-owned group pause, AZSX emits the existing native
`SMSG_MONSTER_MOVE` stop representation to each participant for the other current
actors. This cancels the client's outstanding remote rendering spline, which
would otherwise keep moving while authoritative simulation time is stopped.
The local actor receives no stop for itself. Actor coordinates, simulation,
network processing and native resume authority remain unchanged; ordinary native
movement updates resume the display. The sender's selected bytes and current
slot/GUID identities are checked. No new hook or transport is added.

The retained native pause renderer also forces visibility for a participant-owned
menu. The native guest Input::Pause branch alone does not open that menu; AZSX
must carry personal ownership through the same retained visibility/rebuild path
as the waiting panel, while preserving native menu actions. A personal menu is
distinct from another participant's waiting panel and does not release the
shared barrier on close.

## Guest Escape input boundary

Observe native UI Pause actions before the guest input filter, rather than only
at the later apply handler. In 0.138.0 the filter consumes Pause (tag 7) for an
active guest and returns true, so the apply handler never sees that action.
AZSX records the validated personal-menu request first and forwards both original
arguments to the filter. Its boolean return is preserved exactly; native queue
ownership, filtering, fallback dispatch and every other action remain native.

The diagnostic overlap fixture follows the same filter-then-apply ordering.
`coop-native-escape` separately generates the native keyboard Escape event in a
normal guest client; passing synthetic ownership state cannot satisfy it.
The explicit `coop-native-escape-visual` variant adds sparse captures.
Bounded menu screenshots belong
only to the owned guest window. They do not inspect the user's desktop.

## Inspection windows

Talents and other read-only inspection windows also emit native UI Pause actions.
In a live cooperative run, AZSX consumes these payload-free requests before they
enter the native queue when no personal menu is owned. The reviewed Escape
producer is identified by its exact call return address and keeps both open and
close requests. Once a menu is owned, its Back, Options and Resume actions retain
their native queue flow. Solo, lobby, non-Pause and invalid inputs forward
unchanged. No actor phase, clock or native input buffer is rewritten.

The queue stub has a Windows forwarding test; exact-image tests validate both
Escape producers and the queue entry. `coop-ui-probe` uses real Fortune
replacements, the native talents key and separate bounded Lua character/button
actions. Its passive readout and acceptance checks require current shared values
on both clients, both windows and ongoing game time. It excludes unrelated
compatibility/crystal fixtures. Visual claims additionally require reviewing the
explicit sparse captures.
