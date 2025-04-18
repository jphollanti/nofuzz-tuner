<svelte:head>
	<!-- Primary SEO -->
	<title>Online Guitar Tuner - No-Fuzz, Browser-Based &amp; Free</title>
	<meta name="description" content="Tune your guitar fast with browser-based tuner. No ads, 100% free." />
	<link rel="canonical" href="https://www.nofuzz.app/" />

	<!-- Open Graph -->
	<meta property="og:type"        content="website" />
	<meta property="og:title"       content="Online Guitar Tuner - No-Fuzz, Browser-Based &amp; Free" />
	<meta property="og:description" content="Tune your guitar accurately with browser-based tuner. No ads." />
	<meta property="og:url"         content="https://www.nofuzz.app/" />
	<meta property="og:image"       content="https://www.nofuzz.app/cover.png" />

	<!-- Twitter Card -->
	<meta name="twitter:card"        content="summary_large_image" />
	<meta name="twitter:site"        content="@NoFuzzTuner" />
	<meta name="twitter:title"       content="Online Guitar Tuner - No-Fuzz, Browser-Based &amp; Free" />
	<meta name="twitter:description" content="Fast guitar tuner." />
	<meta name="twitter:image"       content="https://www.nofuzz.app/cover.png" />

	<!-- PWA / install prompts -->
	<link rel="manifest" href="/manifest.webmanifest" />
	<meta name="theme-color" content="#121212" />

	<!-- App‑object schema for richer search results -->
	<script type="application/ld+json">
	{
	  "@context": "https://schema.org",
	  "@type": "SoftwareApplication",
	  "name": "No-Fuzz Guitar Tuner",
	  "applicationCategory": "MusicApplication",
	  "operatingSystem": "Web",
	  "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
	  "featureList": [
	    "High-accuracy YIN pitch detection",
	    "No ads or tracking"
	  ],
	  "screenshot": "https://www.nofuzz.app/cover.png",
	  "url": "https://www.nofuzz.app/"
	}
	</script>
</svelte:head>

<script>
	import '../app.css';
</script>

<div class="app">
	<main>
		<slot />
	</main>

	<footer>
		<p></p>
	</footer>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 12px;
	}

	@media (min-width: 480px) {
		footer {
			padding: 12px 0;
		}
	}
</style>
