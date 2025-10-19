# 🤝 Guide de contribution à MultiShiva

Merci de votre intérêt pour contribuer à MultiShiva ! Ce document vous guide à travers le processus de contribution.

## 📋 Table des matières

- [Code of Conduct](#code-of-conduct)
- [Comment puis-je contribuer ?](#comment-puis-je-contribuer)
- [Setup environnement de développement](#setup-environnement-de-développement)
- [Standards de code](#standards-de-code)
- [Processus de Pull Request](#processus-de-pull-request)
- [Tests](#tests)
- [Signaler des bugs](#signaler-des-bugs)
- [Proposer des fonctionnalités](#proposer-des-fonctionnalités)

---

## Code of Conduct

### Nos engagements

- **Respectueux** : Soyez respectueux et constructif dans vos interactions
- **Inclusif** : Accueillir tous les contributeurs, quel que soit leur niveau
- **Collaboratif** : Aider les autres contributeurs et être ouvert aux retours
- **Professionnel** : Maintenir un environnement professionnel et courtois

### Comportements inacceptables

- Langage ou images à caractère sexuel
- Trolling, insultes ou commentaires dégradants
- Harcèlement public ou privé
- Publication d'informations privées sans permission

---

## Comment puis-je contribuer ?

### Signaler des bugs

1. Vérifiez que le bug n'a pas déjà été signalé dans les [Issues](https://github.com/yrbane/multishiva/issues)
2. Créez une nouvelle issue en utilisant le template "Bug Report"
3. Incluez un maximum d'informations :
   - OS et version
   - Version de Rust
   - Steps pour reproduire
   - Logs pertinents

### Proposer des fonctionnalités

1. Vérifiez que la fonctionnalité n'est pas déjà proposée
2. Créez une issue avec le template "Feature Request"
3. Discutez de la fonctionnalité avant de commencer l'implémentation
4. Attendez le feedback des mainteneurs

### Contribuer du code

1. **Trouvez une issue** à résoudre (ou créez-en une)
2. **Commentez** sur l'issue pour indiquer que vous travaillez dessus
3. **Forkez** le repository
4. **Créez une branche** : `git checkout -b feature/issue-XX-description`
5. **Développez** votre solution en suivant l'approche TDD
6. **Testez** votre code localement
7. **Commitez** avec des messages clairs référençant l'issue
8. **Poussez** votre branche : `git push origin feature/issue-XX-description`
9. **Ouvrez une Pull Request** en utilisant le template

---

## Setup environnement de développement

### Prérequis

```bash
# Rust 1.70+ (édition 2021)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js 18+ (pour l'interface Tauri, optionnel pour le core)
# Voir https://nodejs.org/
```

### Installation

```bash
# Cloner le repository
git clone https://github.com/yrbane/multishiva.git
cd multishiva

# Installer les git hooks
./install-hooks.sh

# Builder le projet
cargo build

# Lancer les tests
cargo test
```

### Dépendances système

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get install libx11-dev libxtst-dev
```

#### macOS
```bash
# Autoriser dans : Préférences Système → Sécurité → Accessibilité
```

#### Windows
```bash
# Aucune dépendance spéciale
```

---

## Standards de code

### Approche TDD (Test-Driven Development)

**MultiShiva suit strictement l'approche TDD :**

1. **🔴 Red** : Écrire un test qui échoue
2. **🟢 Green** : Écrire le minimum de code pour faire passer le test
3. **🔵 Refactor** : Améliorer le code tout en gardant les tests verts

### Règles de code

#### Formatage
```bash
# Formater le code (obligatoire avant commit)
cargo fmt --all

# Vérifier le formatage
cargo fmt --all -- --check
```

#### Linting
```bash
# Linter sans warnings (obligatoire avant commit)
cargo clippy --all-targets --all-features -- -D warnings
```

#### Tests
```bash
# Tous les tests doivent passer
cargo test --all

# Tests avec coverage (optionnel)
cargo tarpaulin --ignore-tests
```

### Conventions de nommage

- **Modules** : `snake_case` (ex: `network.rs`, `focus_manager.rs`)
- **Structures** : `PascalCase` (ex: `NetworkManager`, `Config`)
- **Fonctions** : `snake_case` (ex: `start_host`, `transfer_focus`)
- **Constants** : `SCREAMING_SNAKE_CASE` (ex: `DEFAULT_PORT`)

### Documentation

```rust
/// Description courte de la fonction.
///
/// # Arguments
/// * `param` - Description du paramètre
///
/// # Returns
/// Description de la valeur retournée
///
/// # Examples
/// ```
/// let result = my_function(param);
/// ```
pub fn my_function(param: Type) -> ReturnType {
    // Implementation
}
```

### Messages de commit

**Format** : `type: description (closes #XX)`

**Types** :
- `feat`: Nouvelle fonctionnalité
- `fix`: Correction de bug
- `docs`: Documentation
- `test`: Ajout/modification de tests
- `refactor`: Refactoring sans changement de fonctionnalité
- `ci`: Changements CI/CD
- `perf`: Amélioration de performance

**Exemples** :
```
feat: add network module (closes #6)
fix: resolve edge detection bug (#15)
docs: update README with installation steps
test: add topology tests (refs #4)
```

**IMPORTANT** : Toujours référencer une issue dans vos commits !

---

## Processus de Pull Request

### Avant de soumettre

- [ ] Les tests passent localement (`cargo test`)
- [ ] Le code est formaté (`cargo fmt --all`)
- [ ] Pas de warnings clippy (`cargo clippy -- -D warnings`)
- [ ] Les tests couvrent les nouvelles fonctionnalités
- [ ] La documentation est à jour
- [ ] Les commits référencent des issues

### Soumission

1. **Ouvrir une PR** avec le template fourni
2. **Remplir le template** complètement
3. **Lier l'issue** correspondante
4. **Attendre la review** des mainteneurs
5. **Apporter les modifications** demandées si nécessaire

### Processus de review

- Les mainteneurs revieweront votre PR dans un délai de 3-5 jours
- Des changements peuvent être demandés
- Une fois approuvée, la PR sera mergée par un mainteneur
- Votre contribution apparaîtra dans la prochaine release !

---

## Tests

### Types de tests

1. **Tests unitaires** : Dans chaque fichier source
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_my_function() {
           assert_eq!(my_function(), expected);
       }
   }
   ```

2. **Tests d'intégration** : Dans `tests/`
   ```rust
   #[tokio::test]
   async fn test_integration() {
       // Test end-to-end
   }
   ```

3. **Tests de propriétés** : Avec `quickcheck` (optionnel)

### Couverture

- **Objectif minimum** : 80% de couverture
- **Vérifier la couverture** : `cargo tarpaulin --ignore-tests`

### Lancer les tests

```bash
# Tous les tests
cargo test

# Tests spécifiques
cargo test test_network

# Tests d'intégration uniquement
cargo test --test test_integration

# Avec output verbeux
cargo test -- --nocapture
```

---

## Signaler des bugs

### Informations nécessaires

1. **Description** claire du bug
2. **Étapes de reproduction** détaillées
3. **Comportement attendu** vs actuel
4. **Environnement** :
   - OS et version
   - Version Rust
   - Version MultiShiva
5. **Logs** pertinents
6. **Screenshots** si applicable

### Template

Utilisez le template "Bug Report" dans les issues.

---

## Proposer des fonctionnalités

### Avant de proposer

1. Vérifiez que la fonctionnalité n'existe pas déjà
2. Consultez la [roadmap](https://github.com/yrbane/multishiva/milestones)
3. Discutez dans [Discussions](https://github.com/yrbane/multishiva/discussions) si incertain

### Informations nécessaires

1. **Problème à résoudre** : Quel besoin cette fonctionnalité comble ?
2. **Solution proposée** : Comment implémenteriez-vous cette fonctionnalité ?
3. **Alternatives** : Quelles autres approches avez-vous envisagées ?
4. **Impact** : Qui bénéficiera de cette fonctionnalité ?

### Template

Utilisez le template "Feature Request" dans les issues.

---

## Questions ?

- 💬 [Discussions GitHub](https://github.com/yrbane/multishiva/discussions)
- 📧 Issues pour les questions techniques
- 📚 [Documentation](https://github.com/yrbane/multishiva#readme)

---

**Merci de contribuer à MultiShiva ! 🕉️**

Chaque contribution, quelle que soit sa taille, fait la différence.
