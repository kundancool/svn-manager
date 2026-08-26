# Security Policy

## Supported versions

The latest release receives security fixes.

## Reporting a vulnerability

Please report security issues privately via GitHub's
[private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
on this repository. Do not open public issues for security problems.

You can expect an initial response within a week.

## Security model notes

- Passwords are stored only in the operating system keychain (macOS Keychain,
  Windows Credential Manager, Linux secret-service) — never in configuration
  files.
- Credentials are passed to `svn` via stdin (`--password-from-stdin`), never as
  command-line arguments, so they don't appear in the process list.
- The app shells out to the system `svn` binary; it does not bundle or patch
  Subversion itself.
