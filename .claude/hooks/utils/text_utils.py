#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.8"
# ///

import re
import sys
from pathlib import Path

# Import config for text length limit
sys.path.append(str(Path(__file__).parent))
from config import get_text_length_limit

def preserve_inline_code_content(text):
    """
    Intelligently handle inline code - preserve the content for speech while removing wrapper tokens.
    
    Examples:
    - "the `except` block" -> "the except block"  
    - "use `get_config()` function" -> "use get_config() function"
    - "set `timeout=30`" -> "set timeout=30"
    
    This preserves important technical terms that should be spoken naturally.
    """
    def extract_inline_content(match):
        content = match.group(1)
        
        # If it's a single word (likely a keyword), preserve it
        if re.match(r'^\w+$', content):
            return content
            
        # If it's a simple function call or variable, preserve it
        if re.match(r'^[\w_]+\(\)$', content) or re.match(r'^[\w_]+=[\w_]+$', content):
            return content
            
        # If it's complex code (multiple operators, etc.), remove it
        if any(char in content for char in [';', '{', '}', '[', ']', '&&', '||']):
            return ' '
            
        # Default: preserve simple technical terms
        return content
    
    # Apply intelligent inline code handling
    return re.sub(r'`([^`]+)`', extract_inline_content, text)

def clean_text_for_speech(text):
    """
    Clean text to make it suitable for TTS with natural speech markup.
    Remove code blocks, excessive formatting, and add speech pauses/emphasis.
    """
    # Remove code blocks (```...```) entirely
    text = re.sub(r'```[\s\S]*?```', '', text)
    
    # Intelligently handle inline code - preserve content, strip wrapper tokens
    text = preserve_inline_code_content(text)
    
    # Remove markdown links [text](url)
    text = re.sub(r'\[([^\]]+)\]\([^\)]+\)', r'\1', text)
    
    # Remove markdown headers (#, ##, ###)
    text = re.sub(r'^#{1,6}\s+', '', text, flags=re.MULTILINE)
    
    # Convert bullet points and list markers to natural speech with pauses
    text = re.sub(r'^\s*[-*+]\s+', '', text, flags=re.MULTILINE)
    text = re.sub(r'^\s*\d+\.\s+', '', text, flags=re.MULTILINE)
    
    # Remove tool call indicators and XML-like tags
    text = re.sub(r'<[^>]+>', '', text)
    
    # Remove all emojis for better speech
    text = remove_emojis(text)
    
    # Add natural speech markup before cleaning whitespace
    text = add_speech_markup(text)
    
    # Convert newlines to pauses (for list items and natural breaks)
    text = re.sub(r'\n', ' ... ', text)
    
    # Remove excessive whitespace but preserve our added pauses
    text = re.sub(r'\s+', ' ', text)
    
    # Clean up multiple consecutive pauses
    text = re.sub(r'(\.\.\.\s*){2,}', '... ', text)
    
    # Clean up and limit length
    text = text.strip()
    
    # Limit to configured TTS length with fallback
    try:
        text_limit = get_text_length_limit()
    except Exception:
        text_limit = 2000  # Fallback if config unavailable
        
    if len(text) > text_limit:
        text = text[:text_limit] + "..."
    
    return text

def add_speech_markup(text):
    """
    Add cross-platform speech markup for natural pauses and emphasis.
    Uses simple punctuation that works across TTS engines.
    """
    # Add pause after exclamation marks and question marks
    text = re.sub(r'([!?])\s+', r'\1... ', text)
    
    # Add slight pause after periods (but not abbreviations)
    text = re.sub(r'(\w)\.(\s+[A-Z])', r'\1... \2', text)
    
    # Add pause after colons
    text = re.sub(r':\s+', ': ... ', text)
    
    # Add emphasis for text in **bold** (convert to caps with pauses)
    text = re.sub(r'\*\*([^*]+)\*\*', r'... \1 ...', text)
    
    # Add emphasis for text in *italics* (add slight pauses)
    text = re.sub(r'\*([^*]+)\*', r'... \1', text)
    
    # Add pause after parenthetical statements
    text = re.sub(r'\)(\s+)', r') ... \1', text)
    
    # Add pause before parenthetical statements
    text = re.sub(r'(\s+)\(', r'\1... (', text)
    
    # Convert dashes to natural pauses
    text = re.sub(r'\s+--?\s+', ' ... ', text)
    
    return text

def remove_emojis(text):
    """
    Remove all Unicode emoji ranges from text for clean TTS speech.
    This regex matches most Unicode emoji ranges.
    """
    # Remove various Unicode emoji ranges
    text = re.sub(r'[\U0001F600-\U0001F64F]', '', text)  # emoticons
    text = re.sub(r'[\U0001F300-\U0001F5FF]', '', text)  # symbols & pictographs
    text = re.sub(r'[\U0001F680-\U0001F6FF]', '', text)  # transport & map
    text = re.sub(r'[\U0001F1E0-\U0001F1FF]', '', text)  # flags
    text = re.sub(r'[\U00002600-\U000027BF]', '', text)  # misc symbols
    text = re.sub(r'[\U0001F900-\U0001F9FF]', '', text)  # supplemental symbols
    
    return text

def variable_to_speech(var_name):
    """
    Convert variable names and technical terms to natural speech.
    
    Examples:
    - session_id -> "session I D"
    - stop_hook_active -> "stop hook active"
    - apiKey -> "A P I key"
    """
    # Split on underscores and camelCase
    words = re.sub(r'([A-Z])', r' \1', var_name).split('_')
    words = [word.strip() for word in words if word.strip()]
    
    # Convert common programming terms to speech-friendly versions
    replacements = {
        'id': 'I D',
        'url': 'U R L', 
        'api': 'A P I',
        'tts': 'text to speech',
        'config': 'configuration',
        'json': 'jay-sawn',  # JSON sounds like "jay-sawn"
        'xml': 'X M L',
        'html': 'H T M L',
        'css': 'C S S',
        'js': 'java script',
        'py': 'python',
        'sql': 'sequel',  # SQL commonly pronounced as "sequel"
        'db': 'database',
        'auth': 'authentication',
        'oauth': 'O auth',
        'jwt': 'J W T',
        'uuid': 'U U I D',
        'crud': 'crud'  # CRUD sounds like "crud"
    }
    
    result_words = []
    for word in words:
        lower_word = word.lower()
        if lower_word in replacements:
            result_words.append(replacements[lower_word])
        else:
            result_words.append(word)
    
    return ' '.join(result_words)