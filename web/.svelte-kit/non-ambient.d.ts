
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	export interface AppTypes {
		RouteId(): "/(auth)" | "/" | "/ai" | "/ai/history" | "/(auth)/login" | "/mission" | "/projects" | "/projects/[id]" | "/projects/[id]/reports" | "/projects/[id]/scans" | "/projects/[id]/targets" | "/settings" | "/settings/api-keys" | "/settings/appearance" | "/settings/general" | "/(auth)/setup" | "/specs" | "/tools" | "/tools/exploitation" | "/tools/reconnaissance" | "/tools/terminal";
		RouteParams(): {
			"/projects/[id]": { id: string };
			"/projects/[id]/reports": { id: string };
			"/projects/[id]/scans": { id: string };
			"/projects/[id]/targets": { id: string }
		};
		LayoutParams(): {
			"/(auth)": Record<string, never>;
			"/": { id?: string };
			"/ai": Record<string, never>;
			"/ai/history": Record<string, never>;
			"/(auth)/login": Record<string, never>;
			"/mission": Record<string, never>;
			"/projects": { id?: string };
			"/projects/[id]": { id: string };
			"/projects/[id]/reports": { id: string };
			"/projects/[id]/scans": { id: string };
			"/projects/[id]/targets": { id: string };
			"/settings": Record<string, never>;
			"/settings/api-keys": Record<string, never>;
			"/settings/appearance": Record<string, never>;
			"/settings/general": Record<string, never>;
			"/(auth)/setup": Record<string, never>;
			"/specs": Record<string, never>;
			"/tools": Record<string, never>;
			"/tools/exploitation": Record<string, never>;
			"/tools/reconnaissance": Record<string, never>;
			"/tools/terminal": Record<string, never>
		};
		Pathname(): "/" | "/ai" | "/ai/" | "/ai/history" | "/ai/history/" | "/login" | "/login/" | "/mission" | "/mission/" | "/projects" | "/projects/" | `/projects/${string}` & {} | `/projects/${string}/` & {} | `/projects/${string}/reports` & {} | `/projects/${string}/reports/` & {} | `/projects/${string}/scans` & {} | `/projects/${string}/scans/` & {} | `/projects/${string}/targets` & {} | `/projects/${string}/targets/` & {} | "/settings" | "/settings/" | "/settings/api-keys" | "/settings/api-keys/" | "/settings/appearance" | "/settings/appearance/" | "/settings/general" | "/settings/general/" | "/setup" | "/setup/" | "/specs" | "/specs/" | "/tools" | "/tools/" | "/tools/exploitation" | "/tools/exploitation/" | "/tools/reconnaissance" | "/tools/reconnaissance/" | "/tools/terminal" | "/tools/terminal/";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): "/favicon.png.placeholder" | string & {};
	}
}