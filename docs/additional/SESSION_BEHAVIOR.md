# Terminal Lock - Session Behavior Explained

## 🎯 Core Concept: Per-Terminal Authentication

**IMPORTANT:** Each terminal window is **completely independent** and requires its own authentication.

## How Sessions Work

### Session Basics

- ✅ Each terminal has its **own session file** (based on Process ID)
- ✅ Each session lasts **15 minutes** from last activity
- ✅ Sessions are **NOT shared** between terminals
- ✅ Opening a new terminal **ALWAYS requires authentication**

### Session File Structure

```
~/.terminal_lock/
├── session.12345.json    # Terminal 1 (PID 12345)
├── session.12346.json    # Terminal 2 (PID 12346)
├── session.12347.json    # Terminal 3 (PID 12347)
└── ...
```

Each terminal gets its own session file based on its Process ID (PID).

## Real-World Scenarios

### ✅ Scenario 1: Open First Terminal

```
Time: 9:00 AM

Action: Open Terminal 1
Result: 
  → Prompts for password ✓
  → Enter password: "Pass123"
  → Shows greeting and quote
  → Session created (valid until 9:15 AM)
  → Can use Terminal 1 freely until 9:15 AM
```

### ✅ Scenario 2: Open Second Terminal (While First is Open)

```
Time: 9:05 AM (Terminal 1 still open and authenticated)

Action: Open Terminal 2
Result:
  → Prompts for password ✓ (ALWAYS!)
  → Enter password: "Pass123"
  → Shows greeting and quote
  → New session created for Terminal 2 (valid until 9:20 AM)
  → Terminal 1: Still valid until 9:15 AM
  → Terminal 2: Valid until 9:20 AM
```

**Why?** Security! Each terminal is independent. If someone gains access to your computer, they can't just open a new terminal and bypass authentication.

### ✅ Scenario 3: Use Terminal Within 15 Minutes

```
Time: 9:10 AM (5 minutes after authentication)

Action: Type command in Terminal 1: "ls"
Result:
  → Command executes immediately ✓
  → No password prompt
  → Session extends to 9:25 AM (new 15 minutes)
```

### ✅ Scenario 4: Use Terminal After 15 Minutes

```
Time: 9:30 AM (Terminal 1 has been idle since 9:15 AM)

Action: Type command in Terminal 1: "ls"
Result:
  → Session expired! ⏰
  → Prompts for password ✓
  → Enter password
  → New session created (valid until 9:45 AM)
  → Command executes
```

### ✅ Scenario 5: Multiple Terminals at Different Times

```
Time: 9:00 AM
Terminal 1: Open → Auth → Session until 9:15 AM

Time: 9:05 AM  
Terminal 2: Open → Auth → Session until 9:20 AM

Time: 9:10 AM
Terminal 3: Open → Auth → Session until 9:25 AM

Time: 9:16 AM
Terminal 1: Type command → Session expired → Re-auth required
Terminal 2: Type command → Still valid (expires 9:20 AM)
Terminal 3: Type command → Still valid (expires 9:25 AM)
```

## Why This Design?

### Security Benefits

1. **No Session Hijacking**: Can't open new terminal to bypass auth
2. **Time-Limited Access**: Each terminal session expires independently
3. **Granular Control**: Each terminal tracks its own timeout
4. **Attack Prevention**: Attacker can't leverage existing sessions

### Practical Benefits

1. **Clear Security**: You always know when a terminal needs re-auth
2. **Independent Work**: Each terminal window is self-contained
3. **No Confusion**: Session state is per-terminal, easy to understand

## Common Misconceptions

### ❌ WRONG: "Once authenticated, all terminals are unlocked"

**NO!** Each terminal must authenticate separately.

### ❌ WRONG: "Opening 5 terminals in 1 minute = only authenticate once"

**NO!** Each terminal requires its own authentication, even if opened simultaneously.

### ❌ WRONG: "15-minute session = I can open unlimited terminals for 15 minutes"

**NO!** The 15 minutes is **per terminal**, not global.

## Correct Understanding

### ✅ RIGHT: "Each terminal window needs its own password"

