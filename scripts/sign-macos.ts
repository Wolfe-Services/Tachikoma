// scripts/sign-macos.ts
import { execSync } from 'child_process';
import { join } from 'path';

interface SigningConfig {
  identity: string;
  entitlements: string;
  hardenedRuntime: boolean;
  timestamp: boolean;
}

export async function signMacOSApp(
  appPath: string,
  config: SigningConfig
): Promise<void> {
  const { identity, entitlements, hardenedRuntime, timestamp } = config;

  // Build codesign arguments
  const args: string[] = [
    '--force',
    '--deep',
    '--sign',
    identity,
    '--entitlements',
    entitlements,
  ];

  if (hardenedRuntime) {
    args.push('--options', 'runtime');
  }

  if (timestamp) {
    args.push('--timestamp');
  }

  args.push(appPath);

  console.log('Signing application...');
  execSync(`codesign ${args.join(' ')}`, { stdio: 'inherit' });

  // Verify signature
  console.log('Verifying signature...');
  execSync(`codesign --verify --deep --strict --verbose=2 "${appPath}"`, {
    stdio: 'inherit',
  });

  console.log('Checking entitlements...');
  execSync(`codesign -d --entitlements :- "${appPath}"`, { stdio: 'inherit' });
}

export async function signMacOSFrameworks(appPath: string, identity: string): Promise<void> {
  const frameworksPath = join(appPath, 'Contents', 'Frameworks');

  // Sign each framework
  const frameworks = [
    'Electron Framework.framework',
    'Mantle.framework',
    'ReactiveCocoa.framework',
    'Squirrel.framework',
  ];

  for (const framework of frameworks) {
    const frameworkPath = join(frameworksPath, framework);
    try {
      execSync(
        `codesign --force --deep --sign "${identity}" --options runtime "${frameworkPath}"`,
        { stdio: 'inherit' }
      );
    } catch {
      console.log(`Framework ${framework} not found, skipping...`);
    }
  }

  // Sign helper apps
  const helpers = [
    'Tachikoma Helper.app',
    'Tachikoma Helper (GPU).app',
    'Tachikoma Helper (Renderer).app',
    'Tachikoma Helper (Plugin).app',
  ];

  for (const helper of helpers) {
    const helperPath = join(frameworksPath, 'Electron Framework.framework', 'Helpers', helper);
    try {
      execSync(
        `codesign --force --deep --sign "${identity}" --options runtime "${helperPath}"`,
        { stdio: 'inherit' }
      );
    } catch {
      console.log(`Helper ${helper} not found, skipping...`);
    }
  }
}

// Get available signing identities
export function getSigningIdentities(): string[] {
  try {
    const output = execSync('security find-identity -v -p codesigning', { encoding: 'utf-8' });
    const lines = output.split('\n');
    const identities: string[] = [];
    
    for (const line of lines) {
      const match = line.match(/"([^"]+)"/);
      if (match && match[1].includes('Developer ID Application')) {
        identities.push(match[1]);
      }
    }
    
    return identities;
  } catch (error) {
    console.error('Failed to get signing identities:', error);
    return [];
  }
}

// Main signing function for CLI usage
export async function signMacOSCLI(): Promise<void> {
  const appPath = process.argv[2];
  
  if (!appPath) {
    console.error('Usage: npm run sign:mac <app-path>');
    process.exit(1);
  }

  const identities = getSigningIdentities();
  if (identities.length === 0) {
    console.error('No valid Developer ID Application certificates found');
    process.exit(1);
  }

  console.log('Available identities:', identities);
  const identity = identities[0]; // Use first available
  
  const config: SigningConfig = {
    identity,
    entitlements: join(__dirname, '..', 'electron', 'build', 'entitlements.mac.plist'),
    hardenedRuntime: true,
    timestamp: true,
  };

  try {
    // Sign frameworks first
    await signMacOSFrameworks(appPath, identity);
    
    // Then sign the main app
    await signMacOSApp(appPath, config);
    
    console.log('macOS signing complete!');
  } catch (error) {
    console.error('Signing failed:', error);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  signMacOSCLI();
}