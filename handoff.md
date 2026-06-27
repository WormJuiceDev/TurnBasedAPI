# FriendBox Handoff

This handoff reflects the current engine state as of June 27, 2026. It is a clean status snapshot only. Forward-looking suggestions and "next topic" planning have been removed.

## Repo Context

- Outer workspace: `D:\Studio\CodexFarm\FriendBox`
- Engine repo: `D:\Studio\CodexFarm\FriendBox\FriendBox Engine`
- Engine git remote: `https://github.com/WormJuiceDev/FriendBoxEngine`
- Outer API git repo: `https://github.com/WormJuiceDev/TurnBasedAPI`
- Use the engine repo for engine/editor `git status`, commits, and pushes
- Use the outer repo for backend/API commits and deployment staging
- Leave `FriendBox Engine/editor_config.json` alone unless explicitly asked
- Ignore `FriendBox Engine/saves/`
- Ignore `FriendBox Engine/data/`

## Current Engine State

- Android and Windows are currently aligned for the large runtime parity work completed across HUD, turnbased, arrays, JSON helpers, lightweight data persistence, and Android packaging/export behavior.
- The Android runtime uses the shared baked world collision source for `Linetrace` and particle `DieOnCollision`, matching the intended Windows-side architecture.
- HUD/world presentation supports separate world-vs-HUD sizing, integer fill / overscan, screen-space anchor handling, and correct Android overlay placement.
- The runtime now has a dedicated `OnUiTick` event phase that runs before gameplay input/events and before normal actor `OnUpdate`.
- A single blueprint can use both `OnUiTick` and `OnUpdate` in the same asset, so UI logic and gameplay logic can coexist without splitting the blueprint.
- UI input capture now exists at the engine core level for pointer, wheel, and keyboard ownership during the UI phase.
- Gameplay events such as `OnTouchPressed`, `OnTouchReleased`, `OnTouchedActor`, and left mouse button input are now blocked when UI has already claimed that same-frame input.
- The UI runtime now includes `UI::ItemBrowser`, a sprite-driven selectable browser/grid that mirrors into the generated/exported runtime path.
- `UI::ItemBrowser` supports horizontal or vertical scrolling, optional item titles and values, per-text scale/color controls, selection color, per-text Y offsets, slot sizing, slot spacing, and `MaxItemsPerLane` wrapping for inventory-style grids.
- UI sprite slot rendering now uses a dedicated `DrawCommand::SpriteRect` path with nearest-neighbor style scaled redraw instead of placing scene sprite actors into the level.
- Interactive UI nodes now participate in capture correctly across the current desktop/runtime pass: `UI::Button`, `UI::ButtonHold`, `UI::Thumbstick`, `UI::TextInput`, `UI::RoomBrowser`, `UI::GamesBrowser`, `UI::ListBrowser`, and `UI::ItemBrowser`.
- Browser-style UI containers are intended to claim their full visible panel area, not only their active rows/items. `UI::ItemBrowser` was updated to match that rule, including its title/header strip.
- Mobile orientation is a real project/export/runtime setting with portrait and landscape support.
- Android packaging is self-contained from the built engine executable through the embedded `android-template` path.
- Turnbased gameplay support is present in the engine/runtime and remains intentionally separate from realtime networking.
- Typed arrays are first-class across pins, variables, runtime values, JSON helpers, and Android export/runtime support.
- Lightweight string persistence uses the per-project `data` folder with `.DT` files on desktop and matching naming on Android.

## Current Editor State

- Content browser plain click is selection-only.
- Explicit tab clicks in the top bar now force the requested editor mode instead of getting stuck in Blueprint mode.
- If a sprite asset has blueprint nodes, content-browser opening can still auto-select the Blueprint view.
- Clicking the `Sprite` tab now correctly opens the sprite editor for the selected sprite asset.
- Clicking the `Blueprint` tab now correctly opens the blueprint graph when available.
- The content browser supports right click on empty visible space to open the create menu.
- The node library/editor surface now exposes `UI::ItemBrowser` with the full pin set used by the current runtime implementation.

## Blueprint Graph State

- Graph node dragging and comment-header dragging move in whole grid cells by default.
- `G` snap still exists as a separate explicit snap action with overlap protection for multi-selection cases.
- Comment-corner resize handles snap to the graph grid on release.
- Route-node and comment-box UX refinements from the earlier graph cleanup remain part of the current editor state.
- Sprite blueprint editing still removes the active sprite from `project.assets.sprites` during the edit pass.
- Self-visible sprite graph lookups are restored through the editor-side resolver path, avoiding the rejected full-library clone approach.

## Find In Graph

- `Ctrl+F` opens a dedicated `Find in Graph` panel in the blueprint editor.
- Search results are built on query/revision change, not every frame.
- The search can find matching nodes/comments across the current blueprint graph context.
- Clicking a result switches to the correct graph when needed, selects the target, and recenters the view.
- Closing the panel and moving to another blueprint now resets the cached find state so stale results do not carry over.
- The find panel opens centered in the editor the first time it is shown.

## Sprite Editor State

- The sprite editor has outline-only `Rect` and `Circle` draw tools.
- Those shape tools exist in both normal paint mode and collision-mask editing mode.
- `R` selects `Rect` and `C` selects `Circle`.
- Shape preview uses the same cell/raster logic as the final applied result instead of a misleading vector-style preview.
- Circle drawing now works from center outward.
- Even-sized sprites such as `32x32` can now produce properly centered left/right symmetric circles.
- Collision-mask editing has a dedicated `Clear Mask` button for the current frame.

## Runtime / Node Surface Present

- `OnUiTick`
- `UI::ButtonHold`
- `UI::Button`
- `UI::Thumbstick`
- `UI::TextInput`
- `UI::Text`
- `UI::RoomBrowser`
- `UI::GamesBrowser`
- `UI::ListBrowser`
- `UI::ItemBrowser`
- `Utils::IsAndroid`
- `Game::QuitGame`
- `Data::SaveString`
- `Data::LoadString`
- `Data::DoesSaveStringExist`
- `Data::DeleteData`
- typed `Array::*` nodes
- typed JSON color/array helpers
- turnbased auth, profile, invite, game, load, submit, and delete nodes/events

## Known Real Caveats

- `UI::TextInput` password mode exists in the editor, but Android-specific masking/input-type parity is still not called out here as complete.
- `OnDraw` still exists as a node/event surface, but it should not be treated as the primary phase for interactive UI input. Interactive UI should be driven from `OnUiTick`.
- The recent graph/editor changes are editor-side behavior updates; they do not change packaged gameplay runtime behavior by themselves.
- A regression audit was completed after the `UI::ItemBrowser` work: no code-level regressions were found in shared runtime/editor paths, `cargo check` passes, and `cargo test` passes, but there are currently no automated tests so runtime coverage is still manual.

## Main Files Most Relevant To The Current State

- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\layout.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\sprite_editor.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\core.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\mod.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\find_panel.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\generated_source.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\data\project.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\game_loop.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\turnbased.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\render_pass.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\array.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\data.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\turnbased.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\ui.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\utils.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\android-template\app\src\main\java\app\friendbox\exported\MainActivity.java`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_NODE_LIBRARY_AUDIT.md`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_VM_SUPPORT_MATRIX.md`
- `D:\Studio\CodexFarm\FriendBox\handoff.md`
