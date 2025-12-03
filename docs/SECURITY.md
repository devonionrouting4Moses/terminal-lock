# Security Documentation

## Security Model

Terminal Lock implements a defense-in-depth security model with multiple layers of protection.

## Password Security

### Hashing Algorithm
- **Algorithm**: Argon2id
- **Why Argon2**: Memory-hard, resistant to GPU/ASIC attacks, OWASP recommended
- **Configuration**:
  - Memory cost: 64 MB (default)
  - Time cost: 3 iterations
  - Parallelism: 4 threads

### Salt Generation
- **Length**: 16 characters minimum
- **Source**: Cryptographically secure random (`OsRng`)
- **Storage**: Separate file from hash
- **Permissions**: 600 (owner read/write only)

### Password Requirements
```
Length: Exactly 6 characters
Must contain:
  - At least 1 uppercase letter (A-Z)
  - At least 1 lowercase letter (a-z)
  - At least 1 digit (0-9)
```

**Rationale**: Balance between security and usability for terminal authentication.

### Password Storage
```
~/.terminal_lock/
├── pass.hash    # Argon2 hash (600 permissions)
└── salt         # Random salt (600 permissions)
```

**Never** stored in plaintext anywhere.

## Session Management

### Session Lifecycle
1. **Creation**: After successful authentication
2. **Validation**: On every terminal use
3. **Extension**: Valid session extends timeout
4. **Expiration**: After 15 minutes of inactivity

### Session Files
- **Location**: `~/.terminal_lock/session.<pid>.json`
- **Content**: Expiry timestamp, terminal ID
- **Permissions**: 600 (owner only)
- **Isolation**: Per-terminal (PID-based)

### Session Security Features
- Process-ID isolation (one session per terminal)
- Automatic timeout
- Extends on activity
- Removed on logout
- Cleared on system reboot

## Lockout Protection

### Per-Terminal Lockout
- **Trigger**: 3 failed attempts in one terminal
- **Duration**: 60 seconds
- **Effect**: Terminal completely blocked
- **Bypass**: None (must wait)

### Global System Lock
- **Trigger**: 9 total failed attempts across all terminals
- **Effect**: Laptop locks (screen saver/suspend)
- **Reset**: On laptop unlock

### Attempt Counter
- **Location**: `~/.terminal_lock/total_attempts.json`
- **Persistence**: Until laptop unlock
- **Detection**: Monitors system boot time
- **Reset**: Automatic on laptop unlock

## File System Security

### Directory Permissions
```bash
~/.terminal_lock/           # 700 (drwx------)
```

### File Permissions
```bash
pass.hash                   # 600 (-rw-------)
salt                        # 600 (-rw-------)
display_name                # 600 (-rw-------)
preferences.json            # 600 (-rw-------)
session.*.json              # 600 (-rw-------)
lockout.*.json              # 600 (-rw-------)
total_attempts.json         # 600 (-rw-------)
```

**Why 600**: Only the owner can read/write. Group and others have no access.

## Threat Model

### Threats Addressed

#### ✅ Brute Force Attacks
- **Mitigation**: Progressive lockout, Argon2 slow hashing
- **Effect**: Makes brute force impractical

#### ✅ Dictionary Attacks
- **Mitigation**: Password complexity requirements, Argon2
- **Effect**: Common passwords fail validation

#### ✅ Rainbow Table Attacks
- **Mitigation**: Unique random salt per password
- **Effect**: Pre-computed tables useless

#### ✅ Timing Attacks
- **Mitigation**: Argon2 constant-time verification
- **Effect**: Cannot leak information via timing

#### ✅ Unauthorized Terminal Access
- **Mitigation**: Per-terminal sessions, automatic timeout
- **Effect**: Shared terminals protected

#### ✅ Privilege Escalation
- **Mitigation**: 600 permissions, no setuid/setgid
- **Effect**: Only owner can access

### Threats NOT Addressed

#### ❌ Physical Access with Root
- **Risk**: Root user can read all files
- **Mitigation**: None (by design, root can do anything)

#### ❌ Memory Dumps
- **Risk**: Password might be in memory briefly
- **Mitigation**: Zeroize on drop (not implemented yet)

#### ❌ Keyloggers
- **Risk**: Malware can capture keystrokes
- **Mitigation**: None (OS-level concern)

#### ❌ Side-Channel Attacks
- **Risk**: CPU cache timing, power analysis
- **Mitigation**: Relies on Argon2's protections

