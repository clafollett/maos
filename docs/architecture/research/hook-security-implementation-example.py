#!/usr/bin/env python3
"""
Hook-Based Security Implementation Example for Claude Code
Demonstrates a comprehensive security validation system using hooks
"""

import json
import sys
import re
import os
from pathlib import Path
from typing import Dict, List, Tuple, Set, Optional
from dataclasses import dataclass
from enum import Enum
import hashlib
import time

# Security Capabilities
class Capability(Enum):
    """Security capabilities that can be granted to agents"""
    READ_WORKSPACE = "read:workspace"
    WRITE_WORKSPACE = "write:workspace"
    READ_SYSTEM = "read:system"
    EXECUTE_SAFE = "execute:safe"
    EXECUTE_ANY = "execute:any"
    NETWORK_LOCAL = "network:local"
    NETWORK_EXTERNAL = "network:external"
    MODIFY_SETTINGS = "modify:settings"
    ACCESS_SECRETS = "access:secrets"

@dataclass
class SecurityPolicy:
    """Security policy configuration"""
    name: str
    capabilities: Set[Capability]
    path_whitelist: List[str]
    command_whitelist: List[str]
    blocked_patterns: List[Tuple[str, str]]  # (pattern, reason)
    max_file_size: int = 10 * 1024 * 1024  # 10MB
    rate_limit: Dict[str, int] = None  # operations per minute

