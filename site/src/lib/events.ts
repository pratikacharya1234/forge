import { writable } from 'svelte/store';

export const forgeEvents = writable([]);

export function emitForgeEvent(type, data = {}) {
  forgeEvents.update(events => [...events, { type, data, timestamp: Date.now() }]);
}
