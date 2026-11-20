# Acerola Reader — regras do projeto

Leitor de quadrinhos e mangás local-first: Android (Kotlin + Compose), desktop (Tauri + Svelte),
sincronização P2P entre os dispositivos (`lib/p2p`) e o portal de documentação (`docs/web`).

## Padrão Acerola — REGRA OBRIGATÓRIA

Antes de mexer, carregue a skill certa e siga-a:

| Vai mexer em | Skill |
|---|---|
| tela, componente, cor, fonte, ícone, peça de marca | `acerola-design` |
| nome, pasta, rota, banco, texto de tela, i18n, camada, licença | `acerola-project` |
| commit, branch, PR, issue, workflow, release | `acerola-git` |

- **Tudo que o usuário não vê é em inglês**: pastas, arquivos, rotas, identificadores, chaves,
  tabelas, colunas, modelos, logs e comentários de código.
- **i18n obrigatório**: nenhum texto de tela no código; chave em inglês e aninhada por assunto,
  sem `_`/`-` emendando contexto (`docs/web/messages/`, `acerola/desktop/svelte/messages/` e os
  `strings.xml` do Android).
- Commit `[tipo](local): Mensagem`; locais em `acerola.config.json` (`commit.scopes`).
- **Só três branches vivas: `main`, `develop` e a de trabalho.** Juntou, apaga (local e remota).
  Outra branch só fica se quem administra pedir de forma específica.
- `node scripts/design/check-design.mjs` e `node scripts/i18n/check-i18n.mjs` precisam passar
  (hooks do Claude, `pre-commit` do lefthook e CI). Exceção de design só com
  `design-allow: <motivo>` na linha.
- Licença MPL-2.0.

## Deste projeto

- `lib/p2p` é dependência por caminho do Android e do desktop: mudou ali, vale para os dois no
  mesmo PR.
- Cores em `theme/colors/catppuccin.css` (web e desktop), cópia exata das cores do
  `tokens.css` do acerola-brand; raio e sombra no `theme/tokens.css`. No Android, `Color.kt` e
  `Shape.kt` são gerados do `tokens.json` do acerola-brand.
- **Logo e mascote**: Android e desktop levam os arquivos da marca como asset local (o app é open
  source e não depende da CDN). Só o `docs/web` puxa da CDN da marca, e só o que é da marca.
- Cada app tem o seu guia: `acerola/desktop/CONTRIBUTING.md`, `lib/p2p/CONTRIBUTING.md`.

## Desvios do padrão

- **Tags por app e canal**: `<app>/<canal>/vX.Y.Z`, com app `android`, `desktop`, `docs` ou
  `p2p` e canal `alpha`, `beta` ou `prod` (ex.: `desktop/prod/v1.0.2`), em vez de uma `vX.Y.Z` por
  repositório: cada app tem a sua versão e o seu canal. A tag continua sendo a única coisa que
  publica e precisa estar na `main`.
- **Versão sai da tag**: o workflow de release lê a versão da tag e grava nos manifestos
  (`cargo make bump-<app>` no `makefile.toml`), com commit do bot na `main` e na `develop`. Por
  isso ele não confere a tag contra o manifesto.
- **Pastas**: os apps moram em `acerola/<app>/`; o portal fica em `docs/web/` com local de commit
  `web`; o front do desktop fica em `acerola/desktop/svelte/` (o `src-tauri/` é a outra metade);
  `monorepo` é o local da configuração da raiz e de `scripts/`, para manter o histórico.
- **Chaves do Android**: o nome de recurso do `strings.xml` não aninha; o nível vira `_`
  (`library_search_placeholder`), que é a convenção do Android. Cada `_` separa um nível, nunca
  emenda palavras de um mesmo nível (essas vão em camelCase).
