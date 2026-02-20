# 📘 **licensegen — Vendor‑Side License Generator (Rust)**

`licensegen` is the upcoming vendor‑side license generator for Linktech Engineering LLC.  
It is designed to produce signed, tamper‑evident license payloads for commercial applications, including BotScanner and future Linktech products.

This repository serves as the public home for the project as development progresses.

---

## 🔍 **Overview**

`licensegen` will provide a deterministic, offline‑capable workflow for generating and validating software licenses.  
The tool is being built with a focus on:

- **Security** — cryptographic signing and verification  
- **Determinism** — reproducible outputs for audit transparency  
- **Offline operation** — no external dependencies required  
- **Vendor‑side control** — all sensitive operations remain with the publisher  
- **Clear separation of concerns** — crypto, models, storage, and CLI kept cleanly isolated  

The project is implemented in **Rust** to ensure safety, performance, and long‑term maintainability.

---

## 🛠️ **Planned Capabilities**

The initial release of `licensegen` is expected to include:

- RSA keypair generation  
- License creation from YAML request files  
- License validation  
- Product and edition modeling  
- JSON‑schema‑driven license structure  
- Optional vendor‑side database population  
- Support for long‑term key rotation  
- Clean, predictable CLI commands  

Additional features will be documented as the project evolves.

---

## 📦 **Project Status**

This repository currently contains the initial documentation and project structure.  
Source code will be added as the implementation reaches its first stable milestone.

---

## 🧾 **License**

This project is released under the **MIT License**.  
See the `LICENSE` file for details.

---

## 🏢 **About Linktech Engineering LLC**

Linktech Engineering LLC specializes in automation, diagnostics, and platform‑grade tooling with a focus on deterministic workflows and audit‑transparent engineering practices.
