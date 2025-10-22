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
- 🔄 **Contrôle bidirectionnel** - Transfert automatique du focus dans les deux sens
- 🔒 **Device Grabbing (Linux)** - Blocage intelligent de l'input local avec evdev
- 🐧 **Support Wayland natif** - Compatible X11 et Wayland via evdev
- 🎨 **Interface graphique complète** - Configuration visuelle avec drag & drop (comme GNOME display settings)
- 🖼️ **Éditeur de topologie** - Positionnez vos machines visuellement et créez des connexions
- 📊 **Monitoring temps réel** - StatusBar avec statistiques réseau et état des connexions
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

#### Interface graphique (disponible depuis v1.2.0)

```bash
# Mode développement avec hot-reload
cargo tauri dev

# Build production
cargo tauri build

# Lancer l'interface après build
./target/release/multishiva-gui
```

---

## 🎨 Interface graphique

### Lancement

```bash
# Mode développement (recommandé pour débuter)
cargo tauri dev

# Build production (crée un exécutable optimisé)
cargo tauri build
```

### Fonctionnalités de l'interface

#### 📐 Onglet Topology - Éditeur visuel
- **Drag & drop** : Déplacez les machines sur le canvas pour définir leur position relative
- **Ajouter des machines** : Bouton "+ Add Machine" pour créer de nouveaux agents
- **Créer des connexions** :
  1. Sélectionnez une machine source
  2. Cliquez sur "Connect"
  3. Choisissez la machine cible et le bord (Left/Right/Top/Bottom)
- **Design** :
  - Host = Gradient violet/indigo
  - Agents = Gradient bleu/cyan
  - Connexions = Flèches directionnelles bleues animées
- **Résumé temps réel** : Affichage du nombre de machines et connexions

#### ⚙️ Onglet Settings - Configuration
- **General** :
  - Nom de la machine (self_name)
  - Mode (Host ou Agent)
  - Port réseau
  - Adresse du host (mode agent uniquement)
  - PSK (Pre-Shared Key) avec générateur automatique
- **Hotkeys** :
  - Focus Return (retour au host)
  - Kill Switch (arrêt d'urgence)
- **Behavior** :
  - Edge Threshold (distance du bord en pixels)
  - Friction (délai avant transition en ms)
  - Reconnect Delay (délai de reconnexion en ms)

#### 📊 StatusBar - Monitoring
- **Statut de connexion** : Indicateur visuel (vert = connecté, rouge = déconnecté)
- **Mode actuel** : HOST ou AGENT
- **Machines connectées** : Nombre d'agents en ligne
- **Événements/s** : Débit en temps réel
- **Statistiques réseau** : Bytes envoyés/reçus
- **Features actives** : mDNS, Clipboard Sync
- **Détails extensibles** : Cliquez pour voir plus d'infos (latence, CPU, etc.)

### Sauvegarde de la configuration

La configuration est automatiquement chargée depuis `~/.config/multishiva/multishiva.yml` au démarrage.

Utilisez le bouton **Save** pour persister vos modifications. Le chemin du fichier de config est affiché en haut du panneau Settings.

---

## ⚙️ Configuration (manuelle)

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

### Linux (Wayland/X11)

MultiShiva utilise **evdev** pour un support natif de Wayland et X11 :

```bash
# Ajouter votre utilisateur au groupe input (REQUIS pour evdev)
sudo usermod -a -G input $USER

# Déconnectez-vous puis reconnectez-vous pour appliquer les changements
# Vérifiez l'appartenance au groupe :
groups | grep input
```

**Fonctionnalités Linux :**
- ✅ Support natif Wayland via evdev
- ✅ Compatible X11
- ✅ Device grabbing automatique (blocage input local quand focus distant)
- ✅ Auto-détection des périphériques clavier/souris
- ⚠️ Nécessite l'appartenance au groupe `input`

**Alternative (non recommandé pour Wayland) :**
```bash
# Si vous utilisez X11 uniquement et préférez rdev :
sudo apt-get install libx11-dev libxtst-dev
# Note: rdev ne fonctionne pas sur Wayland
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

## 🔄 Fonctionnement du transfert de focus

MultiShiva implémente un **contrôle bidirectionnel transparent** :

### Transfert Host → Agent

1. **Détection de bord** : Le host détecte quand la souris atteint un bord configuré (ex: bord gauche)
2. **Device grabbing** : Sur Linux, les périphériques sont "grabés" via EVIOCGRAB pour bloquer l'OS local
3. **Envoi FocusGrant** : Le host envoie l'événement `FocusGrant` à l'agent
4. **Forward des events** : Tous les événements clavier/souris sont envoyés à l'agent via TCP/MessagePack
5. **Injection distante** : L'agent injecte les événements sur sa machine locale

### Retour Agent → Host

1. **Détection locale** : L'agent monitore sa propre souris quand il a le focus
2. **Edge opposé** : Quand la souris atteint le bord opposé (ex: bord droit), l'agent détecte le retour
3. **Envoi FocusRelease** : L'agent envoie `FocusRelease` au host via le même canal TCP
4. **Device ungrab** : Le host libère les périphériques sur Linux
5. **Reprise locale** : Le host reprend le contrôle local

### Exemple de flux

```
Linux (Host)                          Mac (Agent)
     │                                     │
     │  Mouse → Left Edge                 │
     │  🔒 Grab devices                   │
     ├─────── FocusGrant ─────────────────>│
     │                                     │  ▶ Has focus
     ├─────── MouseMove events ──────────>│  📍 Inject locally
     ├─────── KeyPress events ───────────>│  ⌨️  Inject locally
     │                                     │
     │                                     │  Mouse → Right Edge
     │  ◀ Lose focus                      │
     │<────── FocusRelease ─────────────────┤
     │  🔓 Ungrab devices                 │
     │  📍 Resume local control           │
```

Cette architecture garantit qu'**un seul système traite les événements à la fois**, évitant les mouvements de curseur dupliqués.

---

## 🏗️ Architecture

```
multishiva/
├── src/
│   ├── main.rs              # Point d'entrée, modes host/agent
│   ├── cli.rs               # Interface CLI
│   ├── app.rs               # Lancement GUI (Tauri)
│   └── core/
│       ├── config.rs        # Configuration YAML
│       ├── network.rs       # Protocole TCP/TLS bidirectionnel
│       ├── input.rs         # Capture/injection I/O (rdev)
│       ├── input_evdev.rs   # Handler Linux natif (Wayland/X11)
│       ├── topology.rs      # Mapping spatial des machines
│       ├── focus.rs         # Gestion du focus
│       ├── events.rs        # Types d'événements (MouseMove, KeyPress, FocusGrant/Release)
│       ├── discovery.rs     # Auto-découverte mDNS
│       ├── clipboard.rs     # Synchronisation presse-papier
│       └── keyring.rs       # Stockage sécurisé des clés
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
| **v1.1**    | ✅   | **Support evdev/Wayland, Device grabbing, Contrôle bidirectionnel - 60+ tests** |
| **v1.2**    | ✅   | **Interface GUI complète avec éditeur visuel de topologie - 60+ tests** |
| v1.3    | 📋   | Multi-écrans avancé par machine       |
| v1.4    | 📋   | Transfert de fichiers                |
| v1.5    | 📋   | Profils de configuration multiples    |
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