**YES!** Opening Terminal 1, 2, 3, 4, 5 = entering password 5 times.

### ✅ RIGHT: "15 minutes per terminal, not globally"

**YES!** Each terminal has its own independent 15-minute countdown.

### ✅ RIGHT: "Using a terminal extends ITS session, not others"

**YES!** Activity in Terminal 1 only affects Terminal 1's session.

## Session Timeline Example

```
Terminal 1:
├── 9:00 → Open + Auth → Session until 9:15
├── 9:10 → Use command → Session extends to 9:25
├── 9:20 → Use command → Session extends to 9:35
└── 9:50 → Use command → Session expired → Re-auth

Terminal 2:
├── 9:05 → Open + Auth → Session until 9:20
├── 9:15 → Use command → Session extends to 9:30
└── 9:45 → Use command → Session expired → Re-auth

Terminal 3:
├── 9:30 → Open + Auth → Session until 9:45
└── 9:40 → Use command → Session extends to 9:55
```

## How to Check Your Session

```bash
terminal-lock status
```

Output shows:
```
Session Status:
  ✓ Session is ACTIVE
  ℹ Timeout: 15 minutes
  ℹ Expires: 9:25 AM
```

This is for **THIS specific terminal only**.

## Session Extension Rules

### When Session Extends:

1. ✅ You type a command
2. ✅ You use the terminal
3. ✅ Session is validated

### When Session Does NOT Extend:

1. ❌ Terminal is idle (no commands)
2. ❌ You're using a different terminal
3. ❌ Session already expired

### Extension Logic:

```
Current time: 9:10 AM
Session expires: 9:15 AM
Time remaining: 5 minutes

Action: Type command
Result: Session extends to 9:25 AM (NEW 15 minutes from NOW)
```

## Frequently Asked Questions

### Q: "Why do I need to authenticate each new terminal?"

**A:** Security! This prevents:
- Session hijacking
- Unauthorized access via new terminals
- Bypassing authentication by opening multiple windows

### Q: "Can I share sessions between terminals?"

**A:** No, by design. Each terminal is independent for security reasons.

### Q: "Is there a way to authenticate once for all terminals?"

**A:** No. This would defeat the purpose of per-terminal security. Each terminal must authenticate.

### Q: "What if I have 10 terminals open?"

**A:** You'll authenticate 10 times (once per terminal). Each maintains its own 15-minute session.

### Q: "Does closing a terminal affect others?"

**A:** No. Each terminal's session is independent. Closing Terminal 1 doesn't affect Terminal 2's session.

## Technical Details

### Session File Format

```json
{
  "expiry": "2025-11-16T09:15:00Z",
  "terminal_id": "12345"
}
```

- `expiry`: When this terminal's session expires (UTC)
- `terminal_id`: Process ID of this specific terminal

### Session Validation

1. Check if `session.<PID>.json` exists
2. Read expiry time
3. Compare with current time
4. If expired: Delete file, require re-auth
5. If valid: Extend expiry, continue

### Session Cleanup

Old session files are automatically cleaned up when:
- Terminal closes (session file deleted)
- System reboots (all session files cleared)
- Manual reset (`terminal-lock reset`)

## Best Practices

### For Users:

1. ✅ Expect to authenticate each new terminal
2. ✅ Don't be surprised by multiple password prompts
3. ✅ Understand each terminal has its own 15-minute timer
4. ✅ Use `terminal-lock status` to check individual terminal session

### For Developers:

1. ✅ Each terminal creates `session.<PID>.json`
2. ✅ Never share sessions across PIDs
3. ✅ Always validate PID matches session file
4. ✅ Clean up session files on terminal close

## Summary

| Aspect | Behavior |
|--------|----------|
| **Session Scope** | Per-terminal (PID-based) |
| **Session Duration** | 15 minutes from last activity |
| **Multiple Terminals** | Each requires authentication |
| **Session Sharing** | NO - completely independent |
| **Extension** | Per-terminal, not global |
| **Authentication** | Required for EVERY new terminal |

## The Bottom Line

**Every terminal window = separate authentication**

This is intentional, secure, and the correct behavior!