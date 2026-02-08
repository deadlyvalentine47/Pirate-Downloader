# Agent Preferences for PirateDownloader

## Command Execution
- **Always propose commands but NEVER auto-run them**
- Set `SafeToAutoRun=false` for ALL commands (cargo, npm, git, etc.)
- User will run commands manually and share output
- Exception: Only auto-run if explicitly requested by user

## Task Tracking & Documentation
- **Use `memory/mvp2/` directory for all MVP2 tracking**
- Update `memory/mvp2/trackers/active-tracker.md` for current work
- Update `memory/mvp2/trackers/pending-tracker.md` for task status
- Use `memory/mvp2/plans/` for implementation plans
- **DO NOT create artifacts** in brain/ directory for:
  - Task tracking
  - Implementation plans
  - Progress documentation
  - Walkthroughs
- Only use artifacts for:
  - Generated images
  - User-requested standalone documents

## Workflow
1. Update trackers in memory/mvp2/trackers/ directory
2. Create/update plans in memory/mvp2/plans/ if needed
3. Make code changes
4. Propose commands for user to run
5. Wait for user to share output
6. Update trackers with results

## Project Structure
- Source code: `d:\Workspace\PirateDownloader\src-tauri\`
- Memory root: `d:\Workspace\PirateDownloader\memory\`
- MVP2 tracking: `d:\Workspace\PirateDownloader\memory\mvp2\`
  - Active tasks: `mvp2/trackers/active-tracker.md`
  - Pending tasks: `mvp2/trackers/pending-tracker.md`
  - Implementation plans: `mvp2/plans/`
- MVP2 PRD: `memory/mvp2_prd.md`
- MVP1 history: `memory/mvp1/`

