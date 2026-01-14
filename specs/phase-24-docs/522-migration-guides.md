# Spec 522: Migration Guides

## Overview
Documentation for migrating between Tachikoma versions and from competing/related tools to Tachikoma.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Version Migration Guides
- Upgrade path documentation
- Breaking changes listing
- Deprecation warnings
- Automated migration scripts
- Rollback procedures

### Version Migration Template
```markdown
# Migrating from v1.x to v2.x

## Overview
Key changes in v2.x and migration steps.

## Breaking Changes
- Config key `foo` renamed to `bar`
- CLI flag `--old` removed

## Migration Steps
1. Backup current configuration
2. Run migration script
3. Update config files
4. Test in staging
5. Deploy to production

## Automated Migration
```bash
tachikoma migrate --from v1 --to v2
```

## Rollback
If issues occur, revert with:
```bash
tachikoma migrate --rollback v1
```
```

### Tool Migration Guides
- From Make/Makefiles
- From Task (go-task)
- From Just
- From npm scripts
- From shell scripts
- From CI-only workflows

### Migration Tool Features
- Config file conversion
- Syntax translation
- Validation of converted config
- Side-by-side comparison
- Incremental migration support

### Compatibility Matrix
- Version compatibility table
- Feature availability by version
- Plugin compatibility
- OS/platform support matrix

### Migration Checklist
- [ ] Backup existing setup
- [ ] Review breaking changes
- [ ] Test migration in dev
- [ ] Update CI/CD pipelines
- [ ] Notify team members
- [ ] Monitor after migration

## Dependencies
- Spec 511: Documentation Structure
- Spec 516: Configuration Reference

## Verification
- [ ] All versions covered
- [ ] Tool migrations complete
- [ ] Scripts tested
- [ ] Rollback works
- [ ] Compatibility matrix accurate