class SecurityValidator:
    """Main security validation engine"""
    
    def __init__(self, policy: SecurityPolicy):
        self.policy = policy
        self.operation_history = []
        self.blocked_operations = []
        
    def validate_pre_tool_use(self, tool_name: str, tool_input: Dict) -> Tuple[bool, Optional[str]]:
        """Validate before tool execution"""
        
        # Tool-specific validation
        validators = {
            "Write": self._validate_write,
            "Edit": self._validate_edit,
            "Read": self._validate_read,
            "Bash": self._validate_bash,
            "WebSearch": self._validate_web,
            "WebFetch": self._validate_web,
        }
        
        if tool_name in validators:
            allowed, reason = validators[tool_name](tool_input)
            if not allowed:
                self.blocked_operations.append({
                    "timestamp": time.time(),
                    "tool": tool_name,
                    "reason": reason,
                    "input": tool_input
                })
                return False, reason
                
        # Rate limiting check
        if self._check_rate_limit(tool_name):
            return False, f"Rate limit exceeded for {tool_name}"
            
        # Record operation
        self.operation_history.append({
            "timestamp": time.time(),
            "tool": tool_name,
            "input": tool_input
        })
        
        return True, None
    
    def _validate_write(self, params: Dict) -> Tuple[bool, Optional[str]]:
        """Validate file write operations"""
        if Capability.WRITE_WORKSPACE not in self.policy.capabilities:
            return False, "Write capability not granted"
            
        file_path = params.get("file_path", "")
        content = params.get("content", "")
        
        # Path validation
        if not self._is_safe_path(file_path, require_workspace=True):
            return False, f"Path outside workspace: {file_path}"
            
        # Size validation
        if len(content.encode('utf-8')) > self.policy.max_file_size:
            return False, f"File too large (max {self.policy.max_file_size} bytes)"
            
        # Content validation
        for pattern, reason in self.policy.blocked_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                return False, f"Blocked content pattern: {reason}"
                
        # Sensitive file protection
        sensitive_patterns = [
            r'\.env$', r'\.ssh/', r'\.aws/', r'\.git/config',
            r'\.gpg$', r'\.key$', r'\.pem$', r'id_rsa'
        ]
        
        for pattern in sensitive_patterns:
            if re.search(pattern, file_path):
                if Capability.ACCESS_SECRETS not in self.policy.capabilities:
                    return False, f"Cannot write to sensitive file: {file_path}"
                    
        return True, None
    
    def _validate_edit(self, params: Dict) -> Tuple[bool, Optional[str]]:
        """Validate file edit operations - similar to write"""
        return self._validate_write(params)
    
    def _validate_read(self, params: Dict) -> Tuple[bool, Optional[str]]:
        """Validate file read operations"""
        file_path = params.get("file_path", "")
        
        # Check basic read permission
        if Capability.READ_SYSTEM not in self.policy.capabilities:
            if not self._is_safe_path(file_path, require_workspace=True):
                return False, f"Cannot read outside workspace: {file_path}"
                
        # Protect sensitive files
        if self._is_sensitive_file(file_path):
            if Capability.ACCESS_SECRETS not in self.policy.capabilities:
                return False, f"Cannot read sensitive file: {file_path}"
                
        return True, None
    
    def _validate_bash(self, params: Dict) -> Tuple[bool, Optional[str]]:
        """Validate bash command execution"""
        command = params.get("command", "")
        
        # Check execution capability
        if Capability.EXECUTE_ANY not in self.policy.capabilities:
            if Capability.EXECUTE_SAFE not in self.policy.capabilities:
                return False, "No execution capability granted"
                
            # Validate against whitelist
            if not self._is_safe_command(command):
                return False, f"Command not in whitelist: {command}"
        
        # Block dangerous patterns
        dangerous_patterns = [
            (r'\brm\s+-rf\s+/', "Dangerous recursive deletion"),
            (r'curl.*\|\s*sh', "Remote code execution attempt"),
            (r'wget.*\|\s*bash', "Remote code execution attempt"),
            (r'eval\s*\(', "Unsafe code evaluation"),
            (r'sudo\s+', "Privilege escalation attempt"),
            (r'chmod\s+777', "Unsafe permission change"),
            (r'mkfs\.', "Filesystem formatting attempt"),
            (r'dd\s+if=/dev/zero', "Disk wiping attempt"),
            (r':(){ :|:& };:', "Fork bomb detected"),
        ]
        
        for pattern, reason in dangerous_patterns:
            if re.search(pattern, command, re.IGNORECASE):
                return False, f"Blocked command: {reason}"
                
        return True, None
    
    def _validate_web(self, params: Dict) -> Tuple[bool, Optional[str]]:
        """Validate web access operations"""
        if Capability.NETWORK_EXTERNAL not in self.policy.capabilities:
            return False, "External network access not granted"
            
        url = params.get("url", params.get("query", ""))
        
        # Check for internal network access
        internal_patterns = [
            r'localhost', r'127\.0\.0\.1', r'0\.0\.0\.0',
            r'192\.168\.', r'10\.', r'172\.(1[6-9]|2[0-9]|3[01])\.',
            r'\.local$', r'\.internal$'
        ]
        
        for pattern in internal_patterns:
            if re.search(pattern, url, re.IGNORECASE):
                if Capability.NETWORK_LOCAL not in self.policy.capabilities:
                    return False, "Local network access not granted"
                    
        return True, None
    
    def _is_safe_path(self, path: str, require_workspace: bool = False) -> bool:
        """Check if path is safe to access"""
        try:
            # Resolve to absolute path
            abs_path = Path(path).resolve()
            
            # Check for path traversal
            if ".." in str(abs_path):
                return False
                
            # Check workspace requirement
            if require_workspace:
                workspace = Path.cwd()
                if not str(abs_path).startswith(str(workspace)):
                    return False
                    
            # Check against whitelist
            for allowed_path in self.policy.path_whitelist:
                if str(abs_path).startswith(allowed_path):
                    return True
                    
            return not require_workspace
            
        except Exception:
            return False
    
    def _is_safe_command(self, command: str) -> bool:
        """Check if command is in whitelist"""
        # Extract base command
        base_command = command.split()[0] if command else ""
        
        # Check exact matches
        if base_command in self.policy.command_whitelist:
            return True
            
        # Check pattern matches
        for allowed in self.policy.command_whitelist:
            if '*' in allowed:
                pattern = allowed.replace('*', '.*')
                if re.match(pattern, base_command):
                    return True
                    
        return False
    
    def _is_sensitive_file(self, path: str) -> bool:
        """Check if file contains sensitive information"""
        sensitive_patterns = [
            r'\.env', r'\.ssh/', r'\.aws/', r'\.git/config',
            r'\.gpg$', r'\.key$', r'\.pem$', r'id_rsa',
            r'credentials', r'secrets', r'password', r'token'
        ]
        
        for pattern in sensitive_patterns:
            if re.search(pattern, path, re.IGNORECASE):
                return True
                
        return False
    
    def _check_rate_limit(self, tool_name: str) -> bool:
        """Check if rate limit is exceeded"""
        if not self.policy.rate_limit:
            return False
            
        limit = self.policy.rate_limit.get(tool_name, float('inf'))
        
        # Count recent operations
        current_time = time.time()
        recent_ops = [
            op for op in self.operation_history
            if op["tool"] == tool_name and current_time - op["timestamp"] < 60
        ]
        
        return len(recent_ops) >= limit

