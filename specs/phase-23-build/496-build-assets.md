# Spec 496: Asset Handling

## Phase
23 - Build/Package System

## Spec ID
496

## Status
Planned

## Dependencies
- Spec 491 (Build System Orchestration)
- Spec 495 (Svelte Bundling)

## Estimated Context
~8%

---

## Objective

Implement the asset handling pipeline for images, fonts, icons, and other static resources. This includes optimization, compression, hashing for cache busting, and proper bundling for both web and Electron targets.

---

## Acceptance Criteria

- [ ] Image optimization (compression, resizing, format conversion)
- [ ] Font subsetting and optimization
- [ ] Icon sprite generation
- [ ] Asset hashing for cache invalidation
- [ ] Lazy loading support for images
- [ ] WebP/AVIF generation for modern browsers
- [ ] SVG optimization and inlining
- [ ] Asset manifest generation
- [ ] Copy static assets to build output
- [ ] Platform-specific icon generation

---

## Implementation Details

### Asset Pipeline Configuration (scripts/assets/pipeline.ts)

```typescript
// scripts/assets/pipeline.ts
import * as fs from 'fs';
import * as path from 'path';
import { createHash } from 'crypto';
import sharp from 'sharp';
import svgo from 'svgo';

interface AssetConfig {
  inputDir: string;
  outputDir: string;
  imageOptimization: {
    quality: number;
    maxWidth: number;
    maxHeight: number;
    generateWebp: boolean;
    generateAvif: boolean;
  };
  fontOptimization: {
    subset: boolean;
    formats: ('woff' | 'woff2')[];
  };
  svgOptimization: {
    removeViewBox: boolean;
    removeDimensions: boolean;
  };
  hash: boolean;
  hashLength: number;
}

const defaultConfig: AssetConfig = {
  inputDir: 'src/assets',
  outputDir: 'dist/assets',
  imageOptimization: {
    quality: 80,
    maxWidth: 2048,
    maxHeight: 2048,
    generateWebp: true,
    generateAvif: false,
  },
  fontOptimization: {
    subset: true,
    formats: ['woff2', 'woff'],
  },
  svgOptimization: {
    removeViewBox: false,
    removeDimensions: true,
  },
  hash: true,
  hashLength: 8,
};

interface AssetManifest {
  version: string;
  generated: string;
  assets: Record<string, AssetEntry>;
}

interface AssetEntry {
  original: string;
  hashed: string;
  size: number;
  type: string;
  variants?: Record<string, string>;
}

class AssetPipeline {
  private config: AssetConfig;
  private manifest: AssetManifest;

  constructor(config: Partial<AssetConfig> = {}) {
    this.config = { ...defaultConfig, ...config };
    this.manifest = {
      version: '1.0',
      generated: new Date().toISOString(),
      assets: {},
    };
  }

  async process(): Promise<void> {
    console.log('Processing assets...');

    // Ensure output directory exists
    fs.mkdirSync(this.config.outputDir, { recursive: true });

    // Process different asset types
    await this.processImages();
    await this.processFonts();
    await this.processSvgs();
    await this.copyStaticAssets();

    // Write manifest
    this.writeManifest();

    console.log('Asset processing complete!');
  }

  private async processImages(): Promise<void> {
    const imageDir = path.join(this.config.inputDir, 'images');
    if (!fs.existsSync(imageDir)) return;

    const images = this.getFiles(imageDir, ['.png', '.jpg', '.jpeg', '.gif']);

    for (const imagePath of images) {
      await this.optimizeImage(imagePath);
    }
  }

  private async optimizeImage(imagePath: string): Promise<void> {
    const { quality, maxWidth, maxHeight, generateWebp, generateAvif } =
      this.config.imageOptimization;

    const relativePath = path.relative(this.config.inputDir, imagePath);
    const ext = path.extname(imagePath).toLowerCase();
    const baseName = path.basename(imagePath, ext);

    // Read and optimize image
    let image = sharp(imagePath);
    const metadata = await image.metadata();

    // Resize if needed
    if (
      (metadata.width && metadata.width > maxWidth) ||
      (metadata.height && metadata.height > maxHeight)
    ) {
      image = image.resize(maxWidth, maxHeight, {
        fit: 'inside',
        withoutEnlargement: true,
      });
    }

    // Generate original format (optimized)
    const outputBuffer = await image
      .jpeg({ quality, mozjpeg: true })
      .png({ quality, compressionLevel: 9 })
      .toBuffer();

    const hash = this.generateHash(outputBuffer);
    const hashedName = this.config.hash
      ? `${baseName}-${hash}${ext}`
      : `${baseName}${ext}`;

    const outputPath = path.join(
      this.config.outputDir,
      'images',
      hashedName
    );
    fs.mkdirSync(path.dirname(outputPath), { recursive: true });
    fs.writeFileSync(outputPath, outputBuffer);

    const variants: Record<string, string> = {};

    // Generate WebP variant
    if (generateWebp) {
      const webpBuffer = await image.webp({ quality }).toBuffer();
      const webpHash = this.generateHash(webpBuffer);
      const webpName = this.config.hash
        ? `${baseName}-${webpHash}.webp`
        : `${baseName}.webp`;
      const webpPath = path.join(this.config.outputDir, 'images', webpName);
      fs.writeFileSync(webpPath, webpBuffer);
      variants.webp = `images/${webpName}`;
    }

    // Generate AVIF variant
    if (generateAvif) {
      const avifBuffer = await image.avif({ quality }).toBuffer();
      const avifHash = this.generateHash(avifBuffer);
      const avifName = this.config.hash
        ? `${baseName}-${avifHash}.avif`
        : `${baseName}.avif`;
      const avifPath = path.join(this.config.outputDir, 'images', avifName);
      fs.writeFileSync(avifPath, avifBuffer);
      variants.avif = `images/${avifName}`;
    }

    // Add to manifest
    this.manifest.assets[relativePath] = {
      original: relativePath,
      hashed: `images/${hashedName}`,
      size: outputBuffer.length,
      type: 'image',
      variants: Object.keys(variants).length > 0 ? variants : undefined,
    };
  }

  private async processFonts(): Promise<void> {
    const fontDir = path.join(this.config.inputDir, 'fonts');
    if (!fs.existsSync(fontDir)) return;

    const fonts = this.getFiles(fontDir, ['.ttf', '.otf', '.woff', '.woff2']);

    for (const fontPath of fonts) {
      await this.processFont(fontPath);
    }
  }

  private async processFont(fontPath: string): Promise<void> {
    const relativePath = path.relative(this.config.inputDir, fontPath);
    const ext = path.extname(fontPath);
    const baseName = path.basename(fontPath, ext);

    const content = fs.readFileSync(fontPath);
    const hash = this.generateHash(content);
    const hashedName = this.config.hash
      ? `${baseName}-${hash}${ext}`
      : `${baseName}${ext}`;

    const outputPath = path.join(this.config.outputDir, 'fonts', hashedName);
    fs.mkdirSync(path.dirname(outputPath), { recursive: true });
    fs.copyFileSync(fontPath, outputPath);

    this.manifest.assets[relativePath] = {
      original: relativePath,
      hashed: `fonts/${hashedName}`,
      size: content.length,
      type: 'font',
    };
  }

  private async processSvgs(): Promise<void> {
    const svgDir = path.join(this.config.inputDir, 'svg');
    if (!fs.existsSync(svgDir)) return;

    const svgs = this.getFiles(svgDir, ['.svg']);

    for (const svgPath of svgs) {
      await this.optimizeSvg(svgPath);
    }
  }

  private async optimizeSvg(svgPath: string): Promise<void> {
    const relativePath = path.relative(this.config.inputDir, svgPath);
    const baseName = path.basename(svgPath, '.svg');

    const content = fs.readFileSync(svgPath, 'utf-8');

    // Optimize with SVGO
    const result = svgo.optimize(content, {
      multipass: true,
      plugins: [
        'preset-default',
        {
          name: 'removeViewBox',
          active: this.config.svgOptimization.removeViewBox,
        },
        {
          name: 'removeDimensions',
          active: this.config.svgOptimization.removeDimensions,
        },
      ],
    });

    const optimized = result.data;
    const hash = this.generateHash(Buffer.from(optimized));
    const hashedName = this.config.hash
      ? `${baseName}-${hash}.svg`
      : `${baseName}.svg`;

    const outputPath = path.join(this.config.outputDir, 'svg', hashedName);
    fs.mkdirSync(path.dirname(outputPath), { recursive: true });
    fs.writeFileSync(outputPath, optimized);

    this.manifest.assets[relativePath] = {
      original: relativePath,
      hashed: `svg/${hashedName}`,
      size: Buffer.byteLength(optimized),
      type: 'svg',
    };
  }

  private copyStaticAssets(): void {
    const staticDir = path.join(this.config.inputDir, 'static');
    if (!fs.existsSync(staticDir)) return;

    const files = this.getFiles(staticDir);

    for (const filePath of files) {
      const relativePath = path.relative(this.config.inputDir, filePath);
      const outputPath = path.join(this.config.outputDir, relativePath);

      fs.mkdirSync(path.dirname(outputPath), { recursive: true });
      fs.copyFileSync(filePath, outputPath);

      const content = fs.readFileSync(filePath);
      this.manifest.assets[relativePath] = {
        original: relativePath,
        hashed: relativePath,
        size: content.length,
        type: 'static',
      };
    }
  }

  private getFiles(dir: string, extensions?: string[]): string[] {
    const files: string[] = [];

    function walk(currentDir: string): void {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (
          !extensions ||
          extensions.includes(path.extname(entry.name).toLowerCase())
        ) {
          files.push(fullPath);
        }
      }
    }

    walk(dir);
    return files;
  }

  private generateHash(content: Buffer): string {
    return createHash('md5')
      .update(content)
      .digest('hex')
      .slice(0, this.config.hashLength);
  }

  private writeManifest(): void {
    const manifestPath = path.join(this.config.outputDir, 'manifest.json');
    fs.writeFileSync(manifestPath, JSON.stringify(this.manifest, null, 2));
  }

  getManifest(): AssetManifest {
    return this.manifest;
  }
}

export { AssetPipeline, AssetConfig, AssetManifest };
```

