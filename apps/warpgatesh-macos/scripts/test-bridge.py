#!/usr/bin/env python3
"""Read-only smoke test of the real private backend; never initializes the agent."""
import json
import os
import pathlib
import select
import subprocess
import sys

binary = pathlib.Path(sys.argv[1]).resolve()
process = subprocess.Popen([str(binary), "--native-backend", "--native-parent", str(os.getpid())], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, bufsize=0)
try:
    def request(value):
        process.stdin.write(json.dumps(value).encode() + b"\n")
        ready, _, _ = select.select([process.stdout], [], [], 20)
        if not ready:
            raise RuntimeError("Backend response timed out")
        line = process.stdout.readline()
        if not line:
            raise RuntimeError("Backend exited before replying")
        return json.loads(line)

    state = request({"id": 1, "command": "get_companion_state"})
    assert state["id"] == 1 and "result" in state, "State request failed"
    assert all(key in state["result"] for key in ["agentRunning", "profiles", "targets", "preferences", "update", "alerts"])
    invalid = request({"id": 2, "command": "unknown_operation"})
    assert invalid["id"] == 2 and "error" in invalid
    unapproved = request({"id": 3, "command": "add_profile", "args": {"request": {"name": "fixture", "baseUrl": "https://example.invalid", "token": "fixture-not-a-real-token"}}})
    assert unapproved["id"] == 3 and "error" in unapproved
    assert "fixture-not-a-real-token" not in json.dumps(unapproved)
    unapproved_session = request({"id": 4, "command": "add_profile", "args": {"request": {"name": "fixture", "baseUrl": "https://example.invalid", "sessionCookie": "warpgate-http-session=fixture-session-secret"}}})
    assert unapproved_session["id"] == 4 and "error" in unapproved_session
    assert "fixture-session-secret" not in json.dumps(unapproved_session)
    invalid_session = request({"id": 5, "command": "rotate_profile_token", "args": {"name": "fixture-missing", "session_cookie": "warpgate-http-session=fixture-session-secret"}})
    assert invalid_session["id"] == 5 and "error" in invalid_session
    assert "fixture-session-secret" not in json.dumps(invalid_session)
    process.stdin.close()
    process.wait(timeout=10)
    assert process.returncode == 0
    print("Native bridge: state schema, correlated errors, required SSH approval, secret exclusion and clean EOF verified")
finally:
    if process.poll() is None:
        process.terminate()
        process.wait(timeout=10)
