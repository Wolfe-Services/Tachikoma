// scripts/sign-windows.ts
import { execSync } from 'child_process';
import { existsSync } from 'fs';

interface WindowsSigningConfig {
  certificatePath: string;
  certificatePassword: string;
  timestampServer: string;
  description: string;
  descriptionUrl: string;
}

export async function signWindowsExecutable(
  exePath: string,
  config: WindowsSigningConfig
): Promise<void> {
  const { certificatePath, certificatePassword, timestampServer, description, descriptionUrl } = config;

  // Use signtool from Windows SDK
  const signtool = process.env.SIGNTOOL_PATH || 'signtool';

  const args: string[] = [
    'sign',
    '/f',
    certificatePath,
    '/p',
    certificatePassword,
    '/fd',
    'sha256',
    '/tr',
    timestampServer,
    '/td',
    'sha256',
    '/d',
    `"${description}"`,
    '/du',
    descriptionUrl,
    `"${exePath}"`,
  ];

  console.log(`Signing: ${exePath}`);
  execSync(`${signtool} ${args.join(' ')}`, { stdio: 'inherit' });

  // Verify signature
  console.log('Verifying signature...');
  execSync(`${signtool} verify /pa "${exePath}"`, { stdio: 'inherit' });
}

export async function signWindowsInstaller(
  installerPath: string,
  config: WindowsSigningConfig
): Promise<void> {
  // Sign the installer itself
  await signWindowsExecutable(installerPath, config);

  // If it's an NSIS installer, also sign the uninstaller
  const uninstallerPath = installerPath.replace('.exe', '-uninstaller.exe');
  try {
    if (existsSync(uninstallerPath)) {
      await signWindowsExecutable(uninstallerPath, config);
    }
  } catch {
    console.log('No separate uninstaller found');
  }
}

// electron-builder sign hook for Windows
export async function windowsSignHook(configuration: any): Promise<void> {
  const { path } = configuration;

  if (!process.env.WIN_CSC_LINK || !process.env.WIN_CSC_KEY_PASSWORD) {
    console.warn('Skipping Windows signing: credentials not provided');
    return;
  }

  await signWindowsExecutable(path, {
    certificatePath: process.env.WIN_CSC_LINK,
    certificatePassword: process.env.WIN_CSC_KEY_PASSWORD,
    timestampServer: 'http://timestamp.digicert.com',
    description: 'Tachikoma',
    descriptionUrl: 'https://tachikoma.io',
  });
}

// CLI function
export async function signWindowsCLI(): Promise<void> {
  const exePath = process.argv[2];
  
  if (!exePath) {
    console.error('Usage: npm run sign:win <exe-path>');
    process.exit(1);
  }

  const certificatePath = process.env.WIN_CSC_LINK;
  const certificatePassword = process.env.WIN_CSC_KEY_PASSWORD;

  if (!certificatePath || !certificatePassword) {
    console.error('WIN_CSC_LINK and WIN_CSC_KEY_PASSWORD environment variables required');
    process.exit(1);
  }

  const config: WindowsSigningConfig = {
    certificatePath,
    certificatePassword,
    timestampServer: process.env.WIN_TIMESTAMP_SERVER || 'http://timestamp.digicert.com',
    description: 'Tachikoma',
    descriptionUrl: 'https://tachikoma.io',
  };

  try {
    await signWindowsExecutable(exePath, config);
    console.log('Windows signing complete!');
  } catch (error) {
    console.error('Signing failed:', error);
    process.exit(1);
  }
}

// Check if signtool is available
export function checkSigntoolAvailability(): boolean {
  try {
    const signtool = process.env.SIGNTOOL_PATH || 'signtool';
    execSync(`${signtool} /?`, { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

// Run if called directly
if (require.main === module) {
  if (!checkSigntoolAvailability()) {
    console.error('signtool not found. Please install Windows SDK or set SIGNTOOL_PATH environment variable.');
    process.exit(1);
  }
  signWindowsCLI();
}