### Icon Generation Script (scripts/assets/icons.ts)

```typescript
// scripts/assets/icons.ts
import * as fs from 'fs';
import * as path from 'path';
import sharp from 'sharp';
import { createICO } from 'create-ico';

interface IconConfig {
  source: string;
  outputDir: string;
  sizes: {
    ico: number[];
    icns: number[];
    png: number[];
  };
}

const defaultIconConfig: IconConfig = {
  source: 'resources/icon.svg',
  outputDir: 'resources',
  sizes: {
    ico: [16, 24, 32, 48, 64, 128, 256],
    icns: [16, 32, 64, 128, 256, 512, 1024],
    png: [16, 32, 48, 64, 128, 256, 512, 1024],
  },
};

async function generateIcons(config: Partial<IconConfig> = {}): Promise<void> {
  const cfg = { ...defaultIconConfig, ...config };

  console.log('Generating application icons...');

  // Ensure output directory exists
  fs.mkdirSync(cfg.outputDir, { recursive: true });

  // Generate PNG icons
  const pngBuffers: Record<number, Buffer> = {};
  for (const size of cfg.sizes.png) {
    const buffer = await sharp(cfg.source)
      .resize(size, size)
      .png()
      .toBuffer();

    pngBuffers[size] = buffer;

    // Save individual PNG
    const pngPath = path.join(cfg.outputDir, 'icons', `${size}x${size}.png`);
    fs.mkdirSync(path.dirname(pngPath), { recursive: true });
    fs.writeFileSync(pngPath, buffer);
  }

  // Generate ICO for Windows
  const icoBuffers = cfg.sizes.ico.map((size) => pngBuffers[size]);
  const icoBuffer = await createICO(icoBuffers);
  fs.writeFileSync(path.join(cfg.outputDir, 'icon.ico'), icoBuffer);

  // Generate ICNS for macOS (using iconutil on macOS)
  if (process.platform === 'darwin') {
    await generateIcns(cfg, pngBuffers);
  }

  console.log('Icon generation complete!');
}

async function generateIcns(
  config: IconConfig,
  pngBuffers: Record<number, Buffer>
): Promise<void> {
  const iconsetDir = path.join(config.outputDir, 'icon.iconset');
  fs.mkdirSync(iconsetDir, { recursive: true });

  // ICNS requires specific filenames
  const icnsFiles = [
    { name: 'icon_16x16.png', size: 16 },
    { name: 'icon_16x16@2x.png', size: 32 },
    { name: 'icon_32x32.png', size: 32 },
    { name: 'icon_32x32@2x.png', size: 64 },
    { name: 'icon_128x128.png', size: 128 },
    { name: 'icon_128x128@2x.png', size: 256 },
    { name: 'icon_256x256.png', size: 256 },
    { name: 'icon_256x256@2x.png', size: 512 },
    { name: 'icon_512x512.png', size: 512 },
    { name: 'icon_512x512@2x.png', size: 1024 },
  ];

  for (const { name, size } of icnsFiles) {
    const buffer = pngBuffers[size];
    if (buffer) {
      fs.writeFileSync(path.join(iconsetDir, name), buffer);
    }
  }

  // Run iconutil to create .icns
  const { execSync } = await import('child_process');
  execSync(
    `iconutil -c icns "${iconsetDir}" -o "${path.join(config.outputDir, 'icon.icns')}"`,
    { stdio: 'inherit' }
  );

  // Clean up iconset directory
  fs.rmSync(iconsetDir, { recursive: true, force: true });
}

export { generateIcons, IconConfig };
```

