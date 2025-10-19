# 🕉️ MultiShiva

### *“Many arms. One mind.”*

---

## 🧠 Concept

**MultiShiva** est une application écrite en **Rust** permettant d’utiliser **un seul clavier et une seule souris** pour contrôler plusieurs ordinateurs (**Linux, macOS, Windows**) connectés au **même réseau local**.

L’application vous permet de définir **la position relative de vos machines** (par glisser-déposer dans l’interface graphique), afin que **le curseur passe naturellement d’un écran à l’autre**, comme s’il s’agissait d’un seul espace de travail étendu.

Inspirée de **Shiva**, le dieu aux multiples bras, MultiShiva incarne la **synchronisation parfaite entre plusieurs esprits numériques**.

---

## 🦀 Objectifs techniques

* **Langage :** Rust (édition ≥ 2021)
* **Méthodologie :** TDD (Test Driven Development)
* **Plateformes :** Linux / macOS / Windows
* **Interface :** GUI en **Tauri (Rust + React + TypeScript)**
* **Un seul binaire** : agit en mode **host** (maître) ou **agent** (client) selon les arguments CLI.
* **Aucune dépendance cloud** : tout fonctionne en LAN, en pair-à-pair sécurisé.

---

## 🖥️ Fonctionnalités

| Fonction                                        | Description                                                                                      |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| 🖱️ **Partage clavier/souris**                  | Le host capture les entrées et les redirige vers d’autres machines selon la position du curseur. |
| 🌐 **Multi-OS LAN**                             | Fonctionne sur Linux, macOS et Windows sur le même réseau local.                                 |
| 🧩 **Mapping spatial visuel**                   | Interface graphique pour positionner les machines dans un espace 2D interactif.                  |
| 🔐 **Sécurité TLS + PSK**                       | Authentification par clé pré-partagée et chiffrement via `rustls`.                               |
| 🔄 **Reconnexion automatique**                  | Résilience aux pertes de connexion (reconnecte après coupure LAN).                               |
| 🧱 **Multi-écrans**                             | Détection automatique des écrans locaux (X11/Quartz/WinAPI).                                     |
| 🧪 **Développement piloté par les tests (TDD)** | Chaque module possède ses tests unitaires et d’intégration.                                      |
| 🛑 **Kill-switch & hotkeys**                    | Interruption immédiate du contrôle ou retour au host par raccourci clavier.                      |
| 💾 **Persistance automatique**                  | Sauvegarde automatique des préférences et topologie.                                             |
| 🧰 **Mode simulation**                          | Test des comportements réseau et d’interaction sans périphériques physiques.                     |

---

## 🧩 Architecture du projet

```
multishiva/
 ├── Cargo.toml
 ├── src/
 │   ├── main.rs              # Entrée principale CLI/GUI
 │   ├── cli.rs               # Parsing des options (--mode, --config, etc.)
 │   ├── app.rs               # Lancement GUI (Tauri)
 │   ├── core/
 │   │   ├── config.rs        # Chargement & validation du YAML
 │   │   ├── network.rs       # Protocole TCP/TLS + mDNS
 │   │   ├── input.rs         # Capture/injection clavier-souris (rdev)
 │   │   ├── topology.rs      # Gestion de la grille et des bords
 │   │   ├── focus.rs         # Franchissement de bord et transfert de focus
 │   │   └── events.rs        # Types d’événements
 │   ├── simulation.rs        # Mode simulation pour tests & debug
 │   └── logging.rs           # Gestion centralisée des logs
 ├── gui/
 │   ├── src/
 │   │   ├── App.tsx
 │   │   ├── components/
 │   │   │   ├── MachineGrid.tsx
 │   │   │   ├── SettingsPanel.tsx
 │   │   │   ├── SecurityPanel.tsx
 │   │   │   └── StatusBar.tsx
 │   └── tauri.conf.json
 ├── tests/
 │   ├── test_config.rs
 │   ├── test_network.rs
 │   ├── test_input.rs
 │   ├── test_focus.rs
 │   ├── test_topology.rs
 │   ├── test_integration.rs
 │   └── test_security.rs
 └── README.md
```

---

## 🧪 Développement piloté par les tests (TDD)

### Cycle :

1. **Écrire un test** unitaire ou d’intégration (rouge 🔴)
2. **Coder le minimum** pour le faire passer (vert 🟢)
3. **Refactoriser** pour clarté et respect de SOLID (bleu 🔵)

### Outils :

* `tokio-test` (tests asynchrones)
* `mockall` (mock de périphériques ou réseau)
* `assert_cmd` (tests CLI)
* `insta` (snapshots)
* `quickcheck` (tests de propriétés)
* `tracing-test` (analyse des logs pendant tests)

---

## ⚙️ Modes d’exécution

```bash
# Lancer le mode host
multishiva --mode host --config ./config.yaml

# Lancer le mode agent
multishiva --mode agent --config ./config.yaml

# Lancer la GUI
multishiva --gui

# Lancer en simulation
multishiva --mode host --simulate
```

---

## 🧱 Configuration YAML

### Exemple (host)

```yaml
self: "desktop-shiva"
mode: "host"
port: 53421
tls:
  psk: "SUPER_SECRET_KEY"
edges:
  right_of: "laptop-shiva"
  below: "mbp-shiva"
hotkeys:
  focus_return: "Ctrl+Ctrl"
  kill_switch: "Ctrl+Alt+Pause"
behavior:
  edge_threshold_px: 3
  friction_ms: 80
```

### Exemple (agent)

