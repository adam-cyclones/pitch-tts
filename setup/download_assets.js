const https = require('https');
const fs = require('fs');
const path = require('path');

const vendorDir = path.resolve(__dirname, '../vendor');

const filesToDownload = [
    {
        url: 'https://cdn.jsdelivr.net/npm/@mintplex-labs/piper-tts-web@1.0.4/dist/piper-tts-web.js',
        destDir: 'piper-tts-web',
        fileName: 'piper-tts-web.js'
    },
    {
        url: 'https://cdn.jsdelivr.net/npm/@mintplex-labs/piper-tts-web@1.0.4/dist/piper-o91UDS6e.js',
        destDir: 'piper-tts-web',
        fileName: 'piper-o91UDS6e.js'
    },
    {
        url: 'https://cdn.jsdelivr.net/npm/@mintplex-labs/piper-tts-web@1.0.4/dist/voices_static-D_OtJDHM.js',
        destDir: 'piper-tts-web',
        fileName: 'voices_static-D_OtJDHM.js'
    },
    {
        url: 'https://cdnjs.cloudflare.com/ajax/libs/onnxruntime-web/1.18.0/ort-wasm.wasm',
        destDir: 'onnxruntime-web',
        fileName: 'ort-wasm.wasm'
    },
    {
        url: 'https://cdnjs.cloudflare.com/ajax/libs/onnxruntime-web/1.18.0/ort-wasm-simd.wasm',
        destDir: 'onnxruntime-web',
        fileName: 'ort-wasm-simd.wasm'
    },
    {
        url: 'https://cdnjs.cloudflare.com/ajax/libs/onnxruntime-web/1.18.0/ort-wasm-threaded.wasm',
        destDir: 'onnxruntime-web',
        fileName: 'ort-wasm-threaded.wasm'
    },
    {
        url: 'https://cdn.jsdelivr.net/npm/@diffusionstudio/piper-wasm@1.0.0/build/piper_phonemize.wasm',
        destDir: 'piper-wasm',
        fileName: 'piper_phonemize.wasm'
    },
    {
        url: 'https://cdn.jsdelivr.net/npm/@diffusionstudio/piper-wasm@1.0.0/build/piper_phonemize.data',
        destDir: 'piper-wasm',
        fileName: 'piper_phonemize.data'
    }
];

async function downloadFile(url, destPath) {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(destPath);
        https.get(url, (response) => {
            if (response.statusCode !== 200) {
                reject(new Error(`Failed to get '${url}' (${response.statusCode})`));
                return;
            }
            response.pipe(file);
            file.on('finish', () => {
                file.close(resolve);
            });
        }).on('error', (err) => {
            fs.unlink(destPath, () => reject(err)); // Delete the file if download fails
        });
    });
}

async function main() {
    if (!fs.existsSync(vendorDir)) {
        fs.mkdirSync(vendorDir, { recursive: true });
        console.log(`Created directory: ${vendorDir}`);
    }

    for (const fileInfo of filesToDownload) {
        const fullDestDir = path.join(vendorDir, fileInfo.destDir);
        if (!fs.existsSync(fullDestDir)) {
            fs.mkdirSync(fullDestDir, { recursive: true });
            console.log(`Created directory: ${fullDestDir}`);
        }
        const destFilePath = path.join(fullDestDir, fileInfo.fileName);
        console.log(`Downloading ${fileInfo.url} to ${destFilePath}...`);
        try {
            await downloadFile(fileInfo.url, destFilePath);
            console.log(`Successfully downloaded ${fileInfo.fileName}`);
        } catch (error) {
            console.error(`Error downloading ${fileInfo.fileName}: ${error.message}`);
        }
    }
    console.log('\nAll downloads attempted.');
    console.log('Please check for any errors above.');
    console.log(`\nMake sure the vendor directory (expected at: ${path.resolve(vendorDir)}) and its contents are correctly placed in your workspace for Jules to use.`);
}

main();
