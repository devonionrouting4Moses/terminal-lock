# Terminal Lock - Visual Session Behavior

## 🎨 Session Flow Diagram

### Opening Multiple Terminals

```
User Opens Terminal 1
         │
         ▼
   ┌─────────────┐
   │ No Session  │
   │ (PID 12345) │
   └──────┬──────┘
          │
          ▼
   ┌──────────────────┐
   │ Prompt Password  │
   │ 🔐 Enter: Pass123│
   └────────┬─────────┘
            │
            ▼
     ┌─────────────────────┐
     │ ✓ Authentication OK  │
     │ Create session.12345 │
     │ Valid until 9:15 AM  │
     └─────────────────────┘
            │
            ▼
    ┌──────────────────┐
    │ Terminal 1 Ready │
    │ Can use freely   │
    └──────────────────┘

─────────────────────────────────────

User Opens Terminal 2 (9:05 AM)
         │
         ▼
   ┌─────────────┐
   │ No Session  │
   │ (PID 12346) │ ← Different PID!
   └──────┬──────┘
          │
          ▼
   ┌──────────────────┐
   │ Prompt Password  │ ← Must auth again!
   │ 🔐 Enter: Pass123│
   └────────┬─────────┘
            │
            ▼
     ┌─────────────────────┐
     │ ✓ Authentication OK  │
     │ Create session.12346 │ ← New session file
     │ Valid until 9:20 AM  │
     └─────────────────────┘
            │
            ▼
    ┌──────────────────┐
    │ Terminal 2 Ready │
    │ Can use freely   │
    └──────────────────┘

Terminal 1: Still valid until 9:15 AM
Terminal 2: Valid until 9:20 AM
```

## 📊 Timeline View

```
Time    Terminal 1 (PID 12345)         Terminal 2 (PID 12346)
────────────────────────────────────────────────────────────
9:00    ┌─ Open                        
        │  Auth ✓
        │  Session → 9:15               
        
9:05    │  [Active]                    ┌─ Open
        │                               │  Auth ✓
        │                               │  Session → 9:20
        
9:10    │  Type: ls                    │  [Active]
        │  Session extends → 9:25       │
        
9:15    │  [Active]                    │  Type: pwd
        │                               │  Session extends → 9:30
        
9:20    │  Type: cd /                  │  [Active]
        │  Session extends → 9:35       │
        
9:25    │  [Active]                    │  [Active]
        
9:30    │  [Active]                    │  [Session Expired!]
        
9:36    │  [Session Expired!]          │  Type: ls
        │                               │  Must Re-Auth! 🔐
        
9:40    │  Type: pwd                   │  [Active until 9:51]
        │  Must Re-Auth! 🔐             │
```

## 🔄 Session Lifecycle

### Single Terminal

```
┌────────────────────────────────────────────┐
│ TERMINAL LIFECYCLE                         │
└────────────────────────────────────────────┘

Open Terminal (PID 12345)
      │
      ▼
┌───────────────┐
│  Auth Check   │
└───────┬───────┘
        │
        ├─ Session exists? ──NO──┐
        │                         │
        └─ Session exists? ──YES─┤
                                  │
                                  ▼
                         ┌──────────────┐
                         │ Is it valid? │
                         └──────┬───────┘
                                │
                    ┌───────────┼───────────┐
                    │                       │
                   YES                     NO
                    │                       │
                    ▼                       ▼
            ┌──────────────┐      ┌─────────────────┐
            │ Show Greeting│      │ Prompt Password │
            │ Allow Access │      │ Create Session  │
            └──────────────┘      └─────────────────┘
                    │                       │
                    └───────────┬───────────┘
                                │
                                ▼
                    ┌──────────────────────┐
                    │ Terminal Ready       │
                    │ Session: 15 min      │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │ User Types Command   │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │ Session Extends      │
                    │ New expiry: +15 min  │
                    └──────────┬───────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │ Wait for Activity    │
                    └──────────┬───────────┘
                               │
                    ┌──────────┴───────────┐
                    │                      │
              15min pass?              Activity?
                    │                      │
                   YES                    YES
                    │                      │
                    ▼                      │
            ┌──────────────┐              │
            │ Session      │              │
            │ Expired      │              │
            └──────┬───────┘              │
                   │                      │
                   └──────────────────────┘
```

## 🎯 Key Points Visualization

### ❌ INCORRECT Mental Model

```
┌─────────────────────────────────────────┐
│ WRONG: Shared Session                   │
└─────────────────────────────────────────┘

Terminal 1 ──┐
             ├──► Global Session (15 min)
Terminal 2 ──┤
             │
Terminal 3 ──┘

❌ This is NOT how it works!
```

### ✅ CORRECT Mental Model

