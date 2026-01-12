// src/electron/main/windows/jumplist.ts
import { app, JumpListCategory } from 'electron';
import { Logger } from '../logger';

const logger = new Logger('jumplist');

interface RecentProject {
  path: string;
  name: string;
  lastOpened: Date;
}

export function setupJumpList(recentProjects: RecentProject[] = []): void {
  if (process.platform !== 'win32') {
    logger.debug('Jump list only supported on Windows');
    return;
  }

  const tasks: JumpListCategory = {
    type: 'tasks',
    items: [
      {
        type: 'task',
        title: 'New Project',
        description: 'Create a new Tachikoma project',
        program: process.execPath,
        args: '--new-project',
        iconPath: process.execPath,
        iconIndex: 0,
      },
      {
        type: 'task',
        title: 'Open Project',
        description: 'Open an existing project',
        program: process.execPath,
        args: '--open-project',
        iconPath: process.execPath,
        iconIndex: 0,
      },
      {
        type: 'task',
        title: 'New Terminal',
        description: 'Open integrated terminal',
        program: process.execPath,
        args: '--terminal',
        iconPath: process.execPath,
        iconIndex: 0,
      },
    ],
  };

  const recent: JumpListCategory = {
    type: 'custom',
    name: 'Recent Projects',
    items: recentProjects.slice(0, 10).map((project) => ({
      type: 'task' as const,
      title: project.name,
      description: project.path,
      program: process.execPath,
      args: `"${project.path}"`,
      iconPath: process.execPath,
      iconIndex: 0,
    })),
  };

  try {
    const jumpList = [tasks];
    if (recent.items.length > 0) {
      jumpList.push(recent);
    }
    
    app.setJumpList(jumpList);
    logger.info('Jump list configured', { 
      tasksCount: tasks.items.length, 
      recentCount: recent.items.length 
    });
  } catch (error) {
    logger.error('Failed to set jump list:', error);
  }
}

export function clearJumpList(): void {
  if (process.platform !== 'win32') {
    return;
  }

  try {
    app.setJumpList([]);
    logger.info('Jump list cleared');
  } catch (error) {
    logger.error('Failed to clear jump list:', error);
  }
}

export function addToRecentDocuments(path: string): void {
  if (process.platform !== 'win32') {
    return;
  }

  try {
    app.addRecentDocument(path);
    logger.debug('Added to recent documents', { path });
  } catch (error) {
    logger.error('Failed to add to recent documents:', error);
  }
}

export function clearRecentDocuments(): void {
  if (process.platform !== 'win32') {
    return;
  }

  try {
    app.clearRecentDocuments();
    logger.info('Recent documents cleared');
  } catch (error) {
    logger.error('Failed to clear recent documents:', error);
  }
}