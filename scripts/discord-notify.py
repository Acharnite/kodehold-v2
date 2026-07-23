#!/usr/bin/env python3
"""Discord notification script for KodeHold loops.

Usage:
    python3 scripts/discord-notify.py <workspace> <pattern> <outcome> <duration> [findings_file]
"""

import sys
import json
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

WEBHOOK_URL = "https://discord.com/api/webhooks/1529351013541220403/T9Raz0O2pj6ubKFoeh30xvY_mjozRCudJa39iuTsQU_5FctdWGLo7lOu_wFR2hkHgmxY"

def extract_findings(log_file: str) -> list[str]:
    """Extract findings from loop run log file."""
    findings = []
    try:
        with open(log_file, 'r') as f:
            content = f.read()
        
        # Look for common patterns in loop output
        lines = content.split('\n')
        for line in lines:
            line = line.strip()
            # Look for uncommitted files
            if 'uncommitted' in line.lower() and ('file' in line.lower() or 'change' in line.lower()):
                findings.append(line)
            # Look for stale PRs
            elif 'stale' in line.lower() and 'pr' in line.lower():
                findings.append(line)
            # Look for pending reviews
            elif 'pending' in line.lower() and 'review' in line.lower():
                findings.append(line)
            # Look for test failures
            elif 'test' in line.lower() and ('fail' in line.lower() or 'error' in line.lower()):
                findings.append(line)
            # Look for issues found
            elif 'issues found' in line.lower() or 'finding' in line.lower():
                findings.append(line)
            # Look for warnings
            elif line.startswith('⚠️') or line.startswith('!') or 'warning' in line.lower():
                findings.append(line)
            # Look for high priority items
            elif 'high priority' in line.lower():
                findings.append(line)
            # Look for watch list items
            elif 'watch' in line.lower() and ('list' in line.lower() or 'item' in line.lower()):
                findings.append(line)
        
        # If no specific findings found, look for bullet points or numbered items
        if not findings:
            for line in lines:
                line = line.strip()
                if line.startswith(('•', '-', '*', '1.', '2.', '3.', '4.', '5.')):
                    findings.append(line)
        
        # Limit to 5 most important findings
        return findings[:5]
    except Exception:
        return []

def send_notification(workspace: str, pattern: str, outcome: str, duration: int, findings: list[str]) -> bool:
    """Send detailed Discord notification."""
    # Determine color based on outcome
    if "clean" in outcome.lower() or "success" in outcome.lower():
        color = 3066993  # Green
        emoji = "✅"
    elif "error" in outcome.lower() or "fail" in outcome.lower():
        color = 15158332  # Red
        emoji = "❌"
    else:
        color = 16776960  # Yellow
        emoji = "⚠️"
    
    # Build description
    description = f"""**Outcome:** {emoji} {outcome}
**Duration:** {duration}s"""
    
    # Add findings if available
    if findings:
        description += f"\n**Findings ({len(findings)}):**"
        for finding in findings:
            description += f"\n• {finding}"
    
    embed = {
        "embeds": [
            {
                "title": f"{workspace} — {pattern}",
                "description": description,
                "color": color,
                "author": {"name": "KodeHold Loop Engine"},
                "footer": {"text": f"kiffer/project/kodehold | loop_runner.py | {pattern}"},
                "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            }
        ]
    }
    
    data = json.dumps(embed).encode("utf-8")
    req = urllib.request.Request(
        WEBHOOK_URL,
        data=data,
        headers={
            "Content-Type": "application/json",
            "User-Agent": "KodeHold/1.0",
        },
    )
    
    try:
        urllib.request.urlopen(req, timeout=10)
        return True
    except Exception as e:
        print(f"[WARN] Discord webhook failed: {e}", file=sys.stderr)
        return False

if __name__ == "__main__":
    if len(sys.argv) < 5:
        print("Usage: discord-notify.py <workspace> <pattern> <outcome> <duration> [findings_file]")
        sys.exit(1)
    
    workspace = sys.argv[1]
    pattern = sys.argv[2]
    outcome = sys.argv[3]
    duration = int(sys.argv[4])
    
    # Extract findings from log file if provided
    findings = []
    if len(sys.argv) > 5:
        log_file = sys.argv[5]
        findings = extract_findings(log_file)
    
    success = send_notification(workspace, pattern, outcome, duration, findings)
    sys.exit(0 if success else 1)
