#!/bin/bash

# Release script for astudios
# This script automates the release process using cargo-release

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Unified print function
print_msg() {
    local type=$1
    local msg=$2
    case $type in
        "info") echo -e "${BLUE}[INFO]${NC} $msg" ;;
        "success") echo -e "${GREEN}[SUCCESS]${NC} $msg" ;;
        "warning") echo -e "${YELLOW}[WARNING]${NC} $msg" ;;
        "error") echo -e "${RED}[ERROR]${NC} $msg" ;;
    esac
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_msg "info" "Checking prerequisites..."
    
    # Check if cargo-release is installed
    if ! command_exists cargo-release; then
        print_msg "error" "cargo-release is not installed. Installing..."
        cargo install cargo-release
    fi
    
    # Check if git is clean
    if ! git diff-index --quiet HEAD --; then
        print_msg "error" "Working directory is not clean. Please commit or stash your changes."
        exit 1
    fi
    
    # Check if we're on the main branch
    current_branch=$(git branch --show-current)
    if [ "$current_branch" != "main" ]; then
        print_msg "warning" "You're not on the main branch (current: $current_branch)"
        read -p "Do you want to continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    print_msg "success" "Prerequisites check passed"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [RELEASE_TYPE]"
    echo ""
    echo "RELEASE_TYPE: patch|minor|major|alpha|beta|rc"
    echo "If no type is provided, defaults to 'patch'"
}

# Function to validate release type
validate_release_type() {
    local type=$1
    case $type in
        "patch"|"minor"|"major"|"alpha"|"beta"|"rc")
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Function to confirm release with user
confirm_release() {
    local release_type=$1
    print_msg "warning" "This will create a $release_type release and publish to crates.io!"
    read -p "Continue? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        return 0
    else
        print_msg "info" "Release cancelled by user"
        exit 0
    fi
}

# Function to perform dry run
dry_run() {
    local release_type=$1
    print_msg "info" "Performing dry run for $release_type release..."
    
    local cmd="cargo release"
    if [[ "$release_type" =~ ^(alpha|beta|rc)$ ]]; then
        cmd="$cmd --profile $release_type"
    else
        cmd="$cmd $release_type"
    fi
    
    if $cmd 2>&1; then
        print_msg "success" "Dry run completed successfully"
        return 0
    else
        print_msg "error" "Dry run failed. Please fix the issues and try again."
        return 1
    fi
}

# Function to perform actual release
perform_release() {
    local release_type=$1
    print_msg "info" "Performing $release_type release..."
    
    local cmd="cargo release"
    if [[ "$release_type" =~ ^(alpha|beta|rc)$ ]]; then
        cmd="$cmd --profile $release_type --execute"
    else
        cmd="$cmd $release_type --execute"
    fi
    
    if $cmd; then
        return 0
    else
        print_msg "error" "Release command failed"
        return 1
    fi
}

# Main function
main() {
    # Check for help
    if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
        show_usage
        exit 0
    fi
    
    print_msg "info" "Starting release process for astudios..."
    
    # Check prerequisites
    check_prerequisites
    
    # Determine release type (default to patch if not provided)
    local release_type="${1:-patch}"
    
    # Validate release type
    if ! validate_release_type "$release_type"; then
        print_msg "error" "Invalid release type: $release_type"
        show_usage
        exit 1
    fi
    
    print_msg "info" "Selected release type: $release_type"
    
    # Perform dry run
    if ! dry_run "$release_type"; then
        print_msg "error" "Please fix the issues and run the script again."
        exit 1
    fi
    
    # Show release summary and confirm
    echo ""
    print_msg "info" "=== Release Summary ==="
    echo "  Release type: $release_type"
    echo "  Current version: $(cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[0].version' 2>/dev/null || echo 'unknown')"
    echo "  Branch: $(git branch --show-current 2>/dev/null || echo 'unknown')"
    echo ""
    
    # Get user confirmation
    confirm_release "$release_type"
    
    # Perform actual release
    print_msg "info" "🚀 Starting actual release process..."
    
    if perform_release "$release_type"; then
        print_msg "success" "🎉 Release completed successfully!"
        print_msg "info" "Check: https://github.com/astudios-org/astudios/releases"
        print_msg "info" "Verify: https://crates.io/crates/astudios"
    else
        print_msg "error" "❌ Release failed!"
        exit 1
    fi
}

# Run main function with all arguments
main "$@"