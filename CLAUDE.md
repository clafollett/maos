# MANDATORY INSTRUCTIONS - READ BEFORE PROCEEDING

You MUST read and follow ALL instructions found in the common Agent rules document: [AGENT_INSTRUCTIONS.md](./docs/AGENT_INSTRUCTIONS.md).

# VERIFICATION:
Before proceeding with any changes, confirm you have:
- [ ] Read and understood all rules in AGENT_INSTRUCTIONS.md
- [ ] Will follow the PR workflow (no direct pushes to main)
- [ ] Will follow test-driven development practices
- [ ] Will avoid analysis paralysis

## Claude-Specific Tips

1. **Use parallel search** - Multiple `Grep`/`Glob` calls in one message for efficiency
2. **Reference locations precisely** - Use `file.rs:123` format when mentioning code
