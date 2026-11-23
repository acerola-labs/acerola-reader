# Como contribuir

Este projeto segue o padrão Acerola, o mesmo de todos os repositórios Acerola. As regras completas
estão nas skills em `.claude/skills/` (`acerola-design`, `acerola-project`, `acerola-git`) e o que
só vale aqui está no [`CLAUDE.md`](CLAUDE.md).

O Acerola Reader é um monorepo: cada app tem a sua stack, os seus comandos e o seu guia.

| App | Local de commit | Guia |
| --- | --- | --- |
| Android (Kotlin + Compose) | `android` | sem guia próprio ainda: abra uma issue antes do PR |
| Desktop (Rust + Tauri + Svelte) | `desktop` | [acerola/desktop/CONTRIBUTING.md](acerola/desktop/CONTRIBUTING.md) |
| Portal web e documentação | `web` | sem guia próprio ainda: abra uma issue antes do PR |
| P2P, biblioteca compartilhada (Rust) | `p2p` | [lib/p2p/CONTRIBUTING.md](lib/p2p/CONTRIBUTING.md) |
| Relay | `relay` | sem guia próprio ainda: abra uma issue antes do PR |
| Relay lib, vendorizado do iroh (Rust) | `iroh` | sem guia próprio ainda: abra uma issue antes do PR |

## Preparar a máquina

1. Node 22 ou mais novo (os scripts do padrão são `.mjs`) e
   [lefthook](https://github.com/evilmartians/lefthook) (`scoop install lefthook`,
   `winget install evilmartians.lefthook` ou `npm i -g lefthook`).
2. Na raiz: `node scripts/git/setup-git.mjs`. Grava o Git-Flow, o template de commit e instala os hooks.
3. [`cargo-make`](https://github.com/sagiegurari/cargo-make) (`cargo install cargo-make`): cada
   crate Rust usa o seu `makefile.toml`, e o da raiz tem as tarefas do monorepo inteiro
   (`cargo make clean` apaga o `target/` de todos os crates de uma vez).
4. Cada app: siga o guia da tabela acima.

## Fluxo

- Branch `feature/<nome>` ou `bugfix/<nome>` a partir da `develop`. A `main` e a `develop` são travadas.
- Commit `[tipo](local): Mensagem`, um local por commit. Locais em `acerola.config.json`.
- PR para a `develop`, um app por PR (salvo mudança de fato compartilhada), com o template
  preenchido e prints no claro e no escuro quando mexer em tela.
- `lib/p2p` é dependência por caminho do Android e do desktop: uma mudança ali já vale para os
  dois no mesmo PR.

## Antes do PR

```bash
node scripts/design/check-design.mjs
node scripts/i18n/check-i18n.mjs
# lint, tipos e testes de cada app (veja o guia do app)
```

## Licença

Ao contribuir, você concorda que sua contribuição sai sob a [MPL-2.0](LICENSE).
