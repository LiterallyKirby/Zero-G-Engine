import { self } from "./context";

export function init(): void {
	console.log("Setting initial position");
	self().transform.position.x = -2.0;
	self().transform.position.y = 1.0;

	console.log(
		"Position set - x: " +
		self().transform.position.x.toString() +
		" y: " +
		self().transform.position.y.toString()
	);
}

export function update(dt: f32): void {
	const oldX = self().transform.position.x;
	const oldY = self().transform.position.y;

	self().transform.position.x += dt * 1.01;
	self().transform.position.y += dt * 1.01;

	const newX = self().transform.position.x;
	const newY = self().transform.position.y;


	console.log(
		"Update - oldX: " + oldX.toString() +
		" newX: " + newX.toString() +
		" oldY: " + oldY.toString() +
		" newY: " + newY.toString()
	);
}
