import{c as _,s as g,a as y,v as f,m as h}from"./vendor.js";import"./environment.js";let k={};function E(n){}function O(n){k=n}let x=null;function S(n){x=n}function U(n){}const w=_((n,t,e,v)=>{let{stores:o}=t,{page:r}=t,{constructors:s}=t,{components:a=[]}=t,{form:d}=t,{data_0:c=null}=t,{data_1:m=null}=t;g("__svelte__",o),y(o.page.notify),t.stores===void 0&&e.stores&&o!==void 0&&e.stores(o),t.page===void 0&&e.page&&r!==void 0&&e.page(r),t.constructors===void 0&&e.constructors&&s!==void 0&&e.constructors(s),t.components===void 0&&e.components&&a!==void 0&&e.components(a),t.form===void 0&&e.form&&d!==void 0&&e.form(d),t.data_0===void 0&&e.data_0&&c!==void 0&&e.data_0(c),t.data_1===void 0&&e.data_1&&m!==void 0&&e.data_1(m);let l,p,u=n.head;do l=!0,n.head=u,o.page.set(r),p=`  ${s[1]?`${f(s[0]||h,"svelte:component").$$render(n,{data:c,params:r.params,this:a[0]},{this:i=>{a[0]=i,l=!1}},{default:()=>`${f(s[1]||h,"svelte:component").$$render(n,{data:m,form:d,params:r.params,this:a[1]},{this:i=>{a[1]=i,l=!1}},{})}`})}`:`${f(s[0]||h,"svelte:component").$$render(n,{data:c,form:d,params:r.params,this:a[0]},{this:i=>{a[0]=i,l=!1}},{})}`} `;while(!l);return p}),q={app_template_contains_nonce:!1,async:!1,csp:{mode:"auto",directives:{"upgrade-insecure-requests":!1,"block-all-mixed-content":!1},reportOnly:{"upgrade-insecure-requests":!1,"block-all-mixed-content":!1}},csrf_check_origin:!0,csrf_trusted_origins:[],embedded:!1,env_public_prefix:"PUBLIC_",env_private_prefix:"",hash_routing:!1,hooks:null,preload_strategy:"modulepreload",root:w,service_worker:!1,service_worker_options:void 0,templates:{app:({head:n,body:t,assets:e,nonce:v,env:o})=>`<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="color-scheme" content="dark light" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';" />
    <link rel="icon" href="`+e+`/favicon.png" />
    `+n+`
  </head>
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">`+t+`</div>
  </body>
</html>`,error:({status:n,message:t})=>`<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<title>`+t+`</title>

		<style>
			body {
				--bg: white;
				--fg: #222;
				--divider: #ccc;
				background: var(--bg);
				color: var(--fg);
				font-family:
					system-ui,
					-apple-system,
					BlinkMacSystemFont,
					'Segoe UI',
					Roboto,
					Oxygen,
					Ubuntu,
					Cantarell,
					'Open Sans',
					'Helvetica Neue',
					sans-serif;
				display: flex;
				align-items: center;
				justify-content: center;
				height: 100vh;
				margin: 0;
			}

			.error {
				display: flex;
				align-items: center;
				max-width: 32rem;
				margin: 0 1rem;
			}

			.status {
				font-weight: 200;
				font-size: 3rem;
				line-height: 1;
				position: relative;
				top: -0.05rem;
			}

			.message {
				border-left: 1px solid var(--divider);
				padding: 0 0 0 1rem;
				margin: 0 0 0 1rem;
				min-height: 2.5rem;
				display: flex;
				align-items: center;
			}

			.message h1 {
				font-weight: 400;
				font-size: 1em;
				margin: 0;
			}

			@media (prefers-color-scheme: dark) {
				body {
					--bg: #222;
					--fg: #ddd;
					--divider: #666;
				}
			}
		</style>
	</head>
	<body>
		<div class="error">
			<span class="status">`+n+`</span>
			<div class="message">
				<h1>`+t+`</h1>
			</div>
		</div>
	</body>
</html>
`},version_hash:"1y1e5y5"};async function F(){return{handle:void 0,handleFetch:void 0,handleError:void 0,handleValidationError:void 0,init:void 0,reroute:void 0,transport:void 0}}export{O as a,S as b,U as c,F as g,q as o,k as p,x as r,E as s};
//# sourceMappingURL=internal.js.map
