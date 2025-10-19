# 🍎 MultiShiva - Guide de configuration macOS

Ce guide vous aide à configurer MultiShiva sur macOS Sequoia (Tahoe) et versions ultérieures.

## ⚠️ Permissions requises

MultiShiva nécessite plusieurs permissions système pour fonctionner correctement :

### 1. Accessibilité (OBLIGATOIRE)

**Pourquoi ?** MultiShiva doit capturer et injecter des événements clavier/souris.

**Comment activer :**

1. Ouvrez **Réglages Système** (System Settings)
2. Allez dans **Confidentialité et sécurité** → **Accessibilité**
3. Cliquez sur le bouton **+** (ou sur le cadenas pour déverrouiller)
4. Sélectionnez votre application Terminal :
   - **Terminal.app** (par défaut)
   - **iTerm2** (si vous l'utilisez)
   - Ou le binaire compilé : `multishiva/target/release/multishiva`
5. Cochez la case à côté de l'application
6. **IMPORTANT** : Redémarrez complètement votre Terminal

**Vérification :**
```bash
# MultiShiva affichera un warning si la permission manque
cargo run -- -m agent
```

### 2. Connexions réseau

**Problème :** Le firewall macOS peut bloquer les connexions sortantes.

**Solution 1 - Désactiver temporairement le firewall pour tester :**
```bash
# Vérifier l'état
/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

# Dans Réglages Système → Réseau → Firewall
# Vous pouvez désactiver temporairement pour tester
```

**Solution 2 - Autoriser Terminal/multishiva :**
1. **Réglages Système** → **Réseau** → **Firewall** → **Options**
2. Ajoutez Terminal ou le binaire multishiva
3. Autorisez les connexions entrantes

### 3. Test de connectivité réseau

Avant de lancer MultiShiva, vérifiez la connexion au host :

```bash
# Remplacez par l'IP de votre machine host Linux
nc -zv 192.168.1.100 53421

# Si succès, vous verrez :
# Connection to 192.168.1.100 port 53421 [tcp/*] succeeded!

# Si échec :
# nc: connectx to 192.168.1.100 port 53421 (tcp) failed: Connection refused
```

## 🚀 Démarrage rapide sur macOS

### En tant qu'agent (se connecter à un host Linux)

**Option 1 : Auto-discovery (recommandé)**
```bash
# Le Mac va découvrir automatiquement le host sur le réseau
cargo run -- -m agent
```

**Option 2 : Spécifier l'adresse du host**
```bash
# Remplacez par l'IP de votre host Linux
cargo run -- -m agent --host 192.168.1.100:53421
```

### En tant que host (si vous voulez que le Mac soit le contrôleur principal)

```bash
cargo run -- -m host
```

## 🔧 Dépannage

### Problème 1 : "Connection refused (os error 61)"

**Causes possibles :**
- Le host n'est pas démarré
- Firewall bloque la connexion
- Mauvaise adresse IP

**Solutions :**
1. Vérifiez que le host est en cours d'exécution
2. Testez avec `nc -zv <IP> 53421`
3. Vérifiez votre firewall (voir section Permissions)
4. Vérifiez que vous êtes sur le même réseau (WiFi/Ethernet)

### Problème 2 : "Permission denied" ou événements non capturés

**Cause :** Permissions d'accessibilité manquantes

**Solution :**
1. Vérifiez les permissions d'accessibilité (voir section 1)
2. Redémarrez le Terminal après avoir accordé les permissions
3. Essayez de compiler en release : `cargo build --release`
4. Accordez les permissions au binaire directement : `target/release/multishiva`

### Problème 3 : "No host found" avec auto-discovery

**Causes possibles :**
- Le host n'a pas démarré son service mDNS
- Vous n'êtes pas sur le même réseau local
- Le firewall bloque le multicast mDNS

**Solutions :**
1. Vérifiez que le host affiche : "✓ Host registered on mDNS"
2. Attendez 5 secondes pour la découverte
3. Utilisez l'option `--host` manuellement

### Problème 4 : macOS Sequoia (Tahoe) - Restrictions supplémentaires

Sur macOS Sequoia, Apple a renforcé les restrictions de sécurité :

**Solution :**
```bash
# Compiler en release
cargo build --release

# Donner les permissions au binaire compilé plutôt qu'à cargo
# Réglages Système → Accessibilité → Ajouter :
# /Users/votre-nom/chemin/vers/multishiva/target/release/multishiva
```

## 📊 Script de diagnostic

Un script de diagnostic est fourni pour vous aider à identifier les problèmes :

```bash
./diagnose-macos.sh
```

Il vérifie :
- Version de macOS
- Connectivité réseau
- État du firewall
- Permissions d'accessibilité
- Environnement Rust

## 🔐 Configuration de sécurité

Créez un fichier de configuration pour l'agent :

```yaml
# multishiva.yml sur macOS
version: 1
self_name: "macbook"
mode: agent
port: 53421

# Optionnel : spécifiez l'adresse du host si l'auto-discovery ne fonctionne pas
# host_address: "192.168.1.100:53421"

tls:
  psk: "change-this-to-a-secure-random-string"  # DOIT être identique sur le host

behavior:
  reconnect_delay_ms: 5000
```

## 📝 Notes importantes

1. **PSK identique** : La clé pré-partagée (`psk`) doit être **exactement la même** sur le host et l'agent
2. **Même réseau** : Les machines doivent être sur le même réseau local (LAN)
3. **Redémarrage** : Après avoir accordé les permissions, redémarrez le Terminal
4. **Firewall** : Assurez-vous que le port 53421 n'est pas bloqué

## 🆘 Besoin d'aide ?

Si vous rencontrez toujours des problèmes :

1. Exécutez le script de diagnostic : `./diagnose-macos.sh`
2. Activez les logs détaillés : `RUST_LOG=debug cargo run -- -m agent`
3. Vérifiez les logs dans : `~/.local/share/multishiva/logs/`
4. Ouvrez une issue sur GitHub avec les logs

## ✅ Liste de vérification

Avant de lancer MultiShiva sur macOS :

- [ ] Permissions d'accessibilité accordées à Terminal/multishiva
- [ ] Terminal redémarré après avoir accordé les permissions
- [ ] Host Linux en cours d'exécution
- [ ] Connectivité réseau testée avec `nc -zv`
- [ ] Firewall configuré ou désactivé pour tester
- [ ] Même PSK sur host et agent
- [ ] Même réseau local (WiFi/Ethernet)

## 🎯 Test rapide

```bash
# Sur le host Linux :
cargo run -- -m host

# Sur le Mac (dans un autre terminal) :
# Attendez de voir "✓ Host registered on mDNS" sur le host

# Puis :
cargo run -- -m agent

# Vous devriez voir :
# 🔍 No host address specified, using mDNS auto-discovery...
# ✓ Found host 'host' at 192.168.1.100:53421
# ✓ Connected to host at 192.168.1.100:53421
```

Bon test ! 🚀
