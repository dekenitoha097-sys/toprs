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
La page d'accueil se trouve dans `public/index.html`, avec ses styles dans
`public/styles.css` et son comportement dans `public/app.js`. Le tableau de bord
occupe la fenêtre sans défilement global ; la liste des processus défile
indépendamment.

```bash
cargo run -- --web
```

L'option courte `-w` est également disponible :

```bash
cargo run -- -w
```

Ouvre ensuite <http://127.0.0.1:3000> dans ton navigateur. Le tableau de bord
interroge `/api/system` et actualise les données toutes les deux secondes.
Par défaut, le serveur écoute uniquement sur la boucle locale. L'adresse peut
être changée avec `TOPRS_WEB_ADDR`, par exemple :

```bash
TOPRS_WEB_ADDR=127.0.0.1:3100 cargo run -- --web
```

Arrête le serveur avec `Ctrl+C`.

## Commandes clavier

- `q` ou `Échap` : quitter
- `↑` / `↓` ou `k` / `j` : parcourir la liste des processus
- `r` : actualiser immédiatement les données

Les statistiques sont actualisées automatiquement toutes les deux secondes.