## Best Practices

### For Users

1. **Strong Password**
   - Use random combination
   - Don't reuse from other services
   - Change periodically

2. **Secure Environment**
   - Lock laptop when away
   - Don't share credentials
   - Keep system updated

3. **Monitor Attempts**
   ```bash
   terminal-lock status
   ```

### For Developers

1. **Never Log Passwords**
   ```rust
   // ❌ NEVER
   println!("Password: {}", password);
   
   // ✅ OK
   println!("Password validated");
   ```

2. **Use Secure Random**
   ```rust
   use rand::rngs::OsRng;
   // Use OsRng for cryptographic randomness
   ```

3. **Check File Permissions**
   ```rust
   #[cfg(unix)]
   {
       use std::os::unix::fs::PermissionsExt;
       std::fs::set_permissions(path, Permissions::from_mode(0o600))?;
   }
   ```

4. **Handle Errors Securely**
   ```rust
   // ❌ NEVER leak information
   Err(Error::AuthenticationFailed(format!("Hash: {}", hash)))
   
   // ✅ Generic message
   Err(Error::AuthenticationFailed("Invalid password".into()))
   ```

## Security Audit Checklist

### Code Review
- [ ] No hardcoded credentials
- [ ] Passwords never logged
- [ ] Proper error messages (no info leaks)
- [ ] Secure random for salts
- [ ] File permissions set correctly
- [ ] No SQL injection (N/A - no SQL)
- [ ] No command injection (validated inputs)

### Testing
- [ ] Password validation tests
- [ ] Hash/verify tests
- [ ] Lockout mechanism tests
- [ ] File permission tests
- [ ] Session timeout tests
- [ ] Attempt counter tests

### Deployment
- [ ] Binary stripped (no debug symbols)
- [ ] Release build (optimizations)
- [ ] Dependencies audited (`cargo audit`)
- [ ] No known vulnerabilities

## Cryptographic Dependencies

### Argon2
- **Version**: 0.5.x
- **Variant**: Argon2id (hybrid)
- **Audited**: Yes (by cryptography community)
- **CVEs**: None known

### Rand
- **Version**: 0.8.x
- **RNG**: OsRng (OS cryptographic RNG)
- **Audited**: Yes
- **CVEs**: None known

### SHA-2
- **Version**: 0.10.x
- **Algorithm**: SHA-512 (fallback only)
- **Audited**: Yes (NIST standard)
- **CVEs**: None known

## Compliance

### OWASP Password Storage Cheat Sheet
- ✅ Use modern hashing (Argon2)
- ✅ Use unique salts
- ✅ Store hash and salt separately
- ✅ Never store plaintext
- ✅ Implement account lockout

### NIST Digital Identity Guidelines
- ✅ Minimum password length (6 chars)
- ✅ Password complexity requirements
- ✅ Secure storage (Argon2)
- ✅ Rate limiting (lockout)

## Incident Response

### Suspected Compromise

1. **Immediate Actions**
   ```bash
   # Reset everything
   terminal-lock reset
   
   # Run setup again with new password
   terminal-lock setup
   ```

2. **Investigation**
   ```bash
   # Check attempts
   terminal-lock status
   
   # Review system logs
   journalctl -u sshd
   ```

3. **Prevention**
   - Change password
   - Review authorized_keys
   - Check for suspicious processes

### Reporting Vulnerabilities

If you discover a security vulnerability:

1. **DO NOT** open a public issue
2. Email: security@yourproject.com
3. Include:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Security Updates

### Monitoring
- Subscribe to security advisories
- Run `cargo audit` regularly
- Update dependencies monthly

### Update Process
1. Run `cargo update`
2. Run `cargo audit`
3. Run full test suite
4. Rebuild release binary
5. Reinstall

## Future Security Enhancements

### Planned
- [ ] Add key derivation function (KDF)
- [ ] Implement zeroize for memory
- [ ] Add TOTP 2FA support
- [ ] Hardware key support (YubiKey)
- [ ] Audit logging
- [ ] Rate limiting at system level

### Under Consideration
- [ ] Biometric authentication
- [ ] Hardware security module (HSM)
- [ ] Remote attestation
- [ ] Encrypted preference storage

## References

- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [Argon2 RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html)
- [NIST SP 800-63B](https://pages.nist.gov/800-63-3/sp800-63b.html)