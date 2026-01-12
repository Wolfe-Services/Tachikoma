

export const index = 2;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/2.C_S7QwTv.js","_app/immutable/chunks/C-NO07ga.js","_app/immutable/chunks/B-eSy4nV.js","_app/immutable/chunks/jsLhJHwu.js"];
export const stylesheets = ["_app/immutable/assets/2.DSIh4lOr.css"];
export const fonts = [];
