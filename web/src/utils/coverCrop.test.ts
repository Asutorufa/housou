import { describe, expect, it } from "vitest";
import {
  applyCoverMorphAt,
  applyPanelMorphAt,
  computeCoverMorphFrame,
  computePanelMorphFrame,
  type MorphRect,
} from "./coverCrop";

function rect(
  left: number,
  top: number,
  width: number,
  height: number,
): MorphRect {
  return {
    left,
    top,
    right: left + width,
    bottom: top + height,
    width,
    height,
  };
}

describe("coverCrop morph geometry", () => {
  it("computes panel translate and rounded clip-path inset matching the card rect", () => {
    const modal = rect(200, 100, 800, 600);
    const card = rect(50, 200, 200, 380);

    const frame = computePanelMorphFrame(card, modal, 16, 24);

    expect(frame.dx).toBe(-450);
    expect(frame.dy).toBe(-10);
    expect(frame.scale).toBe(1);
    expect(frame.insetX).toBe(300);
    expect(frame.insetY).toBe(110);

    const el = document.createElement("div");
    applyPanelMorphAt(el, frame, 0);
    expect(el.style.transform).toBe("translate3d(-450.00px, -10.00px, 0)");
    expect(el.style.clipPath).toBe(
      "inset(110.00px 300.00px 110.00px 300.00px round 16.00px)",
    );

    applyPanelMorphAt(el, frame, 1);
    expect(el.style.transform).toBe("translate3d(0.00px, 0.00px, 0)");
    expect(el.style.clipPath).toBe(
      "inset(0.00px 0.00px 0.00px 0.00px round 24.00px)",
    );
  });

  it("computes exact cover transform and 3:4 crop clip-path relative to the modal panel", () => {
    const modal = rect(200, 100, 800, 600);
    const targetCover = rect(220, 150, 320, 480);
    const card = rect(50, 200, 200, 380);
    const poster = rect(50, 183.3333, 200, 300);
    const windowRect = rect(50, 200, 200, 266.6667);

    const panelFrame = computePanelMorphFrame(card, modal);
    const coverFrame = computeCoverMorphFrame(
      poster,
      windowRect,
      targetCover,
      modal,
      panelFrame,
    );

    expect(coverFrame.dx).toBeCloseTo(220, 2);
    expect(coverFrame.dy).toBeCloseTo(-46.67, 1);
    expect(coverFrame.scaleX).toBeCloseTo(0.625, 4);
    expect(coverFrame.scaleY).toBeCloseTo(0.625, 4);
    expect(coverFrame.clipTop).toBeCloseTo(5.556, 2);
    expect(coverFrame.clipBottom).toBeCloseTo(5.556, 2);
    expect(coverFrame.clipLeft).toBe(0);
    expect(coverFrame.clipRight).toBe(0);

    const img = document.createElement("img");
    applyCoverMorphAt(img, coverFrame, 0);
    expect(img.style.clipPath).toBe("inset(5.556% 0.000% 5.556% 0.000%)");
    applyCoverMorphAt(img, coverFrame, 1);
    expect(img.style.clipPath).toBe("inset(0.000% 0.000% 0.000% 0.000%)");
  });
});
