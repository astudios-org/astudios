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

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check if cargo-release is installed
    if ! command_exists cargo-release; then
        print_error "cargo-release is not installed. Installing..."
        cargo install cargo-release
    fi
    
    # Check if cargo-audit is installed
    if ! command_exists cargo-audit; then
        print_warning "cargo-audit is not installed. Installing..."
        cargo install cargo-audit
    fi
    
    # Check if git is clean
    if ! git diff-index --quiet HEAD --; then
        print_error "Working directory is not clean. Please commit or stash your changes."
        exit 1
    fi
    
    # Check if we're on the main branch
    current_branch=$(git branch --show-current)
    if [ "$current_branch" != "main" ]; then
        print_warning "You're not on the main branch (current: $current_branch)"
        read -p "Do you want to continue? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    print_success "Prerequisites check passed"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [RELEASE_TYPE]"
    echo ""
    echo "RELEASE_TYPE can be one of:"
    echo "  patch    - Patch release (0.1.0 -> 0.1.1)"
    echo "  minor    - Minor release (0.1.0 -> 0.2.0)"
    echo "  major    - Major release (0.1.0 -> 1.0.0)"
    echo "  alpha    - Alpha pre-release (0.1.0 -> 0.1.1-alpha.1)"
    echo "  beta     - Beta pre-release (0.1.0 -> 0.1.1-beta.1)"
    echo "  rc       - Release candidate (0.1.0 -> 0.1.1-rc.1)"
    echo ""
    echo "If no RELEASE_TYPE is provided, you will be prompted to choose."
}

# Function to prompt for release type
prompt_release_type() {
    echo "Select release type:"
    echo "1) patch (0.1.0 -> 0.1.1)"
    echo "2) minor (0.1.0 -> 0.2.0)"
    echo "3) major (0.1.0 -> 1.0.0)"
    echo "4) alpha (0.1.0 -> 0.1.1-alpha.1)"
    echo "5) beta (0.1.0 -> 0.1.1-beta.1)"
    echo "6) rc (0.1.0 -> 0.1.1-rc.1)"
    
    read -p "Enter your choice (1-6): " choice
    
    case $choice in
        1) echo "patch" ;;
        2) echo "minor" ;;
        3) echo "major" ;;
        4) echo "alpha" ;;
        5) echo "beta" ;;
        6) echo "rc" ;;
        *) 
            print_error "Invalid choice"
            exit 1
            ;;
    esac
}

# Function to perform dry run
dry_run() {
    local release_type=$1
    print_info "Performing dry run for $release_type release..."
    
    case $release_type in
        "patch"|"minor"|"major")
            cargo release $release_type --dry-run
            ;;
        "alpha"|"beta"|"rc")
            cargo release --profile $release_type --dry-run
            ;;
        *)
            print_error "Unknown release type: $release_type"
            exit 1
            ;;
    esac
}

# Function to perform actual release
perform_release() {
    local release_type=$1
    print_info "Performing $release_type release..."
    
    case $release_type in
        "patch"|"minor"|"major")
            cargo release $release_type --execute
            ;;
        "alpha"|"beta"|"rc")
            cargo release --profile $release_type --execute
            ;;
        *)
            print_error "Unknown release type: $release_type"
            exit 1
            ;;
    esac
}

# Main function
main() {
    # Check for help first, before any other operations
    if [ $# -eq 1 ] && [[ "$1" == "-h" || "$1" == "--help" ]]; then
        show_usage
        exit 0
    fi
    
    print_info "Starting release process for astudios..."
    
    # Check prerequisites
    check_prerequisites
    
    # Determine release type
    local release_type=""
    if [ $# -eq 0 ]; then
        release_type=$(prompt_release_type)
    elif [ $# -eq 1 ]; then
        case $1 in
            "patch"|"minor"|"major"|"alpha"|"beta"|"rc")
                release_type=$1
                ;;
            *)
                print_error "Invalid release type: $1"
                show_usage
                exit 1
                ;;
        esac
    else
        print_error "Too many arguments"
        show_usage
        exit 1
    fi
    
    print_info "Selected release type: $release_type"
    
    # Perform dry run first
    dry_run $release_type
    
    # Ask for confirmation
    echo ""
    print_warning "This will create a $release_type release. Are you sure?"
    read -p "Continue with release? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Release cancelled"
        exit 0
    fi
    
    # Perform actual release
    perform_release $release_type
    
    print_success "Release completed successfully!"
    print_info "Don't forget to:"
    print_info "  1. Check the GitHub release page"
    print_info "  2. Verify the crates.io publication"
    print_info "  3. Update any documentation if needed"
}

# Run main function with all arguments
main "$@"