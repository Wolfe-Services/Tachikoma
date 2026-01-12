/**
 * Diff parsing and generation utilities
 */

import type { DiffFile, DiffHunk, DiffLine, WordDiff } from '@components/ui/DiffViewer/types';

/**
 * Parse unified diff format
 */
export function parseUnifiedDiff(diffText: string): DiffFile[] {
  const files: DiffFile[] = [];
  const lines = diffText.split('\n');

  let currentFile: DiffFile | null = null;
  let currentHunk: DiffHunk | null = null;
  let oldLineNum = 0;
  let newLineNum = 0;

  for (const line of lines) {
    // File header
    if (line.startsWith('--- ')) {
      if (currentFile && currentHunk) {
        currentFile.hunks.push(currentHunk);
      }
      if (currentFile) {
        files.push(currentFile);
      }

      currentFile = {
        oldPath: line.slice(4).replace(/^a\//, ''),
        newPath: '',
        hunks: []
      };
      currentHunk = null;
    } else if (line.startsWith('+++ ') && currentFile) {
      currentFile.newPath = line.slice(4).replace(/^b\//, '');
      currentFile.isNew = currentFile.oldPath === '/dev/null';
      currentFile.isDeleted = currentFile.newPath === '/dev/null';
      currentFile.isRenamed = currentFile.oldPath !== currentFile.newPath &&
                              !currentFile.isNew && !currentFile.isDeleted;
    }
    // Hunk header
    else if (line.startsWith('@@') && currentFile) {
      if (currentHunk) {
        currentFile.hunks.push(currentHunk);
      }

      const match = line.match(/@@ -(\d+),?(\d*) \+(\d+),?(\d*) @@/);
      if (match) {
        oldLineNum = parseInt(match[1], 10);
        newLineNum = parseInt(match[3], 10);

        currentHunk = {
          oldStart: oldLineNum,
          oldLines: parseInt(match[2] || '1', 10),
          newStart: newLineNum,
          newLines: parseInt(match[4] || '1', 10),
          lines: []
        };
      }
    }
    // Content lines
    else if (currentHunk) {
      if (line.startsWith('+')) {
        currentHunk.lines.push({
          type: 'add',
          content: line.slice(1),
          newLineNumber: newLineNum++
        });
      } else if (line.startsWith('-')) {
        currentHunk.lines.push({
          type: 'remove',
          content: line.slice(1),
          oldLineNumber: oldLineNum++
        });
      } else if (line.startsWith(' ') || line === '') {
        currentHunk.lines.push({
          type: 'unchanged',
          content: line.slice(1) || '',
          oldLineNumber: oldLineNum++,
          newLineNumber: newLineNum++
        });
      }
    }
  }

  // Push final hunk and file
  if (currentFile && currentHunk) {
    currentFile.hunks.push(currentHunk);
  }
  if (currentFile) {
    files.push(currentFile);
  }

  return files;
}

/**
 * Generate simple diff between two strings
 */
export function generateSimpleDiff(oldText: string, newText: string): DiffFile {
  const oldLines = oldText.split('\n');
  const newLines = newText.split('\n');

  const lines: DiffLine[] = [];
  let oldNum = 1;
  let newNum = 1;

  // Simple line-by-line comparison
  const maxLen = Math.max(oldLines.length, newLines.length);

  for (let i = 0; i < maxLen; i++) {
    const oldLine = oldLines[i];
    const newLine = newLines[i];

    if (oldLine === newLine) {
      lines.push({
        type: 'unchanged',
        content: oldLine || '',
        oldLineNumber: oldNum++,
        newLineNumber: newNum++
      });
    } else {
      if (oldLine !== undefined) {
        lines.push({
          type: 'remove',
          content: oldLine,
          oldLineNumber: oldNum++
        });
      }
      if (newLine !== undefined) {
        lines.push({
          type: 'add',
          content: newLine,
          newLineNumber: newNum++
        });
      }
    }
  }

  return {
    oldPath: 'old',
    newPath: 'new',
    hunks: [{
      oldStart: 1,
      oldLines: oldLines.length,
      newStart: 1,
      newLines: newLines.length,
      lines
    }]
  };
}

/**
 * Generate word-level diff for two lines
 */
export function generateWordDiff(oldLine: string, newLine: string): { old: WordDiff[]; new: WordDiff[] } {
  // Simple word-based split for basic word diff
  const oldWords = oldLine.split(/(\s+)/);
  const newWords = newLine.split(/(\s+)/);
  
  const oldResult: WordDiff[] = [];
  const newResult: WordDiff[] = [];
  
  let i = 0, j = 0;
  
  while (i < oldWords.length || j < newWords.length) {
    if (i >= oldWords.length) {
      // Only new words left
      newResult.push({ type: 'add', content: newWords[j] });
      j++;
    } else if (j >= newWords.length) {
      // Only old words left  
      oldResult.push({ type: 'remove', content: oldWords[i] });
      i++;
    } else if (oldWords[i] === newWords[j]) {
      // Words match
      oldResult.push({ type: 'unchanged', content: oldWords[i] });
      newResult.push({ type: 'unchanged', content: newWords[j] });
      i++;
      j++;
    } else {
      // Words differ
      oldResult.push({ type: 'remove', content: oldWords[i] });
      newResult.push({ type: 'add', content: newWords[j] });
      i++;
      j++;
    }
  }
  
  return { old: oldResult, new: newResult };
}

/**
 * Get all change indices for navigation
 */
export function getChangeIndices(hunks: DiffHunk[]): Array<{ hunkIndex: number; lineIndex: number }> {
  const changes: Array<{ hunkIndex: number; lineIndex: number }> = [];
  
  hunks.forEach((hunk, hunkIndex) => {
    hunk.lines.forEach((line, lineIndex) => {
      if (line.type === 'add' || line.type === 'remove') {
        changes.push({ hunkIndex, lineIndex });
      }
    });
  });
  
  return changes;
}

/**
 * Calculate diff statistics
 */
export function calculateDiffStats(file: DiffFile): { additions: number; deletions: number; total: number } {
  let additions = 0;
  let deletions = 0;
  
  file.hunks.forEach(hunk => {
    hunk.lines.forEach(line => {
      if (line.type === 'add') additions++;
      if (line.type === 'remove') deletions++;
    });
  });
  
  return { additions, deletions, total: additions + deletions };
}