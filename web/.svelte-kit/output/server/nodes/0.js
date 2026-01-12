

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const imports = ["_app/immutable/nodes/0.BJ7-RpI5.js","_app/immutable/chunks/C-NO07ga.js","_app/immutable/chunks/B-eSy4nV.js"];
export const stylesheets = ["_app/immutable/assets/0.BqMN6I8Z.css"];
export const fonts = [];
