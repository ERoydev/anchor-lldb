# Anchor Debug Wrapper Generator  
- Generates a standalone Rust crate that lets you simulate and debug program instructions without deploying to Solana.

---

## It scaffolds a standalone Rust crate with:
- `main.rs`: runs all instructions with mock accounts and test data  
- `mock.rs`: mocks for system program, signers, and PDAs  
- `Cargo.toml`: links to your Anchor crate via local path + package name  

---

# Why?
- Easily simulate and debug instructions locally without deploying to Solana.  
- Quickly iterate on instruction logic, argument setup, and PDA derivation.  
- No need for a local validator or on-chain deployment.  
- Get the package name from `your_project_name/programs/your_project_name/Cargo.toml`

---

# Usage:
```bash
anchor-lldb generate --package=<your_package_name_here> 
```
# Anchor account debugging in LLDB
1. View the pointer to the account
```lldb
p shipment
```

3. Dereference the pointer to see the actual struct fields
```lldb
p *shipment
```