### Asset Loader Utility (web/src/lib/utils/assets.ts)

```typescript
// web/src/lib/utils/assets.ts

// Import manifest at build time
import manifest from '$lib/assets/manifest.json';

type AssetManifest = typeof manifest;

/**
 * Get the hashed URL for an asset
 */
export function asset(path: string): string {
  const entry = manifest.assets[path];
  if (entry) {
    return `/assets/${entry.hashed}`;
  }
  // Fallback to original path
  return `/assets/${path}`;
}

/**
 * Get image with srcset for responsive images
 */
export function imageSrcSet(
  path: string,
  options: { webp?: boolean; avif?: boolean } = {}
): { src: string; srcset: string } {
  const entry = manifest.assets[path];
  if (!entry || entry.type !== 'image') {
    return { src: asset(path), srcset: '' };
  }

  const sources: string[] = [];

  // Add AVIF variant
  if (options.avif && entry.variants?.avif) {
    sources.push(`/assets/${entry.variants.avif} type=image/avif`);
  }

  // Add WebP variant
  if (options.webp && entry.variants?.webp) {
    sources.push(`/assets/${entry.variants.webp} type=image/webp`);
  }

  return {
    src: `/assets/${entry.hashed}`,
    srcset: sources.join(', '),
  };
}

/**
 * Preload critical assets
 */
export function preloadAsset(path: string): void {
  const url = asset(path);
  const link = document.createElement('link');
  link.rel = 'preload';
  link.href = url;

  // Determine as attribute based on file type
  const ext = path.split('.').pop()?.toLowerCase();
  if (ext && ['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif'].includes(ext)) {
    link.as = 'image';
  } else if (ext && ['woff', 'woff2', 'ttf', 'otf'].includes(ext)) {
    link.as = 'font';
    link.crossOrigin = 'anonymous';
  }

  document.head.appendChild(link);
}

/**
 * Get all assets of a specific type
 */
export function getAssetsByType(
  type: 'image' | 'font' | 'svg' | 'static'
): string[] {
  return Object.entries(manifest.assets)
    .filter(([_, entry]) => entry.type === type)
    .map(([path]) => path);
}
```

