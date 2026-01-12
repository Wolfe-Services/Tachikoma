

export const index = 2;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/2.C-LnyAmw.js","_app/immutable/chunks/Bt7hakvE.js","_app/immutable/chunks/CjDKn7ff.js","_app/immutable/chunks/fMzpNvnw.js"];
export const stylesheets = ["_app/immutable/assets/2.DSIh4lOr.css"];
export const fonts = [];
