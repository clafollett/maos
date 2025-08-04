# MAOS Text-to-Speech (TTS) System

## Overview

MAOS includes a sophisticated multi-provider TTS system that provides audio feedback during development sessions. The system automatically selects the best available provider based on API keys and falls back gracefully when providers are unavailable.

## Provider Hierarchy

MAOS checks providers in this order:

1. **ElevenLabs** (Highest quality, requires API key)
2. **OpenAI** (Good quality, requires API key)
3. **macOS Say** (System default on Mac)
4. **pyttsx3** (Cross-platform fallback)

## Configuration

### Environment Variables

```bash
# Enable ElevenLabs (recommended for best quality)
export ELEVENLABS_API_KEY="your-api-key"

# Enable OpenAI TTS
export OPENAI_API_KEY="your-api-key"

# Personalize announcements
export ENGINEER_NAME="Alice"
```

### Project Configuration

Configure TTS behavior in `.claude/config.json`:

```json
{
  "maos": {
    "tts": {
      "enabled": true,
      "provider": "auto",  // or "elevenlabs", "openai", "macos", "pyttsx3"
      "voice": "adam",     // Provider-specific voice selection
      "response_tts": false,  // Read AI responses aloud
      "completion_tts": true, // Announce task completion
      "notification_tts": true // General notifications
    }
  }
}
```

## Voice Options

### ElevenLabs Voices
- `adam` - Natural male voice (default)
- `antoni` - Friendly male voice
- `arnold` - Deep male voice
- `bella` - Warm female voice
- `domi` - Energetic female voice
- `elli` - Young female voice
- `josh` - Casual male voice
- `rachel` - Professional female voice

### OpenAI Voices
- `alloy` - Neutral voice (default)
- `echo` - Male voice
- `fable` - British accent
- `onyx` - Deep voice
- `nova` - Female voice
- `shimmer` - Expressive female voice

### macOS Voices
- System default voice
- Any installed macOS voice (e.g., "Alex", "Samantha")

## Usage Scenarios

### 1. Task Completion Announcements

When a task completes, MAOS announces:
```
"[Engineer Name], backend engineer has completed their task"
```

### 2. Session Notifications

Important events during the session:
```
"Starting new orchestration session"
"All agents have completed their work"
"Session cleanup in progress"
```

### 3. Error Notifications

Critical errors that need attention:
```
"Security violation: blocked dangerous command"
"Git conflict detected in frontend workspace"
```

### 4. Response Reading (Optional)

When enabled, MAOS can read Claude's responses:
```json
{
  "maos": {
    "tts": {
      "response_tts": true,
      "response_max_length": 500  // Truncate long responses
    }
  }
}
```

## Implementation Details

### TTS Module Architecture

The [`maos-tts`](../architecture/rust-cli-architecture.md#maos-tts) crate provides a unified interface:

```rust
pub trait TtsProvider {
    fn speak(&self, text: &str) -> Result<(), TtsError>;
    fn is_available(&self) -> bool;
    fn priority(&self) -> u8;
}

pub struct TtsManager {
    providers: Vec<Box<dyn TtsProvider>>,
}

impl TtsManager {
    pub fn speak(&self, text: &str) {
        // Try providers in priority order
        for provider in &self.providers {
            if provider.is_available() {
                if provider.speak(text).is_ok() {
                    return;
                }
            }
        }
    }
}
```

### Performance Optimization

TTS operations are non-blocking:
- Audio generation happens in background thread
- Command execution continues immediately
- Queued announcements for multiple events
- Automatic text truncation for long content

### Text Processing

Before speaking, text is processed for clarity:

1. **Code removal**: Strip code blocks and technical syntax
2. **Abbreviation expansion**: "env" → "environment"
3. **Number formatting**: "404" → "four zero four"
4. **Punctuation optimization**: Better pauses and emphasis

## Platform-Specific Notes

### macOS
- Uses native `say` command via shell
- Respects system voice settings
- No additional dependencies required

### Linux
- Requires `pyttsx3` with `espeak` backend
- May need: `sudo apt-get install espeak`
- Alternative: `festival` TTS engine

### Windows
- Uses Windows SAPI through `pyttsx3`
- Works with built-in Windows voices
- No additional setup required

## Customization

### Custom Announcement Templates

```json
{
  "maos": {
    "tts": {
      "templates": {
        "task_complete": "Hey {engineer_name}, {agent_type} just finished {task_description}",
        "session_start": "Starting MAOS session {session_id}",
        "error": "Uh oh! {error_type}: {error_message}"
      }
    }
  }
}
```

### Voice Speed and Pitch

```json
{
  "maos": {
    "tts": {
      "rate": 150,     // Words per minute
      "pitch": 1.0,    // Voice pitch multiplier
      "volume": 0.8    // Volume level (0.0 - 1.0)
    }
  }
}
```

## Troubleshooting

### No Audio Output

1. Check API keys are set correctly
2. Verify system audio is not muted
3. Test with `maos tts-test "Hello world"`
4. Check logs in `.maos/logs/tts.log`

### Wrong Voice

1. Verify voice name is spelled correctly
2. Check provider supports the voice
3. Fall back to provider default

### Performance Issues

1. Disable response TTS for long outputs
2. Reduce text processing complexity
3. Use local providers (macOS say) instead of API calls

## Privacy Considerations

- **API providers**: Text is sent to ElevenLabs/OpenAI servers
- **Local providers**: All processing happens on your machine
- **No persistent storage**: Audio is not saved
- **Configurable**: Can be completely disabled

## Future Enhancements

- [ ] Local AI voice models (Coqui TTS)
- [ ] Voice cloning for personalized announcements
- [ ] Multi-language support
- [ ] Emotion detection for appropriate tone
- [ ] WebSocket streaming for real-time updates
- [ ] Voice commands for controlling MAOS

## Related Documentation

- [Configuration Guide](../cli/configuration.md) - TTS settings
- [Commands Reference](../cli/commands.md) - TTS in notify/stop commands
- [Architecture](../architecture/rust-cli-architecture.md) - TTS module design