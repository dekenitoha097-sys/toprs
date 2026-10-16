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

## Serveur web

Le serveur sert les fichiers statiques du dossier `public/` sur `127.0.0.1:3000`.
La page d'accueil se trouve dans `public/index.html`.

```bash
cargo run -- --web
```

L'option courte `-w` est également disponible :

```bash
cargo run -- -w
```

Ouvre ensuite <http://127.0.0.1:3000> dans ton navigateur. Arrête le serveur
avec `Ctrl+C`.

## Commandes clavier

- `q` ou `Échap` : quitter
- `↑` / `↓` ou `k` / `j` : parcourir la liste des processus
- `r` : actualiser immédiatement les données

Les statistiques sont actualisées automatiquement toutes les deux secondes.
