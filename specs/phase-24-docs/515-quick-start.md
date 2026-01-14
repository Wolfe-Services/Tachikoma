# Spec 515: Quick Start Guide

## Overview
A streamlined 5-minute getting started guide that gets users from zero to running their first Tachikoma agent with minimal friction.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Prerequisites Section
- One-line system requirement check
- Supported platforms list
- Required permissions
- Network requirements (if any)

### Installation Steps
- Single command installation
- Homebrew: `brew install tachikoma`
- apt: `apt install tachikoma`
- Binary download with curl
- Verification command

### First Agent
- Initialize project: `tachikoma init`
- Explain generated files briefly
- Run first agent: `tachikoma run`
- Observe output and logs
- Success indicators

### Next Steps
- Links to full user guide
- Example projects to explore
- Join community channels
- Provide feedback link

### Troubleshooting Callouts
- Common installation issues
- Permission problems
- Network/proxy issues
- Platform-specific notes

## Content Guidelines
- Maximum 500 words
- No more than 10 steps
- Copy-pasteable commands
- Minimal explanation
- Clear success criteria

## Quick Start Template
```markdown
# Quick Start

## Install
curl -sSL https://tachikoma.dev/install | sh

## Initialize
tachikoma init my-first-agent
cd my-first-agent

## Run
tachikoma run

## Success!
You should see: "Agent running successfully"

## Next Steps
- [User Guide](./user-guide/)
- [Examples](./examples/)
```

## Dependencies
- Spec 511: Documentation Structure
- Spec 514: User Guide

## Verification
- [ ] Completes in under 5 minutes
- [ ] Works on all platforms
- [ ] Commands copy-paste correctly
- [ ] Success is clearly indicated
- [ ] Links to next steps work
