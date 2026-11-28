# Acerola Reader — regras do projeto

Leitor de quadrinhos e mangás local-first: Android (Kotlin + Compose), desktop (Tauri + Svelte),
sincronização P2P entre os dispositivos (`lib/p2p`) e o portal de documentação (`docs/web`).

## Regra core: mensagem de tag 🔒

Vale para toda tag deste repositório, sem exceção e sem desvio (nem em "Desvios do padrão").

- Tag sempre anotada, com a mensagem **exatamente** `Versão X.Y.Z: <o que a versão traz, numa frase>`:
  `git tag -a v1.4.2 -m "Versão 1.4.2: cards clicáveis e o logo de cada app"`.
- `X.Y.Z` é o número da própria tag, sem prefixo: `docs/prod/v1.2.1` → `Versão 1.2.1: …`.
- Nada entre o número e os dois pontos: `Versão 1.2.1 das docs: …` está errado.
- Depois dos dois pontos, uma frase em português, para quem usa, em minúscula e sem ponto final.
- Nunca formato de commit (`[merge](local): …`), lista de commits, parágrafo ou corpo extra.
- Antes do push, esta conferência não pode listar nenhuma tag:
  `git for-each-ref refs/tags --format='%(refname:short)|%(contents:subject)' | grep -vE '\|Versão [0-9]+\.[0-9]+\.[0-9]+: '`
- Tag fora do padrão se corrige reescrevendo só a mensagem, no mesmo commit, e em todo projeto
  com o mesmo erro, não só no que você mexeu.

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
  `tokens.css` do acerola-brand; raio, sombra, fonte e movimento no `theme/tokens.css`; classes
  `.ac-*` no `theme/components.css` (cópia do brand book).
- **Android**: `AcerolaColors.kt` e `Shape.kt` são gerados do `tokens.json` da marca por
  `node scripts/design/android-theme.mjs` (nunca editar à mão); a mascote vira VectorDrawable com
  `node scripts/design/android-mascot.mjs <svg> <xml>`. Fontes (DM Sans e Titan One, SIL OFL 1.1)
  em `ui/src/main/res/font/`, licenças em `ui/src/main/assets/licenses/`.
- **Logo e mascote**: Android e desktop levam os arquivos da marca como asset local (o app é open
  source e não depende da CDN). Só o `docs/web` puxa da CDN da marca, e só o que é da marca.
- **Tela de erro**: web e desktop usam `AcerolaErrorScreen` no `+error.svelte`; no Android é
  `Acerola.Component.ErrorScreen`, e a `ErrorActivity` (processo `:error`) recebe qualquer crash no
  lugar do diálogo do sistema (`ErrorActivity.installCrashHandler` no `AcerolaApplication`).
- Imports do front por `#lib/…` com extensão (`package.json` `imports`); funções puras em
  `lib/utils/<nome>.util.ts`.
- Cada app tem o seu guia: `acerola/desktop/CONTRIBUTING.md`, `lib/p2p/CONTRIBUTING.md`.

## Desvios do padrão

- **Tags por app e canal**: `<app>/<canal>/vX.Y.Z`, com app `android`, `desktop`, `docs` ou
  `p2p` e canal `alpha`, `beta` ou `prod` (ex.: `desktop/prod/v1.0.2`), em vez de uma `vX.Y.Z` por
  repositório: cada app tem a sua versão e o seu canal. A tag continua sendo a única coisa que
  publica e precisa estar na `main`.
- **Versão no PR da release, por app**: `VERSION=X.Y.Z cargo make bump-<app>` na
  `release/<app>-X.Y.Z` (`bump-android` também sobe o `versionCode` em 1). Nenhum workflow faz
  commit: a `main` exige PR com aprovação e o GitHub não aceita o bot do Actions no bypass. Cada
  release confere a tag `<app>/<canal>/vX.Y.Z` contra os manifestos do app e cria a release do
  GitHub com a mensagem da tag. As docs vão ao ar no merge na `main` (webhook da Procedure).
- **Pastas**: os apps moram em `acerola/<app>/`; o portal fica em `docs/web/` com local de commit
  `web`; o front do desktop fica em `acerola/desktop/svelte/` (o `src-tauri/` é a outra metade);
  `monorepo` é o local da configuração da raiz e de `scripts/`, para manter o histórico.
- **Chaves do Android**: nome de recurso do `strings.xml` não aninha, então a chave é em inglês e
  em snake_case (`action_home_select_folder`), a convenção do Android. O idioma base é o
  `values/` (pt-br) e o inglês fica em `values-en/`, com as mesmas chaves.
- **Estrutura do Android**: o app é multi-módulo (`app`, `ui`, `core`, `data`, `infra`,
  `native`), não um `app/` só. O tema mora em `ui/…/common/ux/theme/` (`AcerolaColors.kt`, `Shape.kt`,
  `Type.kt`), e as telas em `ui/…/module/<área>/`.
