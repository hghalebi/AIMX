# Security Policy

## Supported Versions

Security fixes are provided for the latest published minor release.

## Reporting a Vulnerability

Please report security issues privately through GitHub Security Advisories on
the project repository. If advisories are unavailable, contact the repository
owner privately before opening a public issue.

Do not include working exploit details in public issues.

## Security Boundaries

AIMX keeps unsafe Rust isolated to the private FFI layer.
Public APIs should validate values before they cross into Swift or C strings.

Tool handlers are user-provided code. This crate forwards tool-handler failures
back to the model, but it does not sandbox tool execution or catch panics inside
handlers.
