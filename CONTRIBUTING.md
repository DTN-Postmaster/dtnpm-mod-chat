# Guide de Contribution — `dtnpm-mod-chat`

Ce guide présente les règles de développement et les exigences de sécurité pour le module Chat de `dtnpm-mod-chat`.

---

## 1. Principes Fondamentaux

- **Zéro Panic / Zéro Unwrap** : Aucun appel `.unwrap()` ou `.expect()` en code de production.
- **Sécurité BPSec & XSS** :
  - Tout bundle entrant sous politique stricte doit être authentifié avec succès pour être délivré.
  - Le rendu Markdown côté UI doit impérativement utiliser les nonces cryptographiques et bloquer les images distantes par défaut.
- **Traçabilité Architecturale (ADR)** :
  - Tout changement de schéma de base de données, d'API ou de protocole doit faire l'objet d'un fichier ADR sous `adr/` et être indexé dans `adr/README.md`.

---

## 2. Vérification Locale

```bash
cargo fmt --all -- --check
cargo clippy --offline --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::panic
cargo test --offline --all-features
./scripts/check-security.sh --strict
```
