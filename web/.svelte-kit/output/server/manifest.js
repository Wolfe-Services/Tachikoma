export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["favicon.png.placeholder"]),
	mimeTypes: {},
	_: {
		client: {start:"_app/immutable/entry/start.B5wPzgk3.js",app:"_app/immutable/entry/app.D-rR0jEp.js",imports:["_app/immutable/entry/start.B5wPzgk3.js","_app/immutable/chunks/CUdn-OLj.js","_app/immutable/chunks/DYjCt7Qj.js","_app/immutable/entry/app.D-rR0jEp.js","_app/immutable/chunks/DYjCt7Qj.js","_app/immutable/chunks/BI_ABXT4.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