# Pre-defined security policies
SECURITY_POLICIES = {
    "strict": SecurityPolicy(
        name="strict",
        capabilities={
            Capability.READ_WORKSPACE,
            Capability.WRITE_WORKSPACE,
            Capability.EXECUTE_SAFE,
        },
        path_whitelist=[os.getcwd()],
        command_whitelist=["ls", "cat", "grep", "find", "echo", "pwd"],
        blocked_patterns=[
            (r"api[_-]?key", "API key exposure risk"),
            (r"password\s*=", "Password exposure risk"),
            (r"BEGIN.*PRIVATE KEY", "Private key exposure risk"),
        ],
        rate_limit={"Write": 10, "Bash": 20}
    ),
    
    "standard": SecurityPolicy(
        name="standard",
        capabilities={
            Capability.READ_WORKSPACE,
            Capability.WRITE_WORKSPACE,
            Capability.READ_SYSTEM,
            Capability.EXECUTE_SAFE,
            Capability.NETWORK_LOCAL,
        },
        path_whitelist=[os.getcwd(), "/tmp", "/var/tmp"],
        command_whitelist=["ls", "cat", "grep", "find", "echo", "pwd", "git", "npm", "pip", "python*"],
        blocked_patterns=[
            (r"api[_-]?key", "API key exposure risk"),
            (r"BEGIN.*PRIVATE KEY", "Private key exposure risk"),
        ],
        rate_limit={"Write": 50, "Bash": 100}
    ),
    
    "permissive": SecurityPolicy(
        name="permissive",
        capabilities={
            Capability.READ_WORKSPACE,
            Capability.WRITE_WORKSPACE,
            Capability.READ_SYSTEM,
            Capability.EXECUTE_ANY,
            Capability.NETWORK_LOCAL,
            Capability.NETWORK_EXTERNAL,
        },
        path_whitelist=["/"],
        command_whitelist=["*"],
        blocked_patterns=[],
        rate_limit={}
    ),
}

def main():
    """Main hook entry point"""
    try:
        # Read input from stdin
        input_data = json.load(sys.stdin)
        
        # Determine security policy (from env or default)
        policy_name = os.getenv("CLAUDE_SECURITY_POLICY", "standard")
        policy = SECURITY_POLICIES.get(policy_name, SECURITY_POLICIES["standard"])
        
        # Create validator
        validator = SecurityValidator(policy)
        
        # Extract hook data
        hook_event = input_data.get("hook_event_name", "")
        tool_name = input_data.get("tool_name", "")
        tool_input = input_data.get("tool_input", {})
        
        # Validate based on hook type
        if hook_event == "PreToolUse":
            allowed, reason = validator.validate_pre_tool_use(tool_name, tool_input)
            
            if not allowed:
                # Return JSON response to block with reason
                response = {
                    "decision": "block",
                    "reason": reason
                }
                print(json.dumps(response))
                sys.exit(0)
            else:
                # Log the allowed operation
                print(f"Security check passed for {tool_name}", file=sys.stderr)
                sys.exit(0)
                
        elif hook_event == "PostToolUse":
            # Audit logging
            audit_entry = {
                "timestamp": time.time(),
                "tool": tool_name,
                "input": tool_input,
                "response": input_data.get("tool_response", {}),
                "policy": policy_name
            }
            
            # Could write to audit log here
            print(f"Audit: {tool_name} completed", file=sys.stderr)
            sys.exit(0)
            
        elif hook_event == "UserPromptSubmit":
            prompt = input_data.get("prompt", "")
            
            # Check for security policy override attempts
            if re.search(r"disable.*security|bypass.*protection", prompt, re.IGNORECASE):
                response = {
                    "decision": "block",
                    "reason": "Security policy override attempt detected"
                }
                print(json.dumps(response))
                sys.exit(0)
                
            # Add security context
            print(f"Security Policy: {policy_name}")
            print(f"Granted Capabilities: {[cap.value for cap in policy.capabilities]}")
            sys.exit(0)
            
    except Exception as e:
        # Fail open on errors to avoid blocking legitimate work
        print(f"Security hook error: {e}", file=sys.stderr)
        sys.exit(0)

if __name__ == "__main__":
    main()