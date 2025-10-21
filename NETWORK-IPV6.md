# 🌐 MultiShiva - Support IPv4 et IPv6

Ce document explique comment MultiShiva gère les connexions réseau IPv4 et IPv6.

## 📊 Support actuel

MultiShiva supporte **à la fois IPv4 et IPv6** avec détection automatique :

### Mode Dual-Stack (Recommandé)

Par défaut, MultiShiva essaie de créer un socket **dual-stack** qui accepte à la fois :
- ✅ Connexions IPv6 natives
- ✅ Connexions IPv4 (via IPv4-mapped IPv6 addresses)

```
Host bind sur [::]:53421
  ↓
Accepte IPv4 : 192.168.1.100:53421
Accepte IPv6 : 2001:db8::1:53421
```

### Fallback IPv4

Si IPv6 n'est pas disponible sur le système, MultiShiva bascule automatiquement en mode **IPv4 uniquement** :

```
IPv6 indisponible
  ↓
Host bind sur 0.0.0.0:53421
  ↓
Accepte IPv4 uniquement : 192.168.1.100:53421
```

## 🔄 Scénarios de connexion

### ✅ Cas supportés

| Host        | Agent       | Status | Notes                                    |
|-------------|-------------|--------|------------------------------------------|
| IPv4        | IPv4        | ✅ OK  | Connexion IPv4 standard                  |
| IPv6        | IPv6        | ✅ OK  | Connexion IPv6 native                    |
| Dual-stack  | IPv4        | ✅ OK  | IPv4 via IPv4-mapped address             |
| Dual-stack  | IPv6        | ✅ OK  | IPv6 natif                               |
| IPv4        | IPv6        | ⚠️ NON | Nécessite passerelle/NAT64               |

### ⚠️ Mix IPv4/IPv6 (sans dual-stack)

Si le **host est IPv4 uniquement** et l'**agent est IPv6 uniquement** (ou vice-versa), ils **ne pourront pas se connecter directement**.

**Solutions :**

1. **DNS64/NAT64** (automatique sur certains réseaux)
2. **Tunnel IPv6-in-IPv4** (ou inverse)
3. **Passerelle de traduction** (au niveau routeur)
4. **S'assurer que les deux machines ont IPv4** (le plus simple)

## 🧪 Tester votre configuration réseau

### Vérifier le support IPv6 sur votre machine

#### Linux :
```bash
# Vérifier les adresses IPv6
ip addr show | grep inet6

# Tester la connectivité IPv6
ping6 ::1  # Loopback IPv6
ping6 google.com
```

#### macOS :
```bash
# Vérifier les adresses IPv6
ifconfig | grep inet6

# Tester la connectivité IPv6
ping6 ::1
ping6 google.com
```

### Identifier l'adresse IP de votre machine

#### Linux :
```bash
# IPv4
ip addr show | grep "inet " | grep -v 127.0.0.1

# IPv6 (adresses link-local et globales)
ip addr show | grep "inet6" | grep -v "::1"
```

#### macOS :
```bash
# IPv4
ifconfig | grep "inet " | grep -v 127.0.0.1

# IPv6
ifconfig | grep "inet6" | grep -v "::1"
```

### Tester la connexion avec nc (netcat)

#### IPv4 :
```bash
# Sur le host
cargo run -- -m host

# Sur l'agent, tester avec nc
nc -4 -zv 192.168.1.100 53421
```

#### IPv6 :
```bash
# Sur le host
cargo run -- -m host

# Sur l'agent, tester avec nc
nc -6 -zv 2001:db8::1 53421
# OU avec adresse link-local
nc -6 -zv fe80::1%en0 53421
```

## 🔍 mDNS et IPv6

Le module mDNS de MultiShiva (mdns-sd) **supporte IPv6** et annonce les adresses disponibles :

```rust
// Le PeerInfo peut contenir IPv4 ou IPv6
pub struct PeerInfo {
    pub address: IpAddr,  // Peut être V4 ou V6
    pub port: u16,
}
```

Lors de l'auto-discovery :
- Le host annonce **toutes ses adresses** (IPv4 et IPv6)
- L'agent choisit la **première adresse disponible**
- Si les deux sont sur le même réseau local, IPv4 ou IPv6 fonctionnera

## 📝 Configuration manuelle avec IPv6

### Fichier de configuration avec IPv6

```yaml
# multishiva-agent.yml
version: 1
self_name: "agent1"
mode: agent
port: 53421

# IPv6 littéral entre crochets
host_address: "[2001:db8::1]:53421"

# OU IPv4 classique
# host_address: "192.168.1.100:53421"

tls:
  psk: "change-this-to-a-secure-random-string"
```

