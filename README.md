# licensegen — Deterministic License Generator (Rust)
![Rust Version](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-under--construction-yellow)
![Linktech](https://img.shields.io/badge/Linktech-licensegen-blueviolet)

This repository contains the next-generation deterministic license generation engine for Linktech Engineering.

⚠️ **This project is currently under active development.**  
The architecture, schema, and APIs are evolving rapidly as the deterministic ID model, sync pipeline, and product/application loaders are finalized.

# 📘 **licensegen — Vendor‑Side License Generator (Rust)**

`licensegen` is the vendor‑side license generator for Linktech Engineering LLC.  
It produces signed, tamper‑evident license payloads for commercial applications, including BotScanner and future Linktech products.

This repository serves as the public home for the project as development progresses.

---

## 🔍 **Overview**

`licensegen` provides a deterministic, offline‑capable workflow for generating and validating software licenses.  
It is designed with a strong emphasis on:

- **Security** — cryptographic signing and verification  
- **Determinism** — reproducible outputs for audit transparency  
- **Offline operation** — no external dependencies required  
- **Vendor‑side control** — all sensitive operations remain with the publisher  
- **Structured logging** — consistent, audit‑friendly logs  
- **Database support** — optional MariaDB integration for vendor‑side tracking  
- **Clean separation of concerns** — crypto, models, storage, and CLI kept isolated  

The project is implemented in **Rust** for safety, performance, and long‑term maintainability.

---

## 🛠️ **Planned Capabilities**

The initial release of `licensegen` is expected to include:

- RSA keypair generation  
- License creation from YAML request files  
- License validation  
- Product and edition modeling  
- JSON‑schema‑driven license structure  
- **MariaDB support for vendor‑side license tracking**  
- **Structured logging for audit‑transparent operation**  
- Optional vendor‑side database population  
- Support for long‑term key rotation  
- Clean, predictable CLI commands  

Additional features will be documented as the project evolves.

---

## 🗺️ **Roadmap**

### **Phase 1 — Foundations**
- Project scaffolding  
- RSA keypair generation  
- Basic CLI structure  
- Logging framework  
- YAML request parsing  

### **Phase 2 — License Engine**
- License model implementation  
- JSON schema validation  
- Signing and verification  
- Deterministic output guarantees  

### **Phase 3 — Database Integration**
- MariaDB schema  
- Vendor‑side license tracking  
- Optional database population  

### **Phase 4 — Release Preparation**
- Documentation  
- Examples  
- Packaging  
- v0.1.0 release  

---

## 🎯 **Milestone: v0.1.0 (Initial Release)**

The first public milestone will include:

- RSA keypair generation  
- License creation and validation  
- YAML‑driven request workflow  
- JSON‑schema‑based license structure  
- Structured logging  
- Optional MariaDB support  
- Deterministic output guarantees  
- Basic CLI commands  

This milestone establishes the foundation for future expansion and integration with Linktech Engineering products.

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
