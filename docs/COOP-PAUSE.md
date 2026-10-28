# Cooperative pause — 0.125.0

ASX retains the native Paused menu, buttons and waiting panel. It does not add a
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

Guest ownership changes use the reliable native input stream with the ASX menu
opcode. Validate payload extent, current run epoch and connected peer/actor
identity. Departed/inactive actors and reused slots cannot retain another
character's menu. Teardown and a new run discard old ownership.

`pause.rs` owns pause authority and UI trampolines; `menu_pause.rs` owns personal
menu state. Session callbacks use bounded ports. Native offsets, enum/frame
translation and call shapes stay selected in `asx-game`; portable menu policy
lives in `asx-gameplay::official::MenuOwners`.

Portable policy, selected-memory fixtures and Windows stand-ins cover distinct
risks. Read [testing](TESTING.md) for the current check scope; synthetic callback
evidence does not establish every visible menu flow in a running game.
