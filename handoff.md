# FriendBox Handoff

This handoff reflects the current engine state as of July 2, 2026. It is a clean status snapshot only. Forward-looking suggestions and "next topic" planning have been removed.

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

- Android and Windows are currently aligned for the major runtime parity areas completed so far: HUD/world presentation, UI input ownership, turnbased, arrays, JSON helpers, lightweight data persistence, Android export/runtime packaging, generic joystick input support, and the current particle feature set.
- The engine runtime is now on the GPU-driven renderer/compositor path for world presentation, sprite batching, lighting, and final screen composition.
- HUD/world presentation supports separate world-vs-HUD sizing, integer fit and integer fill, overscan-style world framing, screen-space anchor handling, and correct Android overlay placement.
- Desktop display settings now distinguish authored HUD/UI resolution from desktop output resolution. Window size can be larger than the HUD/UI canvas while the HUD/UI canvas remains capped to its authored size.
- Desktop display settings now include a display mode split between `Windowed` and `Fullscreen`.
- Fullscreen / large-window presentation now preserves pixel sharpness while allowing a low-resolution world and HUD to scale cleanly to a 1920x1080-class output.
- The runtime has a dedicated `OnUiTick` event phase that runs before gameplay input/events and before normal actor `OnUpdate`.
- A single blueprint can use both `OnUiTick` and `OnUpdate` in the same asset, so UI logic and gameplay logic can coexist in one blueprint.
- UI input capture exists at the engine core level for pointer, wheel, keyboard, and UI-phase ownership. Gameplay events such as `OnTouchPressed`, `OnTouchReleased`, `OnTouchedActor`, and left mouse button input are blocked when UI has already claimed that same-frame input.
- Interactive UI nodes participate in capture correctly across the current desktop/runtime pass: `UI::Button`, `UI::ButtonHold`, `UI::Thumbstick`, `UI::TextInput`, `UI::RoomBrowser`, `UI::GamesBrowser`, `UI::ListBrowser`, and `UI::ItemBrowser`.
- Browser-style UI containers are intended to claim their full visible panel area, not only their active rows/items. `UI::ItemBrowser` matches that rule, including its title/header strip.
- `UI::ItemBrowser` is present in both shared runtime and generated/exported runtime, with horizontal or vertical scrolling, optional item titles and values, per-text scale/color controls, selection color, per-text Y offsets, slot sizing, slot spacing, and `MaxItemsPerLane` wrapping for inventory-style grids.
- UI sprite slot rendering uses a dedicated `DrawCommand::SpriteRect` path with nearest-neighbor style scaled redraw instead of placing scene sprite actors into the level.
- Generic USB joystick / arcade encoder support is now integrated through SDL on desktop, while `gilrs` remains the path for standardized gamepads.
- Input mapping now supports keyboard, standardized gamepad, joystick button, joystick axis threshold, and joystick hat bindings, with project-level editing and Android parity.
- Android packaging is self-contained from the built engine executable through the embedded `android-template` path.
- Desktop packaging now builds SDL with static linking enabled, so packaged Windows exports no longer require shipping a separate `SDL2.dll`.
- Turnbased gameplay support remains present in the engine/runtime and intentionally separate from realtime networking.
- Typed arrays are first-class across pins, variables, runtime values, JSON helpers, and Android export/runtime support.
- Lightweight string persistence uses the per-project `data` folder with `.DT` files on desktop and matching naming on Android.
- Actor update/runtime performance has received a focused optimization pass covering timer management, script dispatch caching, collision broad-phase reuse, and rotated actor bounds correctness.

## Current Rendering / Smooth Movement State

- GPU presentation is active through the current runtime path, with GPU-side world composition, GPU sprite batching, GPU lighting, and GPU final presentation.
- Scene data now includes main-layer render-mode support through `main_render_mode`, matching the existing per-layer `render_mode` model used by background and foreground layers.
- The level editor exposes a main-layer settings section with smooth-movement opt-in, alongside the existing per-layer settings model for background and foreground layers.
- Smooth parallax / smooth main-layer movement is currently implemented through the pre-upscale filtered presentation path:
  - it is visually smooth
  - it is intentionally blurry
  - it does not currently satisfy the desired “smooth without blur” requirement
- A later experimental branch tried high-resolution nearest-sampled smooth targets to remove blur while keeping smoothness, but that path introduced regressions, including black sprite behavior, and was reverted.
- The engine is therefore back on the last stable pre-upscale smooth implementation.
- The current agreed path forward is documented in `implementation.md` and is now focused only on option 3:
  - keep the current GPU renderer
  - add a custom pixel-aware smooth presentation shader
  - target a result that is smoother than nearest sampling while remaining crisper than full linear filtering

## Particle / Rendering State

- `Particles::Spawn` now supports directional spawning through `Direction`, `MinDistance`, and `MaxDistance`.
- `Particles::Spawn` now supports optional attraction through `UseAttraction`, `Attraction`, `AttractionStrength`, and `SlowdownDistance`.
- `Particles::Spawn` now exposes a render-layer dropdown widget with selectable render bands:
  - `World Default`
  - `World Behind Actors`
  - `World Front`
  - `UI Behind Actors`
  - `UI Front`
- Particle render-layer selection is visual placement only. It is separate from collision behavior.
- Particle `DieOnCollision` and `Bounce` are back to world/tilemap collision only. They do not currently react to actors.
- Particle lights work across all world render bands on desktop and Android.
- Desktop and Android are currently aligned for the particle spawn inputs, attraction behavior, render-layer behavior, world-only collision behavior, and light rendering behavior.

## Current Editor State

