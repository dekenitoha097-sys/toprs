# toprs

Tableau de bord interactif pour surveiller un système Linux depuis le terminal.

## Lancer

```bash
cargo run -- --console
```

L'option courte `-c` est également disponible :

```bash
cargo run -- -c
```

Pour construire le binaire `toprs` :

```bash
cargo build --release
./target/release/toprs --console
```

## Commandes clavier

- `q` ou `Échap` : quitter
- `↑` / `↓` ou `k` / `j` : parcourir la liste des processus
- `r` : actualiser immédiatement les données

Les statistiques sont actualisées automatiquement toutes les deux secondes.
