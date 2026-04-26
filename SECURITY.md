# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately:

1. **DO NOT** create a public GitHub issue
2. Email: [security contact needed]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

## Response Timeline

- **24 hours**: Initial response acknowledging receipt
- **72 hours**: Initial assessment and severity classification
- **7 days**: Detailed response with fix timeline

## Security Best Practices

When contributing:
- Always validate inputs
- Use proper authentication checks
- Follow principle of least privilege
- Test edge cases and error conditions
- Review code for potential vulnerabilities

## Encryption Key Derivation

Sensitive pet and owner fields use a deterministic key derived at runtime from:
- a fixed domain separator (`petchain:encryption-key:v1`)
- the current contract address
- admin context (legacy single admin or first multisig admin when configured)

This replaces the previous static all-zero key. As a result:
- encrypted storage no longer mirrors plaintext by default
- key material is contract-scoped and not hardcoded
- rotating admin configuration can change derived key context, so migrations/rollouts should account for backward compatibility expectations