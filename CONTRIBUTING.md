# Contributing to StellarInsure

Thank you for your interest in contributing to StellarInsure! We welcome contributions from the community.

## Code of Conduct

Please be respectful and constructive in all interactions. We're building an inclusive community.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, versions, etc.)

### Suggesting Features

We welcome feature suggestions! Please:
- Check existing issues first
- Provide clear use cases
- Explain why it benefits users

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Write or update tests
5. Ensure code passes linting
6. Commit with clear messages
7. Push to your fork
8. Open a Pull Request

#### CI Security & Dependency Gates
Pull requests are automatically scanned for security vulnerabilities and lockfile drift:
- **Lockfile Drift Check**: Ensure `package-lock.json` and `Cargo.lock` match their respective `package.json` and `Cargo.toml` configurations. If the drift check fails, regenerate lockfiles locally (e.g., run `npm install` in `frontend/` or `cargo check` in `smartcontract/`) and commit them.
- **Trivy Vulnerability Scan**: CI flags dependency versions containing HIGH or CRITICAL severity advisories. If flagged, update the vulnerable package to a secure version.

### Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/StellarInsure.git
cd StellarInsure

# Smart contracts
cd smartcontract
cargo build
cargo test

# Frontend
cd ../frontend
npm install
npm run dev

# Backend
cd ../backend
pip install -r requirements.txt
python src/main.py
```

## Coding Standards

### Rust (Smart Contracts)
- Follow Rust standard style (`cargo fmt`)
- Write tests for new functionality
- Document public functions
- Keep functions focused and small

### TypeScript/JavaScript (Frontend)
- Use TypeScript for type safety
- Follow ESLint rules
- Write meaningful component names
- Add comments for complex logic

### Python (Backend)
- Follow PEP 8 style guide
- Type hints for function signatures
- Docstrings for classes and functions
- Write unit tests

## Commit Message Format

```
type(scope): subject

body (optional)

footer (optional)
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions or changes
- `chore`: Maintenance tasks

Examples:
```
feat(contracts): add weather insurance policy type
fix(api): resolve oracle data validation error
docs(readme): update installation instructions
```

## Areas Needing Help

Look for issues tagged with:
- `good-first-issue`: Great for newcomers
- `help-wanted`: Community assistance needed
- `bug`: Something isn't working
- `enhancement`: New feature or improvement

## Priority Areas

- Oracle integrations
- Actuarial risk modeling
- Frontend components
- Documentation improvements
- Test coverage

## Questions?

Feel free to ask questions in:
- GitHub Issues
- Pull Request discussions
- Community Discord (coming soon)

Thank you for contributing to StellarInsure!