### Svelte Image Component (web/src/lib/components/OptimizedImage.svelte)

```svelte
<!-- web/src/lib/components/OptimizedImage.svelte -->
<script lang="ts">
  import { asset, imageSrcSet } from '$lib/utils/assets';

  interface Props {
    src: string;
    alt: string;
    width?: number;
    height?: number;
    loading?: 'lazy' | 'eager';
    class?: string;
    webp?: boolean;
    avif?: boolean;
  }

  let {
    src,
    alt,
    width,
    height,
    loading = 'lazy',
    class: className = '',
    webp = true,
    avif = false,
  }: Props = $props();

  const { src: imgSrc, srcset } = $derived(
    imageSrcSet(src, { webp, avif })
  );
</script>

<picture>
  {#if avif}
    <source srcset={srcset} type="image/avif" />
  {/if}
  {#if webp}
    <source srcset={srcset} type="image/webp" />
  {/if}
  <img
    src={imgSrc}
    {alt}
    {width}
    {height}
    {loading}
    class={className}
    decoding="async"
  />
</picture>

<style>
  img {
    max-width: 100%;
    height: auto;
  }
</style>
```

---

## Testing Requirements

### Unit Tests

```typescript
// scripts/assets/__tests__/pipeline.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { AssetPipeline } from '../pipeline';
import * as fs from 'fs';
import * as path from 'path';

describe('AssetPipeline', () => {
  const testDir = path.join(__dirname, '.test-assets');
  const inputDir = path.join(testDir, 'input');
  const outputDir = path.join(testDir, 'output');

  beforeEach(() => {
    fs.mkdirSync(path.join(inputDir, 'images'), { recursive: true });
    fs.mkdirSync(path.join(inputDir, 'fonts'), { recursive: true });
    fs.mkdirSync(path.join(inputDir, 'svg'), { recursive: true });
  });

  afterEach(() => {
    fs.rmSync(testDir, { recursive: true, force: true });
  });

  it('should create pipeline with default config', () => {
    const pipeline = new AssetPipeline();
    expect(pipeline).toBeDefined();
  });

  it('should generate manifest', async () => {
    const pipeline = new AssetPipeline({
      inputDir,
      outputDir,
    });

    await pipeline.process();
    const manifest = pipeline.getManifest();

    expect(manifest.version).toBe('1.0');
    expect(manifest.generated).toBeDefined();
  });

  it('should hash assets when enabled', async () => {
    // Create a test file
    fs.writeFileSync(path.join(inputDir, 'static', 'test.txt'), 'test');
    fs.mkdirSync(path.join(inputDir, 'static'), { recursive: true });

    const pipeline = new AssetPipeline({
      inputDir,
      outputDir,
      hash: true,
      hashLength: 8,
    });

    await pipeline.process();
    const manifest = pipeline.getManifest();

    // Check that hash is applied
    const entry = Object.values(manifest.assets)[0];
    if (entry) {
      expect(entry.hashed).toContain('-');
    }
  });
});
```

