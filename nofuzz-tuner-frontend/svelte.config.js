import adapter from '@sveltejs/adapter-static';
import preprocess from 'svelte-preprocess';

export default {
  kit: {
    adapter: adapter({
      // default outDir is "build" unless you specify something else
      // like "out: 'build'"
      // or "out: '.svelte-kit/output'"
    })
  },
  preprocess: preprocess()
};