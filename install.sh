#!/bin/bash
# Terminal Lock - Universal Installer
# Combines Rust compilation with shell integration
# Supports: Bash, Zsh, Fish, and more

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

# Configuration
BINARY_NAME="terminal-lock"
INSTALL_DIR="$HOME/.local/bin"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
CONFIG_DIR="$HOME/.terminal_lock"
BACKUP_DIR=""
DAEMON_SERVICE_DIR="$HOME/.config/systemd/user"

# Print functions
print_header() {
    echo -e "\n${BLUE}${BOLD}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}${BOLD}  $1${NC}"
    echo -e "${BLUE}${BOLD}═══════════════════════════════════════════════════════════${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Check system compatibility
check_system() {
    print_header "Checking System Compatibility"
    
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        print_success "Linux detected"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        print_success "macOS detected"
    else
        print_warning "System may not be fully supported: $OSTYPE"
        read -p "Continue anyway? (y/n): " continue_choice
        if [[ "$continue_choice" != "y" ]]; then
            exit 1
        fi
    fi
    echo
}

# Install system dependencies
install_dependencies() {
    print_header "Checking Dependencies"
    
    local missing_deps=()
    
    # Check for curl
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    else
        print_success "curl installed"
    fi
    
    # Check for openssl
    if ! command -v openssl &> /dev/null; then
        print_warning "openssl not found (recommended for stronger encryption)"
        missing_deps+=("openssl")
    else
        print_success "openssl installed"
    fi
    
    # Install missing dependencies
    if [ ${#missing_deps[@]} -gt 0 ]; then
        echo
        print_info "Missing dependencies: ${missing_deps[*]}"
        
        if command -v apt-get &> /dev/null; then
            print_info "Detected: Debian/Ubuntu (apt)"
            read -p "Install with apt? (y/n): " install_choice
            if [[ "$install_choice" == "y" ]]; then
                sudo apt-get update
                for dep in "${missing_deps[@]}"; do
                    sudo apt-get install -y "$dep"
                done
            fi
        elif command -v yum &> /dev/null; then
            print_info "Detected: RedHat/CentOS (yum)"
            read -p "Install with yum? (y/n): " install_choice
            if [[ "$install_choice" == "y" ]]; then
                for dep in "${missing_deps[@]}"; do
                    sudo yum install -y "$dep"
                done
            fi
        elif command -v pacman &> /dev/null; then
            print_info "Detected: Arch Linux (pacman)"
            read -p "Install with pacman? (y/n): " install_choice
            if [[ "$install_choice" == "y" ]]; then
                for dep in "${missing_deps[@]}"; do
                    sudo pacman -S --noconfirm "$dep"
                done
            fi
        elif command -v brew &> /dev/null; then
            print_info "Detected: macOS (brew)"
            read -p "Install with brew? (y/n): " install_choice
            if [[ "$install_choice" == "y" ]]; then
                for dep in "${missing_deps[@]}"; do
                    brew install "$dep"
                done
            fi
        else
            print_warning "Could not detect package manager. Please install manually: ${missing_deps[*]}"
        fi
    fi
    
    echo
}

# Check and install Rust
check_rust() {
    print_header "Checking Rust Installation"
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found"
        print_info "Rust is required to build Terminal Lock"
        echo
        read -p "Install Rust now? (y/n): " install_rust
        if [[ "$install_rust" == "y" ]]; then
            print_info "Installing Rust via rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            source "$HOME/.cargo/env"
            print_success "Rust installed successfully"
        else
            print_error "Cannot proceed without Rust"
            exit 1
        fi
    else
        print_success "Rust/Cargo found: $(cargo --version)"
    fi
    echo
}

# Build the Rust binary
build_binary() {
    print_header "Building Terminal Lock"
    
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Are you in the project directory?"
        exit 1
    fi
    
    print_info "Building optimized release binary (this may take a few minutes)..."
    
    if cargo build --release; then
        if [ -f "target/release/$BINARY_NAME" ]; then
            print_success "Binary built successfully"
        else
            print_error "Build succeeded but binary not found at target/release/$BINARY_NAME"
            exit 1
        fi
    else
        print_error "Build failed"
        exit 1
    fi
    
    echo
}

# Setup installation directory
setup_install_dir() {
    print_header "Setting Up Installation Directory"
    
    if [ ! -d "$INSTALL_DIR" ]; then
        mkdir -p "$INSTALL_DIR"
        print_success "Created $INSTALL_DIR"
    else
        print_success "Directory exists: $INSTALL_DIR"
    fi
    
    echo
}

# Install the binary
install_binary() {
    print_header "Installing Binary"
    
    if [ -f "$BINARY_PATH" ]; then
        print_warning "Binary already exists at $BINARY_PATH"
        read -p "Overwrite? (y/n): " overwrite
        if [[ "$overwrite" != "y" ]]; then
            print_info "Skipping binary installation"
            return
        fi
        
        # Kill any running instances of terminal-lock
        print_info "Stopping running terminal-lock instances..."
        pkill -f "terminal-lock" 2>/dev/null || true
        sleep 1
    fi
    
    cp "target/release/$BINARY_NAME" "$BINARY_PATH"
    chmod +x "$BINARY_PATH"
    
    # Set proper ownership and permissions
    chown "$USER:$USER" "$BINARY_PATH" 2>/dev/null || true
    chmod 700 "$BINARY_PATH"
    
    print_success "Installed to $BINARY_PATH"
    echo
}

# Add PATH to shell configuration
add_to_path() {
    local shell_file="$1"
    local path_line="export PATH=\"\$PATH:$INSTALL_DIR\""
    
    if [ -f "$shell_file" ]; then
        if ! grep -q "$INSTALL_DIR" "$shell_file"; then
            echo >> "$shell_file"
            echo "# Terminal Lock - Add to PATH" >> "$shell_file"
            echo "$path_line" >> "$shell_file"
            print_success "Added to PATH in $(basename $shell_file)"
        else
            print_info "PATH already configured in $(basename $shell_file)"
        fi
    fi
}

# Configure Bash
configure_bash() {
    local bashrc="$HOME/.bashrc"
    
    if [ -f "$bashrc" ]; then
        # Remove any old configurations
        sed -i.backup '/# Terminal.*Authentication/,/^fi$/d' "$bashrc" 2>/dev/null || true
        
        # Add new configuration at the end
        cat >> "$bashrc" << 'EOF'

# ============================================
# Terminal Lock - Authentication System
# Each terminal requires independent authentication
# ============================================
if [[ $- == *i* ]]; then
    if command -v terminal-lock &> /dev/null; then
        terminal-lock auth 2>&1
        if [ $? -ne 0 ]; then
            echo "❌ Authentication failed"
            exit 1
        fi
    fi
fi

# ============================================
# Terminal Lock - Session Validation & Extension
# Checks session validity BEFORE showing prompt
# Blocks terminal if session is invalid
# ============================================
PROMPT_COMMAND="terminal-lock check 2>/dev/null || exit 1; terminal-lock extend 2>/dev/null; ${PROMPT_COMMAND:+$PROMPT_COMMAND$'\n'}"
EOF
        print_success "Configured Bash (~/.bashrc)"
        add_to_path "$bashrc"
    else
        print_warning "~/.bashrc not found"
    fi
}

# Configure Zsh
configure_zsh() {
    local zshrc="$HOME/.zshrc"
    
    if [ -f "$zshrc" ]; then
        # Remove any old configurations
        sed -i.backup '/# Terminal.*Authentication/,/^fi$/d' "$zshrc" 2>/dev/null || true
        
        # Add new configuration at the end
        cat >> "$zshrc" << 'EOF'

# ============================================
# Terminal Lock - Authentication System
# Each terminal requires independent authentication
# ============================================
if [[ -o interactive && -t 0 ]]; then
    if command -v terminal-lock &> /dev/null; then
        # CRITICAL: Export shell PID so terminal-lock knows which session to use
        export TERMINAL_LOCK_SESSION_PID=$$
        
        # Authenticate and create session
        terminal-lock auth 2>&1
        if [ $? -ne 0 ]; then
            echo "❌ Authentication failed"
            exit 1
        fi
        
        # CRITICAL: Allow greeting to complete before hooks start running
        sleep 0.3
        
        # Verify session was actually created before proceeding
        if ! terminal-lock check 2>/dev/null; then
            echo "❌ Session creation verification failed"
            exit 1
        fi
    else
        echo "❌ terminal-lock command not found. Install it first."
        exit 1
    fi
fi

# ============================================
# Terminal Lock - Session Validation & Extension
# Uses preexec() as activation trigger
# Validation DISABLED until first command executes
# ============================================

# Start with validation DISABLED - only activate on first command
_TERMINAL_LOCK_READY=0

# HOOK 1: Validate session BEFORE showing prompt
# This prevents typing in an expired/invalid session
zle-line-init() {
    # CRITICAL GUARD: Wait for terminal to be fully ready
    # Skip validation until explicitly marked as ready
    if [ $_TERMINAL_LOCK_READY -eq 0 ]; then
        zle reset-prompt
        return 0
    fi
    
    # NOW safe to validate session before showing prompt
    # IMPORTANT: Must export TERMINAL_LOCK_SESSION_PID for check command
    export TERMINAL_LOCK_SESSION_PID=$$
    
    if ! terminal-lock check 2>/dev/null; then
        echo ""
        echo "🔐 Session expired or invalid. Please authenticate again."
        exit 1
    fi
    
    zle reset-prompt
}
zle -N zle-line-init

# HOOK 2: Update activity and validate AFTER each command
precmd() {
    # CRITICAL GUARD: Skip ALL validation during initialization
    # Only activate after first REAL command has been executed
    if [ $_TERMINAL_LOCK_READY -eq 0 ]; then
        return 0
    fi
    
    # IMPORTANT: Must export TERMINAL_LOCK_SESSION_PID for extend/check commands
    export TERMINAL_LOCK_SESSION_PID=$$
    
    # Update activity timestamp (does NOT extend 15-minute expiry)
    terminal-lock extend 2>/dev/null
    
    # Validate session is still valid after command execution
    if ! terminal-lock check 2>/dev/null; then
        echo ""
        echo "🔐 Session expired or invalid. Please authenticate again."
        exit 1
    fi
}

# HOOK 3: Mark terminal as ready BEFORE first command execution
# This is the ACTIVATOR for all validation hooks
preexec() {
    # First command about to execute - NOW activate all validation
    _TERMINAL_LOCK_READY=1
    
    # IMPORTANT: Must export TERMINAL_LOCK_SESSION_PID for check command
    export TERMINAL_LOCK_SESSION_PID=$$
    
    # Validate session before allowing command to execute
    if ! terminal-lock check 2>/dev/null; then
        echo ""
        echo "🔐 Session expired or invalid. Please authenticate again."
        exit 1
    fi
}
EOF
        print_success "Configured Zsh (~/.zshrc)"
        add_to_path "$zshrc"
    else
        print_info "~/.zshrc not found (Zsh may not be installed)"
    fi
}

# Configure Fish
configure_fish() {
    local fish_config="$HOME/.config/fish/config.fish"
    
    if command -v fish &> /dev/null; then
        mkdir -p "$HOME/.config/fish"
        
        # Remove old configurations
        if [ -f "$fish_config" ]; then
            sed -i.backup '/# Terminal.*Authentication/,/^end$/d' "$fish_config" 2>/dev/null || true
        fi
        
        # Add new configuration
        cat >> "$fish_config" << 'EOF'

# ============================================
# Terminal Lock - Authentication System
# Each terminal requires independent authentication
# ============================================
if status is-interactive
    if command -v terminal-lock > /dev/null
        terminal-lock auth
        if test $status -ne 0
            echo "❌ Authentication failed"
            exit 1
        end
    end
end

# ============================================
# Terminal Lock - Session Validation & Extension
# Checks session validity BEFORE showing prompt
# Blocks terminal if session is invalid
# ============================================
function fish_postexec --description "Terminal Lock session validation and extension"
    # Check session validity after each command
    terminal-lock check 2>/dev/null
    if test $status -ne 0
        echo ""
        echo "🔐 Session expired. Please authenticate again."
        exit 1
    end
    
    # Update activity timestamp
    terminal-lock extend 2>/dev/null
end

# Add to PATH
set -gx PATH $PATH $HOME/.local/bin
EOF
        print_success "Configured Fish (~/.config/fish/config.fish)"
    else
        print_info "Fish shell not installed (skipped)"
    fi
}

# Configure all shells
configure_shells() {
    print_header "Configuring Shell Integration"
    
    configure_bash
    configure_zsh
    configure_fish
    
    echo
}

# Create backup of configurations
backup_configs() {
    print_header "Creating Backup"
    
    BACKUP_DIR="$HOME/.terminal_lock_backup_$(date +%Y%m%d_%H%M%S)"
    
    mkdir -p "$BACKUP_DIR"
    
    [ -f "$HOME/.bashrc" ] && cp "$HOME/.bashrc" "$BACKUP_DIR/"
    [ -f "$HOME/.zshrc" ] && cp "$HOME/.zshrc" "$BACKUP_DIR/"
    [ -f "$HOME/.config/fish/config.fish" ] && cp "$HOME/.config/fish/config.fish" "$BACKUP_DIR/"
    
    if [ -d "$CONFIG_DIR" ]; then
        cp -r "$CONFIG_DIR" "$BACKUP_DIR/"
    fi
    
    print_success "Backup created at: $BACKUP_DIR"
    echo
}

# Install daemon service
install_daemon_service() {
    print_header "Installing Daemon Service"
    
    if [ ! -f "terminal-lock-daemon.service" ]; then
        print_warning "Daemon service file not found (optional)"
        return 0
    fi
    
    mkdir -p "$DAEMON_SERVICE_DIR"
    
    # Copy service file
    cp terminal-lock-daemon.service "$DAEMON_SERVICE_DIR/"
    
    # Enable and start daemon
    systemctl --user daemon-reload
    systemctl --user enable terminal-lock-daemon.service
    systemctl --user start terminal-lock-daemon.service
    
    print_success "Daemon service installed and started"
    
    # Verify daemon is running
    if systemctl --user is-active --quiet terminal-lock-daemon.service; then
        print_success "✓ Daemon is running"
    else
        print_warning "⚠ Daemon failed to start (will use file-based fallback)"
    fi
    
    echo
}

# Run initial setup
run_setup() {
    print_header "Initial Configuration"
    
    print_info "Starting setup wizard..."
    echo
    
    if [ -f "$BINARY_PATH" ]; then
        "$BINARY_PATH" setup
    else
        print_error "Binary not found at $BINARY_PATH"
        exit 1
    fi
    
    echo
}

# Verify installation
verify_installation() {
    print_header "Verifying Installation"
    
    local issues=0
    
    # Check binary
    if [ -x "$BINARY_PATH" ]; then
        print_success "Binary is executable"
    else
        print_error "Binary is not executable"
        issues=$((issues + 1))
    fi
    
    # Check if binary runs
    if "$BINARY_PATH" --version &> /dev/null; then
        print_success "Binary runs correctly"
    else
        print_error "Binary fails to run"
        issues=$((issues + 1))
    fi
    
    # Check PATH
    if command -v terminal-lock &> /dev/null; then
        print_success "Binary is in PATH"
    else
        print_warning "Binary not in current PATH (will work after restart)"
    fi
    
    # Check configuration
    if [ -d "$CONFIG_DIR" ]; then
        print_success "Configuration directory exists"
    else
        print_warning "Configuration directory not yet created (will be created on first run)"
    fi
    
    echo
    
    if [ $issues -gt 0 ]; then
        print_error "$issues issue(s) found"
        return 1
    else
        print_success "All checks passed!"
        return 0
    fi
}

# Show completion message
show_completion() {
    print_header "Installation Complete!"
    
    echo -e "${GREEN}${BOLD}Terminal Lock has been installed successfully!${NC}\n"
    
    echo -e "${CYAN}Installed Components:${NC}"
    echo -e "  • Binary: ${BLUE}$BINARY_PATH${NC}"
    echo -e "  • Config: ${BLUE}$CONFIG_DIR${NC}"
    echo -e "  • Daemon: ${BLUE}Systemd user service${NC}"
    echo
    
    echo -e "${CYAN}Configured Shells:${NC}"
    [ -f "$HOME/.bashrc" ] && echo -e "  ${GREEN}✓${NC} Bash (~/.bashrc)"
    [ -f "$HOME/.zshrc" ] && echo -e "  ${GREEN}✓${NC} Zsh (~/.zshrc)"
    command -v fish &> /dev/null && echo -e "  ${GREEN}✓${NC} Fish (~/.config/fish/config.fish)"
    echo
    
    echo -e "${CYAN}Daemon Status:${NC}"
    if systemctl --user is-active --quiet terminal-lock-daemon.service 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Daemon is running"
    else
        echo -e "  ${YELLOW}⚠${NC} Daemon not running (will use file-based fallback)"
    fi
    echo
    
    echo -e "${CYAN}Key Features:${NC}"
    echo -e "  ${GREEN}✓${NC} Secure Argon2 password hashing"
    echo -e "  ${GREEN}✓${NC} Per-terminal 15-minute sessions"
    echo -e "  ${GREEN}✓${NC} Independent authentication per terminal"
    echo -e "  ${GREEN}✓${NC} Smart lockout protection (3 attempts → 60s, 9 → system lock)"
    echo -e "  ${GREEN}✓${NC} 1000+ color schemes"
    echo -e "  ${GREEN}✓${NC} Background animations"
    echo -e "  ${GREEN}✓${NC} 43+ motivational quote APIs"
    echo -e "  ${GREEN}✓${NC} Daemon-based session management"
    echo
    
    echo -e "${CYAN}Available Commands:${NC}"
    echo -e "  ${BLUE}terminal-lock setup${NC}        - Run setup wizard"
    echo -e "  ${BLUE}terminal-lock preferences${NC}  - Manage preferences"
    echo -e "  ${BLUE}terminal-lock status${NC}       - Check authentication status"
    echo -e "  ${BLUE}terminal-lock reset${NC}        - Reset attempts counter"
    echo -e "  ${BLUE}terminal-lock lock${NC}         - Lock terminal manually"
    echo -e "  ${BLUE}terminal-lock daemon${NC}       - Start daemon manually"
    echo
    
    echo -e "${YELLOW}${BOLD}IMPORTANT - Session Behavior:${NC}"
    echo -e "  ${CYAN}Each terminal window authenticates independently!${NC}"
    echo -e "  • Opening Terminal 1 → Authenticate → 15-min session"
    echo -e "  • Opening Terminal 2 → ${BOLD}Must authenticate again${NC} (separate session)"
    echo -e "  • Each terminal has its own countdown"
    echo -e "  • This is a ${GREEN}security feature${NC}, not a bug!"
    echo
    
    echo -e "${YELLOW}Next Steps:${NC}"
    echo -e "  1. ${BOLD}Close this terminal${NC} completely"
    echo -e "  2. ${BOLD}Open a new terminal${NC}"
    echo -e "  3. You'll be prompted: ${CYAN}🔐 Enter terminal password:${NC}"
    echo -e "  4. After authentication, you'll see greeting and quote"
    echo
    
    echo -e "${CYAN}To test immediately (in current terminal):${NC}"
    echo -e "  ${BLUE}source ~/.zshrc${NC}  # or ~/.bashrc for Bash"
    echo
    
    print_success "Installation complete! Enjoy your secure terminal experience! 🚀"
    echo
}

# Cleanup backup if installation successful
cleanup_backup() {
    if [ -n "$BACKUP_DIR" ] && [ -d "$BACKUP_DIR" ]; then
        print_info "Installation successful - keeping backup at: $BACKUP_DIR"
        print_info "Backup will be kept for 7 days for safety"
        print_info "To restore: cp -r $BACKUP_DIR/* ~/"
    fi
}

# Main installation flow
main() {
    clear
    print_header "Terminal Lock - Universal Installer"
    
    echo -e "${CYAN}This installer will:${NC}"
    echo -e "  • Check system compatibility"
    echo -e "  • Install Rust (if needed)"
    echo -e "  • Build optimized binary from source"
    echo -e "  • Install to ~/.local/bin"
    echo -e "  • Configure ALL shells (Bash, Zsh, Fish)"
    echo -e "  • Set up secure authentication"
    echo -e "  • Create configuration backup"
    echo
    
    echo -e "${YELLOW}${BOLD}NOTE:${NC} ${CYAN}Each terminal window will require independent authentication.${NC}"
    echo -e "${CYAN}This is a security feature to prevent session hijacking.${NC}"
    echo
    
    read -p "Press Enter to continue or Ctrl+C to cancel..."
    
    check_system
    install_dependencies
    check_rust
    backup_configs
    build_binary
    setup_install_dir
    install_binary
    configure_shells
    install_daemon_service
    
    if verify_installation; then
        run_setup
        show_completion
        cleanup_backup
    else
        print_error "Installation completed with issues. Please review the output above."
        print_warning "Backup preserved at: $BACKUP_DIR"
        exit 1
    fi
}

# Run installer
main "$@"