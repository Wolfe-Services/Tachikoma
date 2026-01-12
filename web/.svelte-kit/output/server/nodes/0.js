

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const imports = ["_app/immutable/nodes/0.DG4VJO2s.js","_app/immutable/chunks/Bt7hakvE.js","_app/immutable/chunks/CjDKn7ff.js"];
export const stylesheets = ["_app/immutable/assets/0.BqMN6I8Z.css"];
export const fonts = [];
