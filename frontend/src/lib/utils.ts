import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

export function generateShortId(): string {
	const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
	let result = "";
	const randomValues = new Uint32Array(8);
	crypto.getRandomValues(randomValues);
	for (let i = 0; i < 8; i++) {
		result += chars[randomValues[i] % chars.length];
	}
	return result;
}
