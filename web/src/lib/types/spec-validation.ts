export type ValidationSeverity = 'error' | 'warning' | 'info' | 'suggestion';

export interface ValidationResult {
  isValid: boolean;
  score: number;
  errors: ValidationIssue[];
  warnings: ValidationIssue[];
  info: ValidationIssue[];
  suggestions: ValidationIssue[];
}

export interface ValidationIssue {
  id: string;
  severity: ValidationSeverity;
  code: string;
  message: string;
  description?: string;
  location?: IssueLocation;
  quickFixes?: QuickFix[];
  relatedIssues?: string[];
}

export interface IssueLocation {
  line: number;
  column?: number;
  endLine?: number;
  endColumn?: number;
  field?: string;
}

export interface QuickFix {
  id: string;
  title: string;
  description: string;
  edits: TextEdit[];
  isPreferred?: boolean;
}

export interface TextEdit {
  range: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  newText: string;
}

export interface ValidationRule {
  id: string;
  name: string;
  description: string;
  severity: ValidationSeverity;
  category: ValidationCategory;
  enabled: boolean;
}

export type ValidationCategory =
  | 'frontmatter'
  | 'structure'
  | 'content'
  | 'dependencies'
  | 'links'
  | 'formatting';