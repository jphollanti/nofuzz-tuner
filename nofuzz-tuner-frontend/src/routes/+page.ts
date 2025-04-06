// since there's no dynamic data here, we can prerender
// it so that it gets served as a static asset in production
export const prerender = true;
export const ssr = false; // Disable server-side rendering for this route
import init, { YinPitchDetector } from '../lib/no_fuzz_tuner/pkg/nofuzz_tuner_lib.js';

export const load = () => {
    return {
        /* 
        We'll load the WASM module on the client side.
        This approach of loading WebAssembly modules on the client side is often the best way to 
        handle WASM in Svelte applications, especially when server-side rendering is involved. 
        */
    };
};