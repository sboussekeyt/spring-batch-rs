# Design : Filtrage via ItemProcessorResult

**Date** : 2026-04-07
**Statut** : Approuvé

## Contexte

Actuellement, `ItemProcessorResult<O> = Result<O, BatchError>`. Un processor doit toujours retourner un item ou une erreur — il est impossible de filtrer silencieusement un item. Ce design introduit le filtrage en passant le type de retour à `Result<Option<O>, BatchError>`.

## Changements

### 1. Type `ItemProcessorResult`

```rust
// Avant
pub type ItemProcessorResult<O> = Result<O, BatchError>;

// Après
pub type ItemProcessorResult<O> = Result<Option<O>, BatchError>;
```

Sémantique :
- `Ok(Some(item))` → item traité, transmis au writer
- `Ok(None)` → item filtré, ignoré silencieusement
- `Err(BatchError)` → erreur de traitement (skip/fail selon skip_limit)

### 2. Trait `ItemProcessor`

La signature de `process` devient :

```rust
fn process(&self, item: &I) -> ItemProcessorResult<O>;
// soit : fn process(&self, item: &I) -> Result<Option<O>, BatchError>
```

### 3. `StepExecution` — nouveau champ

```rust
pub filter_count: usize,  // items filtrés (processor a retourné Ok(None))
```

Initialisé à `0` dans `StepExecution::new()`.

### 4. `process_chunk()` — logique de filtrage

```rust
match self.processor.process(item) {
    Ok(Some(processed_item)) => {
        result.push(processed_item);
        step_execution.process_count += 1;
    }
    Ok(None) => {
        // Item filtré
        step_execution.filter_count += 1;
        debug!("Item filtered by processor");
    }
    Err(error) => {
        // Comportement existant : skip ou fail
        step_execution.process_error_count += 1;
        ...
    }
}
```

### 5. `PassThroughProcessor`

Retourne `Ok(Some(item.clone()))` — jamais de filtrage, comportement identique à aujourd'hui.

### 6. Processors existants

Tous les processors dans `src/` retournent `Ok(item)` → mettre à jour en `Ok(Some(item))`.

## Fichiers impactés

| Fichier | Nature du changement |
|---|---|
| `src/core/item.rs` | Nouveau type, doc mis à jour, `PassThroughProcessor` mis à jour, tests mis à jour |
| `src/core/step.rs` | `filter_count` dans `StepExecution`, logique `process_chunk`, doc mis à jour, tests mis à jour |
| `src/item/*/` (processors) | `Ok(item)` → `Ok(Some(item))` partout |
| `examples/filter_records_from_csv_with_processor.rs` | Nouvel exemple |
| `Cargo.toml` | Déclaration `[[example]]` du nouvel exemple |
| `website/src/content/docs/` | Mise à jour de la documentation |

## Exemple

**Fichier** : `examples/filter_records_from_csv_with_processor.rs`
**Features** : `csv,json`
**Scénario** : lit un CSV de personnes (nom, âge), filtre celles dont l'âge < 18, écrit les adultes en JSON.

```rust
struct AgeFilterProcessor;

impl ItemProcessor<Person, Person> for AgeFilterProcessor {
    fn process(&self, item: &Person) -> ItemProcessorResult<Person> {
        if item.age >= 18 {
            Ok(Some(item.clone()))  // garder les adultes
        } else {
            Ok(None)  // filtrer les mineurs
        }
    }
}
```

## Décisions clés

- **Pas de `FilteringProcessor` utilitaire** : les utilisateurs implémentent directement `process()` avec `Ok(None)`.
- **`filter_count` ne compte pas dans `process_count`** : `process_count` = items passés au writer uniquement.
- **`filter_count` ne déclenche pas le skip_limit** : le filtrage est intentionnel, pas une erreur.
- **Rétrocompatibilité** : tous les processors existants fonctionnent après avoir remplacé `Ok(x)` par `Ok(Some(x))`.