```
┌─────────────────────────────────────────┐
│ RIGHT: Independent Sessions             │
└─────────────────────────────────────────┘

Terminal 1 ──► Session 1 (15 min, PID 12345)

Terminal 2 ──► Session 2 (15 min, PID 12346)

Terminal 3 ──► Session 3 (15 min, PID 12347)

✓ Each terminal has its own session!
```

## 📁 File System Representation

```
~/.terminal_lock/
│
├── pass.hash                 # Shared (same password)
├── salt                      # Shared (same salt)
├── display_name              # Shared (same name)
├── preferences.json          # Shared (same preferences)
│
├── session.12345.json        # Terminal 1 only
│   {
│     "expiry": "2025-11-16T09:15:00Z",
│     "terminal_id": "12345"
│   }
│
├── session.12346.json        # Terminal 2 only
│   {
│     "expiry": "2025-11-16T09:20:00Z",
│     "terminal_id": "12346"
│   }
│
└── session.12347.json        # Terminal 3 only
    {
      "expiry": "2025-11-16T09:25:00Z",
      "terminal_id": "12347"
    }

Each terminal gets its own session file!
```

## 🔢 Mathematical Representation

```
Let:
  T = Terminal (identified by PID)
  S = Session
  A = Authentication

For each terminal T_i:
  - T_i has unique PID_i
  - T_i requires A_i (authentication)
  - A_i creates S_i (session for T_i only)
  - S_i expires after 15 minutes
  - S_i is independent of S_j (where j ≠ i)

Therefore:
  Opening N terminals = N authentications
  N terminals = N independent sessions
  No session sharing between terminals
```

## 🎪 Real-World Analogy

```
Terminal Lock Sessions = Movie Theater Tickets

Wrong Analogy (Shared Session):
├── Buy 1 ticket at 9:00 AM
└── Everyone in your family can enter theater all day
    ❌ This is NOT how it works

Correct Analogy (Independent Sessions):
├── Person 1 buys ticket → Valid for 15 minutes but if you get out of the theater before the 15 minutes are up but after 5 minutes you try to get back in.
├── Person 2 buys ticket → Their own 15-minute ticket
├── Person 3 buys ticket → Their own 15-minute ticket
└── Each person's ticket expires independently
    ✓ This is EXACTLY how it works!
```

## 🚦 Authentication Flow Chart

```
Start
  │
  ▼
Open New Terminal
  │
  ▼
Check: session.<MY_PID>.json exists?
  │
  ├─ NO ────────────────┐
  │                      │
  └─ YES                 │
      │                  │
      ▼                  │
  Is expiry > now?       │
      │                  │
      ├─ NO ─────────────┤
      │                  │
      └─ YES             │
          │              │
          ▼              │
     Show Greeting       │
          │              │
          └──────────────┤
                         │
                         ▼
                  Prompt Password
                         │
                         ▼
                  Verify Password
                         │
                    ┌────┴────┐
                    │         │
                 Correct   Wrong
                    │         │
                    ▼         ▼
            Create Session  Retry/Lockout
                    │
                    ▼
               Show Greeting
                    │
                    ▼
              Terminal Ready
```

## 📈 Session Count Over Time

```
Number of Active Sessions vs Number of Open Terminals

Terminals │
    5  │     ┌───────────────────
       │     │
    4  │   ┌─┘
       │   │
    3  │ ┌─┘
       │ │
    2  │─┘
       │
    1  ├─────┐
       │     │
    0  └─────┴───────────────────
       0   2   4   6   8  10  12  Time (minutes)

Each terminal opened = 1 new authentication
Each terminal opened = 1 new session
Sessions are independent and don't affect each other
```

## Summary Diagram

```
┌─────────────────────────────────────────────────────┐
│                 TERMINAL LOCK                       │
│             SESSION ARCHITECTURE                    │
└─────────────────────────────────────────────────────┘

   RULE 1: Each terminal = Unique PID
   RULE 2: Each PID = Separate session file
   RULE 3: Each session = 15 minutes independent
   RULE 4: New terminal = Always authenticate
   RULE 5: Sessions NEVER shared

┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Terminal 1   │  │ Terminal 2   │  │ Terminal 3   │
│ PID: 12345   │  │ PID: 12346   │  │ PID: 12347   │
└──────┬───────┘  └──────┬───────┘  └──────┬───────┘
       │                 │                 │
       ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Session 1    │  │ Session 2    │  │ Session 3    │
│ 15 min timer │  │ 15 min timer │  │ 15 min timer │
│ Independent  │  │ Independent  │  │ Independent  │
└──────────────┘  └──────────────┘  └──────────────┘

NO SHARING ← → ← → ← Independent Sessions
```