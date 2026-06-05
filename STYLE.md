# StellarInsure Style Guide

## General Principles

- Write clear, readable code
- Favor simplicity over cleverness
- Document complex logic
- Keep functions focused and small
- Write tests for new functionality

## Rust (Smart Contracts)

### Formatting
```rust
// Use cargo fmt for automatic formatting
cargo fmt

// Run clippy for linting
cargo clippy
```

### Naming Convention
- Functions: `snake_case`
- Structs/Enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

### Error Handling
```rust
// Use Result types
pub fn create_policy(...) -> Result<u64, Error> {
    // Check conditions
    if coverage_amount <= 0 {
        return Err(Error::InvalidAmount);
    }
    // ...
}
```

### Documentation
```rust
/// Creates a new insurance policy
/// 
/// # Arguments
/// * `policyholder` - Address of the policyholder
/// * `coverage_amount` - Maximum payout amount
/// 
/// # Returns
/// * `Result<u64, Error>` - Policy ID or error
pub fn create_policy(...) -> Result<u64, Error> {
    // implementation
}
```

## TypeScript/JavaScript (Frontend)

### Formatting
- Use Prettier for consistent formatting
- 2 spaces for indentation
- Semicolons required

### Components
```typescript
// Use functional components with TypeScript
interface PolicyCardProps {
  policy: Policy;
  onClaim: (policyId: number) => void;
}

export const PolicyCard: React.FC<PolicyCardProps> = ({ policy, onClaim }) => {
  // implementation
};
```

### File Organization
```
components/
  PolicyCard/
    PolicyCard.tsx
    PolicyCard.test.tsx
    PolicyCard.module.css
```

## Python (Backend)

### Formatting
```python
# Use Black for formatting
black src/

# Use isort for imports
isort src/
```

### Type Hints
```python
from typing import List, Optional

def get_policies(
    status: Optional[str] = None,
    limit: int = 50
) -> List[PolicyResponse]:
    """Retrieve policies with optional filters."""
    # implementation
```

### Docstrings
```python
def calculate_premium(coverage: int, risk: float, duration: int) -> int:
    """
    Calculate premium for an insurance policy.
    
    Args:
        coverage: Coverage amount
        risk: Risk multiplier (0.0 to 1.0)
        duration: Policy duration in seconds
    
    Returns:
        Premium amount
    """
    # implementation
```

## Git Commits

### Format
```
type(scope): subject

body (optional)

footer (optional)
```

### Examples
```
feat(contracts): add flight delay insurance
fix(api): correct premium calculation
docs(readme): update deployment instructions
refactor(policy): optimize claim processing
test(insurance): add weather policy tests
```

## Testing

### Smart Contracts
```rust
#[test]
fn test_create_policy_with_valid_parameters() {
    // Arrange
    let env = Env::default();
    // ...
    
    // Act
    let result = contract.create_policy(...);
    
    // Assert
    assert!(result.is_ok());
}
```

### Frontend
```typescript
describe('PolicyCard', () => {
  it('renders policy details correctly', () => {
    // Test implementation
  });
});
```

### Backend
```python
def test_create_policy():
    """Test policy creation with valid parameters."""
    # Arrange
    policy_data = {...}
    
    # Act
    response = client.post("/api/policies", json=policy_data)
    
    # Assert
    assert response.status_code == 200
```

## Code Review Checklist

- [ ] Code follows style guidelines
- [ ] Tests pass
- [ ] Documentation updated
- [ ] No unnecessary complexity
- [ ] Error handling included
- [ ] Security considerations addressed
- [ ] Performance implications considered
