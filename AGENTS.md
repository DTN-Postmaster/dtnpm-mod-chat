# Agent Guide — `dtnpm-mod-chat`

## 🎯 Mission
Fournir le module applicatif complet (Fullstack WebModule) de messagerie instantanée DTN pour **DTN Postmaster**, comprenant le service BPA (`ChatService`), la persistance locale SQLite des conversations, l'API REST `/api/chat/v1/...` et le composant d'interface Preact sécurisé `<ChatView />`.

---

## 🛡️ Référentiels et Exigences de Sécurité

### 1. Cyber Resilience Act (CRA) & BPSec (RFC 9172)
- **Vérification BPSec Stricte par Défaut** :
  - En mode `VerifyPolicy::Strict`, tout bundle entrant dont le statut n'est pas strictement `Valid` (c'est-à-dire `Unsigned`, `MissingKey` ou `Invalid`) DOIT être rejeté et abandonné avant stockage ou notification.
  - Les politiques permissives (`Warn`, `Ignore`) sont des opt-ins explicites configurables pour les environnements de test.
- **Bornage de la Durée de Vie des Bundles** :
  - Le calcul des dates d'expiration (`lifetime_secs * 1000`) doit être saturé ou plafonné pour éviter tout débordement arithmétique `u64`.
- **Supply-Chain & VEX** :
  - Audit `cargo-audit` et `cargo-deny` vérifié ; VEX `not_affected` pour `RUSTSEC-2026-0459`.
  - Actions CI épinglées par SHA-1 à 40 caractères.
- **Politique de sécurité** : Consignée dans [SECURITY.md](SECURITY.md).

### 2. OWASP & Sécurité Frontend
- **Protection XSS sur le Rendu Markdown** :
  - Le parseur Markdown utilise des nonces cryptographiques aléatoires (`crypto.getRandomValues`) pour l'échappement des balises et autolinks, prévenant toute collision ou injection malveillante.
  - Les images distantes sont désactivées par défaut (`allowExternalImages: false`) pour interdire toute fuite de métadonnées ou de géolocalisation IP.
  - Aucune expression régulière linéaire vulnérable au ReDoS n'est tolérée dans le parseur.
- **Robustesse de l'Interface (`ChatView.tsx`)** :
  - Normalisation rigoureuse des EID (`normalizeChatTarget`) lors du filtrage des conversations actives.
  - Gestion silencieuse des erreurs d'acquittement de lecture (`.catch(() => {})`).

---

## 🦀 Standards d'Ingénierie Rust

1. **Règle Zéro Panic / Zéro Unwrap** :
   - Tout le code sous `src/` doit être 100% exempt de `.unwrap()`, `.expect()` et de macro `panic!`.
   - Linter strict : `-D warnings -D clippy::unwrap_used -D clippy::panic`.
2. **ADRs Obligatoires** :
   - Toute décision architecturale Chat/BPSec doit être consignée sous `adr/` et indexée dans [adr/README.md](adr/README.md).

---

## ⚡ Commandes Critiques de Vérification

- `cargo fmt --all -- --check`
- `cargo clippy --offline --workspace --lib --bins -- -D warnings -D clippy::unwrap_used -D clippy::panic`
- `cargo test --offline --all-features`
- `./scripts/check-security.sh --strict`
- `./scripts/check-coverage.sh --summary-only`
