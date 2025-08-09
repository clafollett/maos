---
allowed-tools: Bash
description: Stop all active TTS (Text-to-Speech) immediately
---

# Stop TTS (STFU)

Stop all active Text-to-Speech processes immediately.

## Kill TTS Processes

Execute the TTS kill script to stop all speech:

!`uv run .claude/hooks/utils/kill_tts.py`

Confirm what was stopped and provide feedback to the user about the result.