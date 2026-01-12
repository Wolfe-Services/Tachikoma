// scripts/notarize-macos.ts
import { execSync } from 'child_process';
import { join } from 'path';

interface NotarizeConfig {
  appBundleId: string;
  appPath: string;
  appleId: string;
  appleIdPassword: string;
  teamId: string;
}

export async function notarizeMacOSApp(config: NotarizeConfig): Promise<void> {
  const { appBundleId, appPath, appleId, appleIdPassword, teamId } = config;

  console.log('Starting notarization...');
  console.log(`App path: ${appPath}`);
  console.log(`Bundle ID: ${appBundleId}`);

  try {
    // Create a zip file for notarization
    const zipPath = appPath.replace('.app', '.zip');
    console.log('Creating zip for notarization...');
    execSync(`ditto -c -k --keepParent "${appPath}" "${zipPath}"`, {
      stdio: 'inherit',
    });

    // Submit for notarization
    console.log('Submitting for notarization...');
    const submitOutput = execSync(
      `xcrun notarytool submit "${zipPath}" --apple-id "${appleId}" --password "${appleIdPassword}" --team-id "${teamId}" --wait`,
      { encoding: 'utf-8', stdio: ['inherit', 'pipe', 'inherit'] }
    );

    // Check if successful
    if (submitOutput.includes('Successfully')) {
      console.log('Notarization successful!');
      
      // Staple the notarization
      console.log('Stapling notarization...');
      execSync(`xcrun stapler staple "${appPath}"`, { stdio: 'inherit' });
      
      // Verify notarization
      console.log('Verifying notarization...');
      execSync(`spctl --assess --type execute --verbose "${appPath}"`, {
        stdio: 'inherit',
      });
    } else {
      throw new Error('Notarization failed');
    }

    // Clean up zip file
    try {
      execSync(`rm "${zipPath}"`);
    } catch {
      // Ignore cleanup errors
    }

  } catch (error) {
    console.error('Notarization failed:', error);
    throw error;
  }
}

// electron-builder afterSign hook
export async function afterSignHook(context: any): Promise<void> {
  const { electronPlatformName, appOutDir } = context;

  if (electronPlatformName !== 'darwin') {
    return;
  }

  if (!process.env.APPLE_ID || !process.env.APPLE_APP_SPECIFIC_PASSWORD) {
    console.warn('Skipping notarization: credentials not provided');
    return;
  }

  const appName = context.packager.appInfo.productFilename;
  const appPath = join(appOutDir, `${appName}.app`);

  await notarizeMacOSApp({
    appBundleId: 'io.tachikoma.app',
    appPath,
    appleId: process.env.APPLE_ID,
    appleIdPassword: process.env.APPLE_APP_SPECIFIC_PASSWORD,
    teamId: process.env.APPLE_TEAM_ID || '',
  });
}

// CLI function
export async function notarizeMacOSCLI(): Promise<void> {
  const appPath = process.argv[2];
  
  if (!appPath) {
    console.error('Usage: npm run notarize:mac <app-path>');
    process.exit(1);
  }

  const appleId = process.env.APPLE_ID;
  const appleIdPassword = process.env.APPLE_APP_SPECIFIC_PASSWORD;
  const teamId = process.env.APPLE_TEAM_ID;

  if (!appleId || !appleIdPassword) {
    console.error('APPLE_ID and APPLE_APP_SPECIFIC_PASSWORD environment variables required');
    process.exit(1);
  }

  try {
    await notarizeMacOSApp({
      appBundleId: 'io.tachikoma.app',
      appPath,
      appleId,
      appleIdPassword,
      teamId: teamId || '',
    });
    
    console.log('macOS notarization complete!');
  } catch (error) {
    console.error('Notarization failed:', error);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  notarizeMacOSCLI();
}