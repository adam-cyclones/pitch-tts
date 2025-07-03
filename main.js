const puppeteer = require('puppeteer');
const fs = require('fs').promises;
const path = require('path');

async function runTTS(textToSynthesize, outputFilePath, voiceId = 'en_US-hfc_female-medium') {
    let browser = null;
    try {
        console.log('Launching headless browser...');
        browser = await puppeteer.launch({
            headless: true, // Use true for actual headless, or 'new' for new headless, false for debugging
            args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage', // Common in Docker/CI environments
                '--use-fake-ui-for-media-stream', // May not be needed but good for media access
                '--use-fake-device-for-media-stream',
                // The Piper TTS library uses Origin Private File System.
                // These flags might be necessary if there are issues with OPFS access in headless.
                // However, it often works out-of-the-box or requires specific user data dir.
                // For now, let's assume default OPFS access works.
            ]
        });
        const page = await browser.newPage();

        // Enable console logging from the page in the Node context
        page.on('console', msg => {
            const type = msg.type();
            const text = msg.text();
            // Filter out verbose logs if necessary, e.g. OPFS access logs
            if (text.includes('OPFS') && text.includes('Opening')) return; // Example filter
            console.log(`PAGE LOG [${type}]: ${text}`);
        });
        page.on('pageerror', error => {
            console.error('PAGE ERROR:', error.message);
        });
        page.on('requestfailed', request => {
            console.error(`REQUEST FAILED: ${request.url()} - ${request.failure().errorText}`);
        });
        // page.on('requestfinished', request => {
        //     if (request.response()?.status() !== 200) {
        //        console.log(`REQUEST FINISHED (non-200): ${request.url()} - Status: ${request.response()?.status()}`);
        //     }
        // });


        const htmlFilePath = path.resolve(__dirname, 'index.html');
        console.log(`Navigating to ${htmlFilePath}...`);
        try {
            await page.goto(`file://${htmlFilePath}`, { waitUntil: 'domcontentloaded', timeout: 30000 });
        } catch (e) {
            console.error(`Error navigating to page: ${e.message}`);
            throw e;
        }


        console.log(`Page navigation complete (DOM content loaded). Waiting for TTS functions to be available on page...`);

        // Wait for the piperTTS object and its methods to be defined
        await page.waitForFunction(
            'window.piperTTS && typeof window.piperTTS.initializeAndDownloadVoice === "function" && typeof window.piperTTS.synthesizeSpeech === "function"',
            { timeout: 30000 } // 30 seconds timeout for functions to appear
        );
        console.log('TTS functions are available.');

        console.log('Initializing TTS and downloading voice model (if not cached)...');
        const voiceInitialized = await page.evaluate(async (selectedVoiceId) => {
            // The functions are now guaranteed to exist by waitForFunction
            return window.piperTTS.initializeAndDownloadVoice(selectedVoiceId);
        }, voiceId);

        if (!voiceInitialized) {
            const pageError = await page.evaluate(() => window.synthesisError);
            console.error(`Failed to initialize voice model: ${voiceId}. Error: ${pageError}`);
            throw new Error(`Failed to initialize voice model: ${voiceId}. Error: ${pageError}`);
        }
        console.log(`Voice model ${voiceId} initialized successfully.`);

        console.log(`Synthesizing speech for: "${textToSynthesize}"`);
        await page.evaluate(async (text, selectedVoiceId) => {
            // The function is now guaranteed to exist
            await window.piperTTS.synthesizeSpeech(text, selectedVoiceId);
        }, textToSynthesize, voiceId);

        // Wait for synthesis to complete
        console.log('Waiting for synthesis to complete...');
        // Increased timeout as model download + synthesis can take time
        await page.waitForFunction('window.synthesisComplete === true', { timeout: 180000 }); // 3 min timeout

        const audioDataURL = await page.evaluate(() => window.audioDataURL);
        const synthesisError = await page.evaluate(() => window.synthesisError);

        if (synthesisError) {
            console.error(`Error during synthesis: ${synthesisError}`);
            throw new Error(`Synthesis error: ${synthesisError}`);
        }

        if (!audioDataURL) {
            console.error('Failed to retrieve audio data URL from the page.');
            throw new Error('Failed to retrieve audio data URL.');
        }

        console.log('Audio data URL retrieved successfully.');

        // Decode base64 and save to file
        const base64Data = audioDataURL.split(',')[1];
        if (!base64Data) {
            console.error('Invalid data URL format.');
            throw new Error('Invalid data URL format.');
        }
        const buffer = Buffer.from(base64Data, 'base64');

        await fs.writeFile(outputFilePath, buffer);
        console.log(`Audio saved to ${outputFilePath}`);

    } catch (error) {
        console.error('An error occurred in runTTS:', error);
        throw error; // Re-throw to be caught by CLI handler
    } finally {
        if (browser) {
            console.log('Closing browser...');
            await browser.close();
        }
    }
}

// Basic CLI argument parsing (to be replaced or enhanced by yargs later if needed)
async function main() {
    const args = process.argv.slice(2);
    if (args.length < 2) {
        console.log("Usage: node main.js <text_to_synthesize> <output_file.wav> [voice_id]");
        console.log("Example: node main.js \"Hello world\" output.wav en_US-lessac-medium");
        console.log("\nAvailable voice IDs can be explored by checking the tts.voices() method in the piper-tts-web library,");
        console.log("or see their documentation for common ones like 'en_US-hfc_female-medium', 'en_US-lessac-medium', etc.");
        process.exit(1);
    }

    const textToSynthesize = args[0];
    const outputFilePath = args[1];
    const voiceId = args[2]; // Optional, defaults in runTTS

    if (!outputFilePath.toLowerCase().endsWith('.wav')) {
        console.warn("Warning: Output file does not end with .wav. It will be saved as a WAV file regardless.");
    }

    try {
        await runTTS(textToSynthesize, outputFilePath, voiceId); // voiceId will be undefined if not provided, handled by default in runTTS
        console.log("TTS process completed successfully.");
    } catch (error) {
        console.error("TTS process failed:", error.message);
        process.exit(1);
    }
}

if (require.main === module) {
    main();
}

module.exports = { runTTS }; // Export for potential programmatic use
