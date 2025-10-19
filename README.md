# 🕉️ MultiShiva

### *"Many arms. One mind."*

[![CI](https://github.com/yrbane/multishiva/workflows/CI/badge.svg)](https://github.com/yrbane/multishiva/actions)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![TDD](https://img.shields.io/badge/Development-TDD-green.svg)](https://github.com/yrbane/multishiva)

**MultiShiva** est une application Rust permettant de contrôler plusieurs ordinateurs (Linux, macOS, Windows) avec un seul clavier et une seule souris via votre réseau local.

---

## 🌟 Fonctionnalités

- 🖱️ **Partage clavier/souris multi-OS** - Contrôlez Linux, macOS et Windows depuis une seule machine
- 🧩 **Interface graphique intuitive** - Positionnez vos machines par glisser-déposer
- 🔐 **Sécurité TLS + PSK** - Chiffrement et authentification par clé pré-partagée
- 🌐 **Auto-découverte mDNS** - Détection automatique des machines sur le réseau
- 🔄 **Reconnexion automatique** - Résilient aux coupures réseau
- 🛑 **Kill-switch & hotkeys** - Reprenez le contrôle instantanément
- 🧪 **TDD & mode simulation** - Tests rigoureux sans matériel physique
- 💾 **Configuration persistante** - Vos préférences sauvegardées automatiquement

---

## 🚀 Démarrage rapide

### Installation

```bash
# Cloner le repository
git clone https://github.com/yrbane/multishiva.git
cd multishiva

# Compiler le projet
cargo build --release

# Installer le binaire
cargo install --path .
```

### Utilisation

#### Mode Host (machine maître)

```bash
# Avec la configuration par défaut (multishiva.yml)
RUST_LOG=info cargo run

# Ou avec le binaire compilé
./target/release/multishiva

# Avec une configuration spécifique
./target/release/multishiva --config /path/to/config.yml
```

#### Mode Agent (machines contrôlées)

```bash
# Copier la configuration exemple
cp multishiva-agent.yml.example multishiva-agent.yml

# Éditer et lancer
./target/release/multishiva --config multishiva-agent.yml
```

#### Mode Simulation (pour tester sans matériel)

```bash
RUST_LOG=info cargo run -- --simulate
```

#### Interface graphique (à venir v1.0)

```bash
multishiva --gui
```

---

## ⚙️ Configuration

### Exemple de configuration Host

Créez un fichier `multishiva.yml` :

```yaml
self_name: "desktop"
mode: host
port: 53421

tls:
  psk: "change-this-to-a-secure-random-string"

edges:
  right: "laptop"    # Machine à droite
  bottom: "macbook"  # Machine en bas
  # left: "other"
  # top: "another"

hotkeys:
  focus_return: "Ctrl+Alt+H"
  kill_switch: "Ctrl+Alt+K"

behavior:
  edge_threshold_px: 10
  friction_ms: 100
  reconnect_delay_ms: 5000
```

### Exemple de configuration Agent

```yaml
self_name: "laptop"
mode: agent
port: 53421
host_address: "192.168.1.100:53421"  # IP du host

tls:
  psk: "change-this-to-a-secure-random-string"  # MÊME clé que le host

edges:
  left: "desktop"    # Le host est à gauche
  right: "macbook"   # Autre machine à droite

behavior:
  edge_threshold_px: 10
  friction_ms: 100
  reconnect_delay_ms: 5000
```

---

## 🧾 Permissions système

### macOS

Autorisez MultiShiva dans :
**Préférences Système → Sécurité et confidentialité → Accessibilité**

### Linux (X11)

```bash
# Installer les dépendances
sudo apt-get install libx11-dev libxtst-dev

# Ajouter votre utilisateur au groupe input (optionnel)
sudo usermod -a -G input $USER
```

### Windows

Aucune configuration spéciale requise. Pour une utilisation optimale, signez le binaire.

---

## 🧪 Développement

### Prérequis

- Rust 1.70+ (édition 2021)
- Node.js 18+ (pour l'interface Tauri)

### Lancer les tests

```bash
# Tests unitaires
cargo test

# Tests d'intégration
cargo test --test test_integration

# Tests avec couverture
cargo tarpaulin --ignore-tests

# Linter
cargo clippy -- -D warnings

# Formatage
cargo fmt --all
```

### Développement GUI

```bash
# Mode développement avec hot-reload
cargo tauri dev

# Build production
cargo tauri build
```

### Mode simulation

Testez sans matériel physique :

```bash
# Host simulé
multishiva --mode host --simulate

# Agent simulé
multishiva --mode agent --simulate
```

---

## 🏗️ Architecture

```
multishiva/
├── src/
│   ├── main.rs              # Point d'entrée
│   ├── cli.rs               # Interface CLI
│   ├── app.rs               # Lancement GUI (Tauri)
│   └── core/
│       ├── config.rs        # Configuration YAML
│       ├── network.rs       # Protocole TCP/TLS
│       ├── input.rs         # Capture/injection I/O
│       ├── topology.rs      # Mapping spatial
│       ├── focus.rs         # Gestion du focus
│       └── events.rs        # Types d'événements
├── gui/
│   └── src/
│       ├── App.tsx
│       └── components/
├── tests/                   # Tests unitaires et intégration
└── docs/                    # Documentation
```

Consultez [IDEA.md](IDEA.md) pour le concept complet et les spécifications détaillées.

---

## 🔐 Sécurité

- **Chiffrement TLS** via `rustls` avec authentification PSK
- **Empreinte TLS** stockée localement et vérifiée à chaque connexion
- **Aucune donnée cloud** - Tout reste sur votre réseau local
- **Kill-switch global** - Interrompt immédiatement les connexions

> ⚠️ **Important** : Ne partagez jamais votre clé PSK. Générez une clé unique par réseau.

---

## 🗺️ Roadmap

| Version | État | Fonctionnalités                       |
|---------|------|---------------------------------------|
| v0.1    | ✅   | Config, CLI, topologie, réseau, focus, input, simulation - **30 tests** |
| v0.2    | ✅   | TLS fingerprint, permissions système - **34 tests** |
| v0.3    | ✅   | Logging avec rotation, stabilité - **41 tests** |
| **v1.0**    | ✅   | **Interface Tauri, mDNS, Clipboard, Keyring, GUI complète - 60 tests** |
| v1.1    | 📋   | Multi-écrans avancé par machine       |
| v1.2    | 📋   | Transfert de fichiers                |
| v1.3    | 📋   | Profils de configuration multiples    |
| v2.0    | 💡   | Application mobile compagnon          |

---

## 🤝 Contribution

Les contributions sont les bienvenues ! Ce projet suit une approche **TDD stricte**.

### Processus

1. Fork le projet
2. Créez une branche feature (`git checkout -b feature/amazing-feature`)
3. **Écrivez d'abord les tests** (cycle TDD)
4. Implémentez la fonctionnalité
5. Vérifiez que tous les tests passent (`cargo test`)
6. Commit (`git commit -m 'Add amazing feature'`)
7. Push (`git push origin feature/amazing-feature`)
8. Ouvrez une Pull Request

### Standards de code

- Tests requis pour toute nouvelle fonctionnalité
- Couverture minimale : 80%
- `cargo clippy` sans warnings
- `cargo fmt --all` appliqué

---

## 📚 Documentation

- [IDEA.md](IDEA.md) - Concept et spécifications détaillées
- [Cargo docs](https://docs.rs) - Documentation des crates
- [Milestones](https://github.com/yrbane/multishiva/milestones) - Feuille de route du projet

---

## 🙏 Inspirations

- [Barrier/Synergy](https://github.com/debauchee/barrier) - Partage clavier/souris open source
- [Input Leap](https://github.com/input-leap/input-leap) - Fork moderne de Barrier
- Philosophie Shiva - L'harmonie dans la multiplicité

---

## 📜 Licence

MIT © 2025

Créé avec 🦀 Rust, ☕ café et approche TDD rigoureuse.

---

## 🧘 Philosophie

> *"Contrôler le chaos sans le dominer."*

MultiShiva ne centralise pas, il **harmonise** : un seul esprit contrôlant plusieurs corps numériques en parfaite synchronisation.

Comme Shiva aux multiples bras, chaque bras agit de manière coordonnée sous une seule conscience.

---

## 📞 Support

- 🐛 [Issues](https://github.com/yrbane/multishiva/issues)
- 💬 [Discussions](https://github.com/yrbane/multishiva/discussions)