- Content browser plain click is selection-only.
- Explicit tab clicks in the top bar force the requested editor mode instead of getting stuck in Blueprint mode.
- If a sprite asset has blueprint nodes, content-browser opening can still auto-select the Blueprint view.
- Clicking the `Sprite` tab correctly opens the sprite editor for the selected sprite asset.
- Clicking the `Blueprint` tab correctly opens the blueprint graph when available.
- The content browser supports right click on empty visible space to open the create menu.
- The content browser now reflects compatible on-disk assets more directly, and deleting an asset from the editor removes it from disk as the source of truth.
- The scene view right sidebar can be fully scrolled when its controls exceed the visible vertical space.
- Foreground layers support a per-layer collision toggle, with foreground collision defaulting to off until enabled manually.
- Project settings now expose desktop `Display Mode` plus `Windowed Resolution` separately from the HUD/UI canvas resolution.
- The level editor includes a same-kind stacking prevention toggle and it now defaults on.
- Editor-side non-play rendering/perf work includes offscreen actor culling and improved heavy-overlap handling compared with the earlier 22 FPS regression scenario.
- The node library/editor surface exposes `UI::ItemBrowser` and the current particle/node/runtime additions with the full pin and widget set used by the runtime implementation.

## Blueprint Graph State

- Graph node dragging and comment-header dragging move in whole grid cells by default.
- `G` snap still exists as a separate explicit snap action with overlap protection for multi-selection cases.
- Comment-corner resize handles snap to the graph grid on release.
- Route-node and comment-box UX refinements from the earlier graph cleanup remain part of the current editor state.
- Sprite blueprint editing still removes the active sprite from `project.assets.sprites` during the edit pass.
- Self-visible sprite graph lookups are restored through the editor-side resolver path, avoiding the rejected full-library clone approach.
- Search-menu node spawning now uses the last mouse position before opening the menu, fixing the earlier offset spawn issue.
- `Physics::MoveAndSlide` uses a selectable mode dropdown instead of exposing raw mode strings to the user.
- `Physics::MoveSimple` is now present as a separate top-down movement node with the same flow outputs as `MoveAndSlide`.
- `MoveSimple` now uses `Speed` input instead of a direct `Velocity` vector. Movement direction is derived from the actor's current rotation/facing.
- `MoveSimple` includes `Bounce`, `BounceResetFrames`, `Momentum`, `TargetID`, and mode support, and bounce can rotate the actor to the reflected direction.
- The old broad `Actor::GetVariableByName` style node was replaced with the newer selector-driven variant; the current path uses widget-backed selection rather than the older raw-name multi-output node.

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

## Scene / Level Editing State

- Asset-browser selection now updates the level-editor brush correctly for placement.
- The current brush and side panel stay in sync with asset-browser sprite selection.
- Added foreground layers can optionally participate in collision when enabled per layer.
- Actor and tile placement can prevent stacking of the same kind at the same location.
- The editor-side active-layer UI and layer options currently reflect these new placement/collision rules.

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
- `Transform::DirectionFromDegrees`
- `Physics::MoveSimple`
- animation frame count node
- animation speed setter node
- animation frame setter node
- typed `Array::*` nodes
- typed JSON color/array helpers
- turnbased auth, profile, invite, game, load, submit, and delete nodes/events

## Input Surface Present

- keyboard input
- mouse input
- touch input
- `gilrs` standardized gamepad input
- SDL generic joystick input for desktop arcade/USB joystick-class devices
- project-level action mapping for keyboard, gamepad button, joystick button, joystick axis threshold, and joystick hat bindings

## Known Real Caveats

- Smooth movement currently requires accepting the filtered/blurred presentation tradeoff on the active pre-upscale path. The “smooth and crisp” problem is not solved yet.
- The reverted high-resolution smooth-target experiment should not be treated as active engine behavior anymore.
- `UI::TextInput` password mode exists in the editor, but Android-specific masking/input-type parity is still not called out here as complete.
- `OnDraw` still exists as a node/event surface, but it should not be treated as the primary phase for interactive UI input. Interactive UI should be driven from `OnUiTick`.
- The recent graph/editor changes are editor-side behavior updates; they do not change packaged gameplay runtime behavior by themselves.
- A regression audit was completed during the particle/render-layer work. One real regression was found and fixed: particle lights had briefly stopped contributing from some world render bands on desktop. That fix is now in.
- A later runtime regression audit found and fixed a rotated-actor broad-phase bug in shared physics bounds code. That bug was severe enough to collapse FPS for always-active rotated actors using collision events, and the fix is shared by desktop runtime, `MoveAndSlide`, `MoveSimple`, and exported/generated runtimes including Android.
- Timer handling is much cheaper than before, but runtime coverage is still mainly manual. `cargo check` passes, and Android/generated-source parity was spot-checked for the recent runtime fixes, but there is still no broad automated gameplay test coverage across editor/runtime/Android paths.

## Main Files Most Relevant To The Current State

- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\layout.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\level_editor.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\project_settings.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\node_widgets.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\sprite_editor.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\core.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\mod.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\graph\find_panel.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\editor\generated_source.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\data\input_map.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\data\project.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\game_loop.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\input.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\particles.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\physics.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\turnbased.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\render_pass.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\renderer.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\gpu_presenter.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\gpu_sprite_renderer.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\engine\gpu_light_renderer.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\flow.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\physics.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\particles.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\transform.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\array.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\data.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\turnbased.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\ui.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\src\libs\utils.rs`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\android-template\app\src\main\java\app\friendbox\exported\MainActivity.java`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_NODE_LIBRARY_AUDIT.md`
- `D:\Studio\CodexFarm\FriendBox\FriendBox Engine\ANDROID_VM_SUPPORT_MATRIX.md`
- `D:\Studio\CodexFarm\FriendBox\handoff.md`
- `D:\Studio\CodexFarm\FriendBox\implementation.md`
