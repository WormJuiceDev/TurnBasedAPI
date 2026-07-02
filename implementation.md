# Option 3 Plan: Custom Smooth Presentation Shader

## Goal

Keep the current GPU renderer and pixel-art workflow, but replace the current smooth-layer presentation method with a custom GPU shader approach that:

- looks much smoother than strict nearest sampling
- stays much crisper than full filtered blur
- works for opt-in smooth background layers
- works for opt-in smooth foreground layers
- works for opt-in smooth main-layer presentation

This is the preferred path forward because:

- pure nearest keeps visible slow-speed pixel stepping
- pure filtering removes the stepping but becomes visibly blurry
- upscaled nearest targets did not remove the slow-speed stepping enough and introduced regressions

## What We Learned

The camera/interp path is already float-based.

The remaining visual issue is presentation:

- nearest sampling preserves crisp pixels, but slow subpixel motion still resolves as visible stepping
- linear filtering smooths that motion, but softens the art too much

So the real solution is not more camera math work.
The real solution is a smarter presentation shader.

## Core Idea

Instead of choosing between:

- nearest = crisp but stepped
- linear = smooth but blurry

we add a third presentation mode:

- custom subpixel presentation

The shader should sample the source texture in a way that preserves hard pixel structure as much as possible, while still distributing very small motion across neighboring screen pixels in a controlled way.

## Desired Behavior

For layers marked `SmoothParallax`:

- movement should look genuinely smoother at low speeds
- sprites and tiles should stay visually sharper than linear filtering
- the result should not smear like normal bilinear sampling

For the main layer when smooth mode is enabled:

- camera motion should use the same presentation logic
- world lighting should continue to apply correctly
- opt-in behavior should stay the same

For non-smooth layers:

- keep the current pixel-perfect nearest path
- do not change their visual behavior

## Recommended Shader Strategy

Use a custom presenter shader for smooth layers that:

1. samples the texture with nearest-style pixel anchoring
2. computes the subpixel phase from the fractional screen offset
3. blends only between immediate neighboring texels based on that phase
4. keeps blend weights tight and biased toward the dominant texel
5. optionally uses a tiny ordered-dither or thresholded coverage rule to reduce softness

This is not standard bilinear blur.
It is a controlled, pixel-aware blend.

## Practical Variants

### Variant A: Tight Neighbor Blend

Sample:

- current texel
- horizontal neighbor when x phase is active
- vertical neighbor when y phase is active
- diagonal neighbor only if both phases are strong

Then blend with a sharpened weighting curve instead of linear weights.

Expected result:

- smoother than nearest
- less blurry than bilinear
- simplest first implementation

### Variant B: Thresholded Pixel Coverage

Use the subpixel phase to decide when a neighboring texel begins contributing, but delay contribution until a threshold is crossed.

Expected result:

- crisper than Variant A
- still smoother than nearest
- can feel more “pixel stable”

### Variant C: Ordered Dither Blend

Use a small Bayer matrix or stable screen-space threshold so fractional motion is represented by controlled per-pixel coverage instead of uniform blur.

Expected result:

- visually sharper than bilinear
- may look more “retro”
- needs careful testing to avoid shimmer

## Recommended Implementation Order

### Step 1

Re-stabilize the renderer on the current pre-upscale smooth path.

Meaning:

- keep the reverted version as the baseline
- no high-resolution smooth target experiment
- no mixed world-light/upscaled world regressions

### Step 2

Add a new presenter-layer mode flag, for example:

- `Nearest`
- `Linear`
- `PixelSmooth`

Only `PixelSmooth` should use the custom shader logic.

### Step 3

Implement Variant A first inside the presenter shader.

Reason:

- lowest risk
- easiest to compare against current nearest and linear modes
- likely enough to prove the direction quickly

### Step 4

Wire `SmoothParallax` layers to use `PixelSmooth` instead of `Linear`.

Apply to:

- smooth background layers
- smooth foreground layers
- smooth main layer when enabled

### Step 5

Tune the weighting curve.

Important controls may include:

- blend sharpness
- neighbor contribution clamp
- optional diagonal contribution strength
- optional dither enable/amount

These should likely live as engine constants first, and only become user settings later if needed.

## Important Constraints

### Keep It GPU-Only

This path should stay entirely on the GPU presentation side.

Do not return to:

- CPU compositing
- CPU filtered scratch buffers
- CPU post-process blending

### Do Not Change Non-Smooth Layers

Only opt-in smooth layers and opt-in smooth main-layer presentation should use the new mode.

Normal pixel-perfect layers should remain untouched.

### Lighting Must Stay Correct

If the main layer uses the custom smooth presentation path:

- world lighting must still sample correctly
- no separate coordinate-space hacks
- no extra upscale target path unless absolutely necessary

The custom presentation shader should operate on the already rendered world layer, not rebuild lighting logic.

## Success Criteria

This option is successful if:

- slow camera/layer motion is visibly smoother than nearest
- art remains visibly sharper than linear filtering
- smooth main-layer mode also benefits
- no black sprite regressions
- no major FPS collapse
- no CPU fallback is reintroduced

## Non-Goals

This plan does not aim to achieve impossible “perfectly smooth and perfectly crisp at every subpixel” output on fixed display pixels.

Instead, the goal is:

- best perceptual balance
- clearly better than nearest
- clearly crisper than blur

## Next Session Starting Point

When continuing:

1. keep the reverted pre-upscale smooth implementation as baseline
2. add a new `PixelSmooth` presenter sampling mode
3. implement Variant A in the presenter shader first
4. test it on smooth BG, FG, and smooth main-layer presentation

