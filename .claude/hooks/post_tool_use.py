#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "python-dotenv",
# ]
# ///

import json
import sys
import asyncio
import time
import threading
import subprocess
from pathlib import Path
from typing import Dict, Optional
from concurrent.futures import ThreadPoolExecutor

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass  # dotenv is optional

# Import shared path utilities
sys.path.append(str(Path(__file__).parent))
from utils.path_utils import PROJECT_ROOT, LOGS_DIR
from utils.async_logging import log_hook_data, log_hook_data_sync, get_task_manager

# Import MAOS handler
sys.path.append(str(Path(__file__).parent))
try:
    from maos.post_tool_handler import handle_maos_post_tool
except ImportError:
    # Fallback if MAOS not available
    handle_maos_post_tool = None

async def run_maos_post_background(tool_name: str, tool_input: Dict, tool_response: Dict, hook_metadata: Dict) -> None:
    """Run MAOS post-processing in background without blocking."""
    if not handle_maos_post_tool:
        return
    
    def maos_task():
        try:
            handle_maos_post_tool(tool_name, tool_input, tool_response, hook_metadata)
        except Exception as e:
            print(f"⚠️  MAOS post-processing error (background): {e}", file=sys.stderr)
    
    # Run MAOS in background task manager
    task_manager = get_task_manager()
    await task_manager.run_background_task(maos_task, timeout=15.0)


async def run_rust_tooling_background(file_path: str) -> None:
    """Run Rust formatting and linting in background process pool."""
    if not file_path.endswith('.rs'):
        return
    
    def rust_tooling():
        try:
            print("🦀 Formatting and linting Rust code (background)...", file=sys.stderr)
            
            # Run cargo fmt
            result_fmt = subprocess.run(
                ['cargo', 'fmt'], 
                check=False, 
                cwd=PROJECT_ROOT,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Run cargo clippy with fixes
            result_clippy = subprocess.run([
                'cargo', 'clippy', 
                '--fix', '--allow-dirty', '--allow-staged', 
                '--', '-D', 'warnings'
            ], 
            check=False, 
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
            timeout=60
            )
            
            if result_fmt.returncode == 0 and result_clippy.returncode == 0:
                print("✅ Rust formatting and linting complete (background)", file=sys.stderr)
            else:
                print(f"⚠️  Rust tooling warnings (background): fmt={result_fmt.returncode}, clippy={result_clippy.returncode}", file=sys.stderr)
                
        except subprocess.TimeoutExpired:
            print("⏰ Rust tooling timeout (background) - continuing", file=sys.stderr)
        except Exception as e:
            print(f"⚠️  Rust tooling error (background): {e}", file=sys.stderr)
    
    # Run Rust tooling in background task manager with extended timeout
    task_manager = get_task_manager()
    await task_manager.run_background_task(rust_tooling, timeout=90.0)


async def main_async():
    """ASYNC main function - immediate response, background processing."""
    try:
        start_time = time.time()
        
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)
        
        tool_name = input_data.get('tool_name', '')
        tool_input = input_data.get('tool_input', {})
        tool_response = input_data.get('tool_response', {})
        hook_metadata = input_data.get('metadata', {})
        
        # 🚀 IMMEDIATE RESPONSE - All processing in background
        
        # Start all background tasks in parallel
        background_tasks = []
        
        # MAOS post-processing in background
        if handle_maos_post_tool:
            maos_task = asyncio.create_task(
                run_maos_post_background(tool_name, tool_input, tool_response, hook_metadata)
            )
            background_tasks.append(maos_task)
        
        # Rust tooling for .rs files (most expensive operation)
        if tool_name in ['Edit', 'MultiEdit']:
            file_path = tool_input.get('file_path', '')
            if file_path.endswith('.rs'):
                rust_task = asyncio.create_task(run_rust_tooling_background(file_path))
                background_tasks.append(rust_task)
        
        # Async logging (JSONL format for true append-only)
        log_path = LOGS_DIR / 'post_tool_use.jsonl'  # Changed to .jsonl
        logging_task = asyncio.create_task(log_hook_data(log_path, input_data))
        background_tasks.append(logging_task)
        
        # Give background tasks a moment to start, but don't wait for completion
        if background_tasks:
            # Wait just long enough to ensure tasks are started (non-blocking)
            try:
                await asyncio.wait_for(
                    asyncio.gather(*background_tasks, return_exceptions=True),
                    timeout=0.1  # Very short timeout - just to start tasks
                )
            except asyncio.TimeoutError:
                # This is expected - we don't want to block
                pass
        
        total_time = time.time() - start_time
        print(f"⚡ Post-tool hook completed in {total_time*1000:.2f}ms (background tasks running)", file=sys.stderr)
        
        sys.exit(0)
        
    except json.JSONDecodeError:
        # Handle JSON decode errors gracefully
        sys.exit(0)
    except Exception as e:
        # Handle any other errors gracefully - don't block operations
        print(f"⚠️  Post-tool hook error (non-blocking): {e}", file=sys.stderr)
        sys.exit(0)


