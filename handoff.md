# FriendBox Handoff

This handoff captures the June 14-15, 2026 Android parity session plus the later HUD/touch follow-up that finished the current Android screen-space HUD anchor pass, added `UI::ButtonHold`, introduced Android multi-touch routing controls for onscreen gamepad-style input, the later follow-up that added `UI::Thumbstick`, `Utils::IsAndroid`, a desktop HUD draw-queue separation fix, Android shortest-path angle interpolation parity, and a project setting for engine-level network status popups, the June 16, 2026 follow-up that added the first full turnbased networking stack, string-based save-data nodes, a password-mode `UI::TextInput` editor option, and the later Android packaging fix for `OnTurnBasedGameLoaded`, the June 16-17, 2026 follow-up that restored lost event/network node controls, added `Game::QuitGame`, and made Android packaging self-contained from the built engine executable, the later June 17, 2026 typed-array follow-up that shipped first-class arrays across variables, array nodes, smart-search filtering, and JSON builder parity on desktop and Android, the June 18, 2026 graph/UI follow-up that added `UI::ListBrowser`, fixed `DoOnce`/`Gate` flow-state regressions, and refined the editor graph comment/route UX, the later June 18, 2026 turnbased lifecycle follow-up that added exact-group invite replacement, completed-game removal, stale-game cleanup, global background-color controls, Android/world-viewport rendering fixes, and synced `DeleteGame` / `ResultingStatus` / `OnTurnBasedGameDeleted` behavior across Windows and Android, and the later June 18, 2026 graph/data cleanup follow-up that removed `Array::ToJson` / `Array::FromJson`, moved lightweight save data into a per-game `data` folder with `.DT` files, added graph `G` snap with overlap protection, added comment-corner drop snap, and made blueprint graph drag movement advance in whole grid cells by default.

## Repo Context

- Outer workspace: `D:\Studio\CodexFarm\FriendBox`
- Engine git repo: `D:\Studio\CodexFarm\FriendBox\FriendBox Engine`
- Use the engine repo for `git status`, commits, and pushes
- Leave `FriendBox Engine/editor_config.json` alone unless explicitly asked
- Ignore `FriendBox Engine/saves/`
- Ignore `FriendBox Engine/data/`

## Main Files Touched In This Work

- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\android-template\app\src\main\java\app\friendbox\exported\MainActivity.java`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_NODE_LIBRARY_AUDIT.md`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_VM_SUPPORT_MATRIX.md`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\android-template\app\src\main\AndroidManifest.xml`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\data\project.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\exporter.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\generated_source.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\layout.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\project_settings.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\global_var_editor.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\renderer.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\core.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\interaction.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\search_menu.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\sidebar.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\utils.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\level_editor.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\ui.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\game_loop.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\render_pass.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\turnbased.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\array.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\data.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\game.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\json.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\turnbased.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\ui.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\utils.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\scripting\graph.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\scripting\nodes.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\scripting\vm.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\main.rs`
- `D:\Studio\CodexFarm\FriendBox\handoff.md`

## What Was Achieved

Android runtime parity fixes in `MainActivity.java` now include:

- `GetVariable` fallback parity with graph variable defaults
- `GetActorVar` fallback parity with sprite-script defaults
- unconnected input fallback parity using exported pin `data_type`
- JSON pin-name fix from `"Json"` to `"InJson"`
- JSON vector parity for desktop object format `{ "x": ..., "y": ... }`
- nested event output-cache preservation so `SpawnActor -> OnStart` no longer wipes outer event outputs
- runtime `MacroInstance` execution support
- `Transform::GetPosition` action-point parity for points like `Shoot` and `Pivot`
- `Sensing::Linetrace` ignore-tag parity
- `Sensing::Linetrace` world-hit support plus debug trace rendering on Android
- frame-aware collision sampling for actor current frame and tile `variantIndex`
- `Sensing::Linetrace` step sampling adjusted toward the Windows per-pixel march
- particle collision sampling updated to use rounded next-position checks like Windows
- a shared baked per-pixel world collision source on Android, with terrain edits layered on top
- both `traceWorld(...)` and particle `DieOnCollision` now reading that same shared source

HUD / presentation work added after that parity sweep:

- world presentation now distinguishes the game-world base resolution from the HUD/UI canvas resolution
- a new world presentation mode path was added so the world can use integer fill / overscan while the HUD still uses the exact UI canvas
- `GetScreenCenterOffset` was intentionally kept tied to the game-world base resolution center for camera graphs
- camera/view-bounds presentation logic was adjusted so Android and Windows both respect the current world presentation mode
- UI anchors were added for HUD nodes and now support edge centers like `TopCenter`, `CenterRight`, and `BottomCenter`
- anchored HUD nodes now hide raw `Pos` and show `OffsetX` / `OffsetY` in the graph UI when an anchor is active
- `UI::Text` was added as a real HUD text node instead of relying on input widgets for static text
- `UI::Button`, `UI::TextInput`, and `UI::RoomBrowser` now have a `Scale` input that scales the whole widget, including text and interaction area
- Android `UI::Text` rendering was fixed after the first rollout by correcting extension-template payload lookup after Android short-name normalization

Landscape / packaging follow-up added after that:

- projects now have a real `orientation` setting with `Portrait` and `Landscape`
- mobile resolution mode now keeps the selected base world resolution and HUD canvas exactly as chosen for that orientation instead of silently rotating dimensions
- the project settings panel now treats portrait and landscape as first-class mobile presets rather than "portrait only plus rotate"
- Android export/runtime now applies the selected project orientation natively instead of forcing portrait
- Android runtime now re-enters immersive fullscreen and hides the system bars instead of leaving the right-side navigation bar and helper text visible over gameplay
- Android integer-fill world presentation now fills the physical display for overscan the same way the user expects from the editor setting
- Android touch mapping and final screen viewport logic now use the same computed presentation rectangle, which fixed the remaining mismatch between world coverage and touch space
- Android camera fractional composition now uses Windows-style rounded shift behavior instead of the earlier raw-float path
- Android packaging now generates a unique `applicationId` per exported game, so installing one packaged game no longer "updates" a different one on the same phone
- the generated Android package ID now avoids invalid numeric-leading package segments by prefixing the hash suffix
- the package dialog/exporter state now refreshes from the newly-opened project, so output path, build name, and target settings do not stay stale when switching games in the editor
- an older apparent camera-centering/jitter regression turned out to be a user-authored second `SetCameraPosition` node fighting the tank camera logic in another shared blueprint, not an engine-side follow-camera bug

Android HUD / touch follow-up added after that:

- `UI::ButtonHold` now exists as a dedicated HUD node with `OnPressed`, `WhileHeld`, `OnReleased`, and `IsHeld`
- `UI::Button` and `UI::ButtonHold` now support shape-aware interaction and drawing for `Rect`, `Round`, and directional triangle shapes
- `UI::Button` and `UI::ButtonHold` now support a `Content` enum for `Text`, `ArrowUp`, `ArrowDown`, `ArrowLeft`, and `ArrowRight`
- Android `ButtonHold` now renders correctly, preserves hold state across frames, and supports slide-off cancel behavior
- Android `ButtonHold` also now latches when an already-held finger slides into the button, so onscreen gamepad movement can transition between buttons without a re-touch
- setter nodes now exist for virtual axis overrides so HUD buttons can drive the same axis values as desktop/project-settings input
- Android HUD widgets with anchors now resolve against the actual physical screen instead of the intermediate HUD buffer:
- `UI::Button`
- `UI::ButtonHold`
- `UI::Text`
- `UI::TextInput`
- `UI::RoomBrowser`
- Android anchored HUD widgets above now also draw in the final screen-space overlay pass, which is what fixed the earlier device mismatch where Android anchors visually drifted relative to Windows
- Android `TextInput` spacing was corrected so the label/title sits above the field like Windows instead of overlapping it on device
- Android touch capture for HUD buttons no longer updates gameplay touch/world position while a UI finger is holding a HUD control
- Android projects now have a real `android_two_finger_pan_enabled` setting exposed in Project Settings as `Android Two-Finger World Pan`
- when that setting is disabled, Android touch routing now supports:
- holding multiple HUD buttons at once
- holding a HUD button while a separate non-UI finger drives gameplay touch/mouse aim
- move+aim and move+shoot combinations without the UI finger hijacking gameplay aim
- when that setting is enabled, Android keeps the earlier two-finger world-pan behavior

Later HUD / input / runtime follow-up added after that:

- `UI::Thumbstick` now exists as a dedicated HUD node on desktop and Android
- `UI::Thumbstick` behaves like a holdable virtual stick with `OnPressed`, `WhileHeld`, `OnReleased`, `IsHeld`, and normalized `AxisX` / `AxisY` outputs
- `UI::Thumbstick` also now outputs `Degrees` so it can directly drive rotation graphs such as `SetRotation`
- Android `Thumbstick` captures a specific UI pointer/finger, keeps tracking that pointer while held, clamps the knob to the stick radius, and resets to center on release
- `Utils::IsAndroid` now exists as a runtime getter node so blueprints can branch between desktop and Android input paths without string comparisons
- desktop HUD widgets now use a dedicated transient HUD draw-command queue instead of sharing the generic world draw-command queue
- that draw-queue split fixed the bug where some `UI::*` HUD elements could also appear in the game world when their offsets made the duplicate visible
- Android `AngleInterpTo` and `AngleInterpToConstant` now mirror the desktop shortest-path wrap behavior across the 180/0 boundary instead of taking the long scalar route
- projects now have a `Show Network Status Messages` setting in Project Settings
- that setting now gates the engine-level runtime popups for connect, disconnect, joined-room, and network-error messages on both desktop and Android

Turnbased / data-state follow-up added on June 16:

- a dedicated Rust-side turnbased runtime now exists in `src/engine/turnbased.rs`
- the new turnbased system is intentionally separate from realtime `network.rs`; use `turnbased.rs` for server-backed async/turn-based play and keep `network.rs` for realtime multiplayer
- turnbased node surface now exists in `src/libs/turnbased.rs` and includes:
- auth/session nodes such as `SetBaseUrl`, `Register`, `Login`, `Logout`, `DeleteAccount`, `IsAuthenticated`, `GetAuthToken`, `GetMyPlayerID`, and `GetLastError`
- profile/search nodes such as `GetMe`, `UpdateProfile`, `SearchPlayers`, and `GetProfileJson`
- invite/game nodes such as `ListInvites`, `CreateInvite`, `AcceptInvite`, `DeclineInvite`, `ListGames`, `GetGame`, `StartGame`, `SubmitTurn`, `GetLoadedGame`, and the related JSON getters
- the intended data model is still "engine-owned logic": game state, per-player state, invites, and turns are exposed to blueprints as JSON strings so the game can load, mutate, and write them back itself
- `SearchPlayers` intentionally returns `PlayersJson` as a JSON array rather than flattening to three plain outputs, because a search can validly return zero, one, or multiple players
- `On TurnBased Players Found` fires when the search request succeeds, including the empty-result case where `PlayersJson` is `[]`; request failures go through `On TurnBased Error`
- duplicate alias turnbased events were cleaned back out; there is now one real event per async result instead of parallel alias events carrying the same payload
- `OnTurnBasedGameLoaded` is the one loaded-game event and now carries `GameJson`, `ParticipantsJson`, and `TurnsJson` on the same event
- the Android runtime dispatch now matches that same one-event-per-result shape
- account deletion now exists in both the engine node set and the outer FriendBox API/server path
- the user confirmed register/login/search/account-delete flows were working against the live server once the staged server changes were deployed
- new `Data::*` string-state nodes now exist in `src/libs/data.rs`:
- `SaveString`
- `LoadString`
- `DoesSaveStringExist`
- `DeleteData`
- these are separate from full `SaveGame` / `LoadGame`; they are lightweight key/value persistence for string data such as player IDs, auth tokens, usernames, profile JSON, or other turnbased helper state
- Android runtime support exists for the key data-string nodes the user asked to verify, including `SaveString`, `LoadString`, and `DoesSaveStringExist`
- `UI::TextInput` now has a `Password Mode` layout toggle in the editor so typed characters can be masked in the widget while the real string value still flows through the node outputs
- `String::StripHttpErrorPrefix` now exists as a cleanup helper for UI-facing messages like `http_409: username_already_taken`
- the Android runtime needed a later packaging fix after the turnbased event cleanup:
- `OnTurnBasedGameLoaded` now parses `event.json` inside a `try/catch` for `JSONException`
- Android falls back to `"[]"` for `ParticipantsJson` and `TurnsJson` if the loaded game payload cannot be parsed
- that fix was required because `new JSONObject(event.json)` is a checked-exception site in Java and otherwise breaks release APK compilation

June 16-17 node/editor/export follow-up added after that:

- `OnNetworkEvent` now again exposes its inline event-name text field in the node UI instead of only showing the title text
- `OnAnimEvent` now again exposes its inline event-name text field in the node UI instead of only showing the title text
- `Raise Network Event` now again exposes the `Target` enum and `Include Self` checkbox in the node UI
- the `Target` restore was done as a real enum widget only, without also duplicating the same value as a raw visible string field on the node
- runtime realtime network routing was rechecked and still maps the node target values to `Others`, `Host`, and `All`
- `Game::QuitGame` now exists as a real node on desktop and Android
- desktop runtime/export harness now watches `project.quit_requested` and cleanly exits the running packaged game
- Android runtime now watches the same quit flag and closes the activity when `Game::QuitGame` is executed
- blueprint graph wheel-zoom now works again while hovering normal graph content instead of only empty canvas space
- graph wheel-zoom is intentionally blocked while hovering overlay UI such as the node-search popup, so scrolling the search list no longer zooms the blueprint behind it
- Android package export from a built engine executable is now self-contained
- `build.rs` now embeds the full `android-template` contents into the engine executable at compile time
- Android export now restores that embedded template into the temporary Gradle workspace before packaging
- the older on-disk `android-template` lookup was kept as a fallback, but a freshly rebuilt engine `.exe` no longer needs the template folder next to it for normal Android packaging
- a later Android compile fix corrected the `Game::QuitGame` close path so the embedded runtime closes through the view context's `Activity` instead of calling `finish()` from the runtime view itself

June 17 typed-array / JSON-builder follow-up added after that:

- `PinType` and runtime values now support first-class typed arrays such as `Array<Number>`, `Array<String>`, `Array<Vector>`, `Array<Boolean>`, and `Array<Color>`
- array defaults are now editable inline in local variables, global variables, and exposed actor variables through the shared recursive runtime-value editor
- the temporary separate `+ Add Array` button was removed again; the intended workflow is now `+ Add Variable` followed by changing the type to an array
- `GetVariable` and `GetGlobalVar` now correctly fall back to typed variable defaults for arrays as well as scalar values
- unconnected array pins now fall back to empty arrays in the runtime instead of `None`
- a dedicated `Array` node library now exists with typed `Clear`, `Add`, `Insert`, `RemoveAt`, `Get`, `Set`, `Length`, `Contains`, `Find`, `ToJson`, and `FromJson` nodes for each supported scalar element type
- array mutator nodes were intentionally redesigned as flow-style state-changer nodes with no array output, so blueprints stay tidy instead of forcing `Get -> mutate -> Set` chains for every change
- the runtime now performs array write-back automatically when those mutator nodes operate on arrays sourced from variable getters, including routed array values
- `For Each Array<...>` now exists as a real flow node family on desktop and Android
- array getter output pins now use the inner element color rather than a special array color, show the label `Array` instead of `Val`, and render with a small grid-style array glyph
- drag-opened node search is now type-aware for value pins, so it only shows nodes/macros that can actually accept the dragged pin type
- that smart-search filtering was later tightened to keep flow-drag macro discovery working too, fixing the one confirmed regression found during the regression review
- the JSON builder now also supports `JsonSetColor`, `JsonGetColor`, and typed `JsonSetArray<...>` / `JsonGetArray<...>` nodes
- Android runtime/export parity was extended for the same array, `ForEachArray`, array-JSON, and JSON-color support in `MainActivity.java`

June 18 graph/UI follow-up added after that:

- `UI::ListBrowser` now exists as a dedicated general-purpose string-array HUD widget on desktop and Android
- `UI::ListBrowser` supports `Title`, `Items`, `InitialSelectedIndex`, persistent selected-item state, scroll handling, `SelectedIndex`, `SelectedItem`, `Out`, and `OnSelectionChanged`
- `InitialSelectedIndex` is intentionally a one-time seed for that widget instance and does not re-force selection every frame
- desktop `UI::ListBrowser` flow was regression-checked against Android and corrected so `Out` continues normally while selection changes fire separately, matching the established `RoomBrowser` pattern
- `Flow::DoOnce` now persists correctly across frames again and no longer keeps retriggering after it has already fired once
- `Flow::Gate` no longer crashes the engine in the reported close-loop usage and now only emits `Out` when triggered from `In` while open
- route nodes in the graph editor now use the revised capsule presentation and easier wire-split placement behavior the user validated during this session
- comment boxes in the graph editor now support a persisted header-brightness setting, copy/paste of both color and brightness, an opaque header bar, and a zoomed-out title bubble that obeys the same on-screen title-anchor visibility rule as the normal inline title
- `Implementation.md` was deleted after the array rollout was confirmed complete and should no longer be referenced as an active plan file

Later June 18 graph/data cleanup follow-up added after that:

- `Array::ToJson<...>` and `Array::FromJson<...>` were intentionally removed from the array library on desktop and Android so array JSON conversion now lives only in the `JSON::JsonSetArray<...>` / `JSON::JsonGetArray<...>` node family
- `Data::SaveString`, `LoadString`, `DoesSaveStringExist`, and `DeleteData` now resolve through a real `data` folder beside the project `.json` during editor playtests and beside the packaged `.exe` in exported Windows builds
- those lightweight data-string files now use the sanitized key directly with a `.DT` extension instead of the older `data_<key>.txt` naming
- Android runtime parity was updated to use the same `.DT` lightweight data-file naming inside the app `data` folder
- graph `G` snap now snaps selected nodes to their nearest graph cell while preventing the snapped selection from collapsing multiple selected nodes into the same occupied cell
- comment-box corner handles now keep free resize while dragging, then snap the dropped corner to the nearest graph grid point on release
- normal blueprint graph dragging for nodes and comment headers now advances in whole 50-unit graph cells by default instead of freeform sub-grid motion

Pushed checkpoint:

- Engine repo commit on `main`: `55d6a03` - `Add HUD scaling and world presentation updates`
- Engine repo commit on `main`: `f61c170` - `Add landscape orientation and Android packaging fixes`
- Engine repo commit on `main`: `c6fd566` - `Add Android hold-button HUD input parity`
- Engine repo commit on `main`: `8e773f0` - `Refine Android HUD anchors and touch routing`
- Engine repo commit on `main`: `f80084a` - `Add turnbased engine support and data string nodes`
- Engine repo commit on `main`: `f88b6cf` - `Restore event node controls and add quit game support`
- Engine repo commit on `main`: `a017f09` - `Add typed arrays and JSON builder support`
- Engine repo commit on `main`: `0292495` - `Add list browser and graph editor refinements`

Accepted current outcomes:

- Android `Linetrace` is corrected again
- the missing shared baked-world collision-source path is now implemented on Android
- particle `DieOnCollision` should be treated as fixed for now unless fresh device evidence clearly reopens it

## Resolved Conclusion

The key June 14 conclusion was correct and directly informed the fix:

1. Windows reads both `Linetrace` and particle `DieOnCollision` from the baked runtime `World` buffer.
2. Windows particle `DieOnCollision` is world-only, not actor-overlap based.
3. The earlier Android mismatch was architectural, not graph-level.
4. The correct Android fix was to stop approximating those queries through separate helper paths and instead give Android one shared baked world collision source.

That fix is now in place.

## Current Status

- The earlier missing-shared-world-source gap is no longer open.
- Android now has the shared baked-world collision path that the June 14 handoff called for.
- Do not reopen the older "particles vs `Linetrace` need different world sources" theory unless new testing clearly proves a remaining mismatch.
- HUD anchors, center-edge anchors, `UI::Text`, and HUD scaling are now implemented in the current workspace.
- The Android-side `UI::Text` regression is fixed in the current workspace and included in the pushed engine commit.
- Integer-fill world presentation plus separate HUD canvas sizing are now part of the current engine state.
- Native project orientation selection is now part of the current engine state.
- Android export/runtime now supports true portrait vs landscape app orientation without relying on live rotation handling.
- Android export now gives each packaged game its own install identity through a generated unique `applicationId`.
- The editor package dialog now re-syncs its per-project state when another game is opened in the same editor session.
- `UI::ButtonHold` is now part of the current engine state on desktop and Android.
- `UI::Thumbstick` is now part of the current engine state on desktop and Android.
- the lost inline name fields for `OnNetworkEvent` and `OnAnimEvent` are restored in the current editor state.
- the lost `Raise Network Event` target enum plus `Include Self` checkbox are restored in the current editor state.
- Android anchored HUD widgets now use real screen-space anchoring for the final overlay pass instead of staying tied to the intermediate HUD buffer.
- Android projects now expose the `Android Two-Finger World Pan` setting for choosing between world-pan behavior and twin-stick-style HUD/gameplay multi-touch routing.
- `Utils::IsAndroid` is now part of the current engine state for runtime blueprint branching.
- desktop HUD widgets no longer leak through the world draw pass because HUD draw commands are now separated from generic world draw commands.
- projects now expose a `Show Network Status Messages` setting that gates the built-in engine network popups.
- Android angle interpolation now matches desktop shortest-path behavior for `AngleInterpTo` and `AngleInterpToConstant`.
- the first full turnbased gameplay stack is now part of the engine on desktop and Android.
- later June 18 lifecycle / presentation follow-up work added after that:
- the level editor now exposes a visible `Center Scene` button that resets the scene camera to `(0, 0)` using the same path as the existing `Home` shortcut
- editor and in-game background colors are now global project settings instead of scene-specific values:
- `Editor Settings -> editor_background_color`
- `Engine Settings -> game_background_color`
- desktop and Android now both use the project-level game background color during play
- desktop debug traces/linetrace drawing are now clipped to the presented world viewport instead of leaking into the letterboxed side bars
- Android debug trace drawing now clips to the same world viewport rectangle for parity
- desktop screen flash now only affects the presented world viewport instead of the entire framebuffer
- Android screen flash now only affects the presented world viewport instead of the full screen
- turnbased `SubmitTurn` now supports a `ResultingStatus` input, but the backend only accepts `completed` as a lifecycle change
- `DeleteGame` now exists as a first-class turnbased management node on desktop and Android
- `OnTurnBasedGameDeleted` now exists as a real event on desktop and Android
- Android runtime/template parity was synced for `DeleteGame`, `ResultingStatus`, and `OnTurnBasedGameDeleted`
- the outer FriendBox API now treats reinvites as replacements only when the host, game type, and pending invitee group match exactly
- if even one invited player differs, the API now treats that as a separate new pending invited game
- declining a pending invite now removes the associated pending invited game on the backend so incomplete required-player groups cannot linger
- submitting the finishing turn with `resulting_status = "completed"` now removes the game and related invite rows on the backend
- the outer FriendBox API also now runs a stale-game cleanup loop that removes games untouched for 14 days
- turnbased account auth, profile, player search, invite flow, game load/start flow, and turn submission nodes now exist in the current engine state.
- the turnbased event set is now cleaned up to one event per async result rather than duplicate alias events.
- `OnTurnBasedGameLoaded` now carries `GameJson`, `ParticipantsJson`, and `TurnsJson` together.
- lightweight string persistence now exists through `Data::SaveString`, `Data::LoadString`, `Data::DoesSaveStringExist`, and `Data::DeleteData`.
- lightweight Windows editor/export data-string persistence now lives in a real per-game `data` folder instead of the engine working directory, and uses `.DT` files keyed by the sanitized save key
- the Android runtime packaging regression caused by checked JSON parsing in `OnTurnBasedGameLoaded` is fixed in the current workspace.
- `Game::QuitGame` now exists on desktop and Android in the current engine state.
- Android packaging from a rebuilt engine executable is now self-contained through an embedded `android-template` bundle.
- first-class typed arrays are now part of the current engine state across local variables, global variables, actor variables, node pins, runtime values, and Android export/runtime handling.
- the dedicated `Array::*` node family is now part of the current engine state.
- `For Each Array<...>` is now part of the current engine state on desktop and Android.
- drag-opened smart search is now type-aware for value pins and should only offer compatible variable/data node choices.
- the JSON builder now includes color getters/setters and typed array getters/setters in the current engine state on desktop and Android.
- the old array-library JSON conversion path is no longer part of the current engine state; array JSON conversion should go through `JSON::JsonSetArray<...>` / `JSON::JsonGetArray<...>` only
- `UI::ListBrowser` is now part of the current engine state on desktop and Android.
- `Flow::DoOnce` and `Flow::Gate` were rechecked and are now in sync again between desktop and Android for the regression cases exercised this session.
- the recent graph comment-box and route-node refinements are editor-only and do not require Android runtime parity.
- blueprint graph node dragging and comment-header dragging now move in whole graph-grid cells by default, `G` still exists as a separate explicit snap action, and comment-corner drop snap is now part of the current editor behavior
- note for future Android follow-up: the new `UI::TextInput` `Password Mode` editor option exists, but the Android runtime still needs an explicit masking/input-type pass if full device parity for masked fields becomes important later.

## Current Documentation State

The Android audit and matrix now reflect the accepted current state:

- `Linetrace` is fixed again
- the shared baked-world collision source exists on Android
- particle `DieOnCollision` is considered fixed for now
- `UI::Text` exists in the HUD path and is fixed on Android
- anchored HUD widgets plus widget-scale support are part of the current runtime
- world presentation now includes integer fill / overscan support with separate world-vs-HUD sizing
- native landscape orientation support now exists as a real project/export/runtime path
- Android packaging now uses unique per-game app IDs
- `UI::ButtonHold` and the Android HUD/touch overlay pass are now part of the documented runtime state
- `UI::Thumbstick`, `Utils::IsAndroid`, and the network-status popup setting are now part of the documented runtime state
- Android anchored HUD widgets now use real screen-space anchoring on device
- Android now has a documented project setting for two-finger world pan vs twin-stick-style HUD/gameplay touch routing
- Android shortest-path angle interpolation parity for `AngleInterpTo` and `AngleInterpToConstant` is now part of the documented runtime state
- the earlier "camera offset/jitter after the landscape work" report should not be treated as an engine regression without new evidence, because the confirmed cause was two competing user-side `SetCameraPosition` calls
- the Android docs should now also reflect the new `Data::*` string-persistence nodes and the first turnbased node/runtime set
- the Android docs now already reflect that array JSON conversion lives in the JSON node family rather than `Array::ToJson` / `Array::FromJson`
- the Android docs now already reflect the `.DT` lightweight data-file naming parity in the runtime path
- the Android docs should also call out that `OnTurnBasedGameLoaded` now emits `ParticipantsJson` and `TurnsJson` directly on the event
- the Android docs should keep the packaging fix note that `GAME_LOADED` JSON parsing is now wrapped safely for APK compilation
- the Android docs should now also reflect that `Game::QuitGame` exists on Android
- the Android docs should now also reflect that Android packaging is self-contained from a rebuilt engine executable because `android-template` is embedded into the editor binary
- the Android docs should mention the current known gap that `UI::TextInput` password masking is not yet explicitly implemented on the Android runtime path
- the Android docs should now also reflect first-class typed arrays, the new `Array::*` node family, `For Each Array<...>`, and Android array write-back behavior for variable-backed array mutator flows
- the Android docs should now also reflect `JsonSetColor`, `JsonGetColor`, and typed `JsonSetArray<...>` / `JsonGetArray<...>` support
- the Android docs now already reflect the current `UI::ListBrowser` runtime support and the one-time `InitialSelectedIndex` behavior, so no extra parity doc gap remains there after the regression fix
## Array Variables Follow-Up

The array rollout that previously lived in `Implementation.md` is now implemented in the current engine state:

- arrays are now first-class typed values in `PinType`, `RuntimeValueData`, runtime VM paths, and Android export/runtime parity
- local variables, global variables, and actor variables can now all use typed arrays with inline default editing
- the engine kept the simpler UX the user preferred: only `+ Add Variable` remains, and the variable can then be switched to an array type
- array getter nodes now present as proper array outputs instead of looking like flow pins
- array mutators are now flow-style state-changer nodes with no array output, matching the intended Unreal-style graph tidiness more closely
- drag-opened smart search is now type-aware for value pins, including array-aware filtering
- JSON builder support now covers array types and color so the array workflow can move cleanly into/out of JSON payloads
- `Implementation.md` has now been removed because that rollout is finished; use the committed engine code plus the Android audit/matrix as the current source of truth instead

Read first next session:

- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_NODE_LIBRARY_AUDIT.md`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_VM_SUPPORT_MATRIX.md`

## Next Session Start

Start in:

```powershell
cd "D:\Studio\CodexFarm\FriendBox\FriendBox Engine"
git status --short
git log --oneline -8
```

Then continue from the resolved state above instead of re-opening the old missing-shared-world-source theory unless fresh evidence requires it.

## Next Topic

Likely next clean-chat follow-ups after this handoff:

- broader device verification of the new Android HUD overlay/touch-routing behavior across a few different phones and aspect ratios
- broader device verification of the new `UI::Thumbstick` feel and degrees output in real twin-stick gameplay
- checking whether any remaining anchored HUD widgets or editor previews still assume buffer-space instead of final screen-space on Android
- deciding whether the project orientation setting should eventually grow beyond `Portrait` / `Landscape` into an optional `Auto` mode
- deciding whether the turnbased node surface should grow first-class friend/social nodes later, or remain centered on player search plus invite/game flow for now
- if Android login/register UX matters soon, consider finishing full Android `UI::TextInput` password-mode parity so device dialogs and rendered HUD text both mask password fields
- if array follow-up continues, the likely next cleanup topics are broader array workflow verification, any remaining edge cases in array write-back from more exotic graph chains, and wider runtime testing of typed array JSON round-tripping on Android
- if graph/editor follow-up continues, the likely next clean topics are broader manual regression testing of the new whole-cell graph drag rule, the `G` snap overlap-protection behavior for larger multi-selections, comment-corner drop snap behavior from all four handles, and any additional small UX polish around graph comments or flow wiring
