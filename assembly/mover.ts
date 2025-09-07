// assembly/mover.ts

import { self } from "./context";

export function init(): void {
  // Set initial position
  self().positionX = -2.0;
}

export function update(dt: f32): void {
  // Move entity along X
   self().positionX += dt * 1.01;
}