```yaml
self: "laptop-shiva"
mode: "agent"
host: "desktop-shiva.local"
port: 53421
tls:
  psk: "SUPER_SECRET_KEY"
behavior:
  reconnect_delay_ms: 1000
```

---

## 🌐 Réseau et découverte automatique

* **Protocole :** TCP/TLS via `tokio-rustls`
* **Sérialisation :** MessagePack (`rmp-serde`)
* **Découverte :** mDNS

  * Host annonce `_multishiva._tcp.local` toutes les 5 s
  * Agents détectent les hôtes disponibles et les affichent dans la GUI
  * Possibilité de saisie manuelle d’adresse IP/port

---

## 🧠 Logique de franchissement de bord

1. La souris atteint le bord d’un écran configuré (via `edge_threshold_px`)
2. Si un voisin est défini dans `edges`, un message `FocusGrant` est envoyé
3. L’agent cible reçoit le focus et place le curseur à la position relative correspondante
4. Le clavier suit automatiquement la machine détentrice du focus

Les écrans **locaux** (multi-moniteurs) sont détectés et ignorés :

> “MultiShiva ne bascule le contrôle que sur des bords réseau configurés.”

---

## 🧩 Interface graphique (Tauri)

L’interface permet de :

* Visualiser les machines dans une **grille 2D interactive**
* **Glisser-déposer** les icônes pour définir la topologie
* Configurer la sécurité, les hotkeys et le comportement
* Voir le **statut réseau** (latence, hôte courant, connexions)
* Activer/désactiver les modes host/agent

### Feedback visuel :

| État      | Couleur                     | Signification |
| --------- | --------------------------- | ------------- |
| 🟢 Vert   | Connecté                    |               |
| 🟠 Orange | En tentative de reconnexion |               |
| 🔴 Rouge  | Déconnecté ou erreur TLS    |               |
| ⚪ Gris    | Mode simulation             |               |

Les configurations sont sauvegardées automatiquement dans :
`$HOME/.config/multishiva/config.yaml`

---

## 🔐 Sécurité

* **TLS (rustls)** avec authentification par **clé pré-partagée (PSK)**
* **Empreinte TLS** affichée à la première connexion et stockée localement
* Si l’empreinte change, une **confirmation utilisateur** est exigée avant reconnexion
* **Kill-switch global** : interrompt immédiatement toute communication active
* **Pas de cloud, pas de collecte de données**

---

## 🧰 Crates principales

| Domaine                | Crate                                                        |
| ---------------------- | ------------------------------------------------------------ |
| Async & réseau         | `tokio`, `tokio-rustls`, `rmp-serde`                         |
| Entrées clavier/souris | `rdev`                                                       |
| Config & sérialisation | `serde`, `serde_yaml`                                        |
| CLI                    | `clap`                                                       |
| GUI                    | `tauri`, `react`, `typescript`                               |
| Logging                | `tracing`, `tracing-subscriber`                              |
| Tests                  | `tokio-test`, `mockall`, `assert_cmd`, `insta`, `quickcheck` |

---

## 🧾 Permissions système

| OS              | Action requise                                                                     |
| --------------- | ---------------------------------------------------------------------------------- |
| **macOS**       | Autoriser “MultiShiva” dans `Préférences Système → Sécurité → Accessibilité`.      |
| **Linux (X11)** | Nécessite `x11-dev` et droits `uinput` (ou fallback XTest).                        |
| **Windows**     | Fonctionne via `SendInput`, sans droits admin, mais signature binaire recommandée. |

MultiShiva vérifie automatiquement les permissions au lancement et affiche un message clair si une autorisation est manquante.

---

## 🧩 Mode Simulation

Permet de tester :

* Les échanges host ↔ agent
* Le mapping des bords
* Les timings de latence
  Sans accès réel au matériel.

```bash
multishiva --simulate --mode host
multishiva --simulate --mode agent
```

---

## 🧰 Journalisation

* Fichiers de log :
  `$HOME/.local/share/multishiva/logs/multishiva.log`
* Niveaux : `error`, `warn`, `info`, `debug`, `trace`
* Consultable depuis la GUI (panneau “Logs”)

---

## 🧩 Compilation & Build

### Identité Tauri :

```json
{
  "identifier": "com.multishiva.app",
  "productName": "MultiShiva"
}
```

### Commandes :

```bash
cargo tauri build --target universal-apple-darwin
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target x86_64-unknown-linux-gnu
```

---

## 🧪 Tests et couverture

```bash
# Tests unitaires
cargo test

# Tests d’intégration (simulations)
cargo test --test test_integration

# Couverture
cargo tarpaulin --ignore-tests

# Lancer GUI en mode dev
cargo tauri dev
```

---

## 🧭 Roadmap

| Version | Fonctionnalités clés                  |
| ------- | ------------------------------------- |
| ✅ v1.0  | CLI + base réseau + TDD               |
| ✅ v1.1  | TLS + PSK + config YAML               |
| 🔜 v1.2 | GUI Tauri (drag & drop)               |
| 🔜 v1.3 | Auto-discovery mDNS + feedback visuel |
| 🔜 v1.4 | Clipboard sync + multi-écrans avancé  |
| 🔜 v1.5 | Mobile companion app (Android/iOS)    |

---

## 🧘 Philosophie du projet

> *“Contrôler le chaos sans le dominer.”*
> MultiShiva ne centralise pas, il **harmonise** : un seul esprit, plusieurs corps.

---

## 📜 Licence

**MIT © 2025 – Projet open source Rust.**
Créé avec amour, tests unitaires et café ☕️