### CLI avec IPv6

```bash
# IPv6 littéral (avec crochets)
cargo run -- -m agent --host "[2001:db8::1]:53421"

# IPv6 link-local avec interface
cargo run -- -m agent --host "[fe80::1%en0]:53421"

# IPv4 classique
cargo run -- -m agent --host "192.168.1.100:53421"
```

## 🐛 Dépannage IPv6

### Problème 1 : "Network unreachable" avec IPv6

**Cause :** Pas de connectivité IPv6 sur votre réseau.

**Solution :**
```bash
# Vérifier que vous avez une adresse IPv6 globale ou ULA
ip addr show | grep "inet6" | grep -v "fe80"

# Si seulement link-local (fe80::), votre réseau n'a pas IPv6
# → Utilisez IPv4 à la place
```

### Problème 2 : "Address already in use"

**Cause :** Un socket dual-stack peut bloquer à la fois IPv4 et IPv6.

**Solution :**
```bash
# Vérifier quel processus utilise le port
sudo lsof -i :53421
# OU
sudo ss -tlnp | grep 53421

# Tuer le processus si nécessaire
kill <PID>
```

### Problème 3 : Mix IPv4/IPv6 ne fonctionne pas

**Cause :** Un host en IPv4-only ne peut pas communiquer avec un agent IPv6-only.

**Solution :**
```bash
# Option 1 : Activer IPv4 sur la machine IPv6-only
# (généralement automatique)

# Option 2 : Activer IPv6 sur la machine IPv4-only
# Vérifier la config réseau du routeur/DHCP

# Option 3 : Utiliser l'auto-discovery mDNS
# qui choisira automatiquement le bon protocole
cargo run -- -m agent  # Pas de --host
```

## 💡 Recommandations

### Pour une compatibilité maximale :

1. **Laisser l'auto-discovery faire son travail**
   ```bash
   cargo run -- -m host    # Sur le contrôleur
   cargo run -- -m agent   # Sur les agents
   ```

2. **Si connexion manuelle, préférer IPv4**
   - Plus universel
   - Moins de problèmes de firewall
   - Configuration plus simple

3. **Utiliser IPv6 si :**
   - Votre réseau est IPv6-native
   - Vous avez besoin de connectivité globale
   - Vous voulez éviter le NAT

4. **Mode dual-stack** (par défaut)
   - Fonctionne dans 99% des cas
   - Pas de configuration nécessaire
   - Fallback automatique vers IPv4

## 🔬 Détails techniques

### Comment fonctionne le dual-stack

Quand MultiShiva bind sur `[::]:53421` :

```
[::]:53421  (IPv6 wildcard)
  │
  ├─ Accepte IPv6 natif : 2001:db8::1 → [2001:db8::1]:53421
  │
  └─ Accepte IPv4 mappé : 192.168.1.100 → [::ffff:192.168.1.100]:53421
     (IPv4-mapped IPv6 address)
```

Le kernel gère automatiquement la conversion IPv4 → IPv6-mapped.

### Ordre de priorité dans le code

1. **Essayer IPv6 dual-stack** `[::]:{port}`
   - Si succès : accepte IPv4 + IPv6
   - Si échec : fallback IPv4

2. **Fallback IPv4** `0.0.0.0:{port}`
   - Accepte IPv4 uniquement
   - Sur toutes les interfaces

3. **Jamais** `127.0.0.1:{port}`
   - Localhost uniquement
   - Ne permettrait pas les connexions réseau

## 📊 Logs de débogage

Pour voir quel protocole est utilisé :

```bash
RUST_LOG=debug cargo run -- -m host
```

Logs attendus :
```
DEBUG multishiva::core::network: Bound to IPv6 dual-stack address [::]:53421
```

OU

```
DEBUG multishiva::core::network: IPv6 not available, falling back to IPv4
```

## ✅ Résumé

- ✅ **IPv4** : Supporté (fallback par défaut)
- ✅ **IPv6** : Supporté (préféré si disponible)
- ✅ **Dual-stack** : Supporté (mode par défaut)
- ✅ **mDNS** : Supporte IPv4 et IPv6
- ⚠️ **Mix IPv4/IPv6** : Nécessite dual-stack ou passerelle
- 🎯 **Recommandation** : Laisser l'auto-discovery gérer

En pratique, **vous n'avez rien à faire** - MultiShiva choisira automatiquement le meilleur protocole disponible ! 🚀