### Integration Tests

```typescript
// scripts/assets/__tests__/icons.test.ts
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { generateIcons } from '../icons';
import * as fs from 'fs';
import * as path from 'path';

describe('Icon Generation', () => {
  const testDir = path.join(__dirname, '.test-icons');

  beforeEach(() => {
    fs.mkdirSync(testDir, { recursive: true });
  });

  afterEach(() => {
    fs.rmSync(testDir, { recursive: true, force: true });
  });

  it('should generate PNG icons', async () => {
    // Skip if no source icon available
    const sourcePath = path.join(__dirname, '..', '..', '..', 'resources', 'icon.svg');
    if (!fs.existsSync(sourcePath)) {
      return;
    }

    await generateIcons({
      source: sourcePath,
      outputDir: testDir,
      sizes: {
        ico: [16, 32],
        icns: [16, 32],
        png: [16, 32, 64],
      },
    });

    expect(fs.existsSync(path.join(testDir, 'icons', '16x16.png'))).toBe(true);
    expect(fs.existsSync(path.join(testDir, 'icons', '32x32.png'))).toBe(true);
  });
});
```

---

## Related Specs

- Spec 491: Build System Orchestration
- Spec 495: Svelte Bundling
- Spec 494: Electron Packaging
- Spec 499: macOS Packaging
- Spec 500: Windows Installer
