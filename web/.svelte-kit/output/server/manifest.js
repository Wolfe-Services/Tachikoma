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
		client: {start:"_app/immutable/entry/start.CoFzMdw6.js",app:"_app/immutable/entry/app.CXhVXZuB.js",imports:["_app/immutable/entry/start.CoFzMdw6.js","_app/immutable/chunks/CqnVcip1.js","_app/immutable/chunks/Bt7hakvE.js","_app/immutable/chunks/fMzpNvnw.js","_app/immutable/entry/app.CXhVXZuB.js","_app/immutable/chunks/Bt7hakvE.js","_app/immutable/chunks/CjDKn7ff.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
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
