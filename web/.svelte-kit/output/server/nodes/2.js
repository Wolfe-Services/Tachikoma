

export const index = 2;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_page.svelte.js')).default;
export const imports = ["_app/immutable/nodes/2.BWh6U2JO.js","_app/immutable/chunks/DYjCt7Qj.js","_app/immutable/chunks/BI_ABXT4.js"];
export const stylesheets = ["_app/immutable/assets/2.B-6-6rYd.css"];
export const fonts = [];