def main():
    """Wrapper to run async main with fallback."""
    try:
        # Try async version first
        try:
            asyncio.get_running_loop()
            # Loop already running, create new thread
            result = [None]
            exception = [None]
            
            def run_async():
                try:
                    new_loop = asyncio.new_event_loop()
                    asyncio.set_event_loop(new_loop)
                    result[0] = new_loop.run_until_complete(main_async())
                except Exception as e:
                    exception[0] = e
                finally:
                    new_loop.close()
            
            thread = threading.Thread(target=run_async)
            thread.start()
            thread.join()
            
            if exception[0]:
                raise exception[0]
                
        except RuntimeError:
            # No running event loop - normal case
            asyncio.run(main_async())
            
    except Exception:
        # Fallback to synchronous version
        main_sync()


def main_sync():
    """Fallback synchronous version with optimized I/O."""
    try:
        start_time = time.time()
        
        # Read JSON input from stdin
        input_data = json.load(sys.stdin)
        
        tool_name = input_data.get('tool_name', '')
        tool_input = input_data.get('tool_input', {})
        tool_response = input_data.get('tool_response', {})
        hook_metadata = input_data.get('metadata', {})
        
        # 🚀 BACKGROUND PROCESSING (sync version)
        
        def background_work():
            # MAOS post-processing
            if handle_maos_post_tool:
                try:
                    handle_maos_post_tool(tool_name, tool_input, tool_response, hook_metadata)
                except Exception as e:
                    print(f"⚠️  MAOS post-processing error (background): {e}", file=sys.stderr)
            
            # Rust tooling for .rs files
            if tool_name in ['Edit', 'MultiEdit']:
                file_path = tool_input.get('file_path', '')
                if file_path.endswith('.rs'):
                    try:
                        print("🦀 Formatting and linting Rust code (background)...", file=sys.stderr)
                        
                        # Run cargo fmt
                        subprocess.run(['cargo', 'fmt'], check=False, cwd=PROJECT_ROOT, timeout=30)
                        
                        # Run cargo clippy with fixes
                        subprocess.run([
                            'cargo', 'clippy', 
                            '--fix', '--allow-dirty', '--allow-staged', 
                            '--', '-D', 'warnings'
                        ], check=False, cwd=PROJECT_ROOT, timeout=60)
                        
                        print("✅ Rust formatting and linting complete (background)", file=sys.stderr)
                    except subprocess.TimeoutExpired:
                        print("⏰ Rust tooling timeout (background)", file=sys.stderr)
                    except Exception as e:
                        print(f"⚠️  Rust tooling error (background): {e}", file=sys.stderr)
            
            # Fast JSONL logging
            log_path = LOGS_DIR / 'post_tool_use.jsonl'
            log_hook_data_sync(log_path, input_data)
        
        # Start background thread
        bg_thread = threading.Thread(target=background_work, daemon=True)
        bg_thread.start()
        
        # Give background thread a tiny moment to start
        bg_thread.join(timeout=0.05)
        
        total_time = time.time() - start_time
        print(f"⚡ Post-tool hook completed in {total_time*1000:.2f}ms (sync fallback)", file=sys.stderr)
        
        sys.exit(0)
        
    except json.JSONDecodeError:
        sys.exit(0)
    except Exception as e:
        print(f"⚠️  Post-tool hook error (sync fallback): {e}", file=sys.stderr)
        sys.exit(0)

if __name__ == '__main__':
    main()