# Versões do Android

Cada versão é uma seção `## X.Y.Z · data`, escrita na `release/X.Y.Z` junto com o bump. O
`android-release.yml` publica essa seção como texto da release do GitHub (a página de onde se
baixa o APK); sem ela, a release não sai.

## Como montar o texto da release

Quem lê é quem vai instalar o APK. Escreva para essa pessoa, não para quem programa.

1. **Destaques**: uma a três linhas com o que mais importa nesta versão.
2. Depois, só as seções que tiverem algo, nesta ordem:
   - `### Novidades`: o que a pessoa passa a conseguir fazer.
   - `### Correções`: o que estava errado e agora funciona.
   - `### Mudanças e melhorias`: o que ficou diferente ou mais rápido.
   - `### Atenção`: o que quebra ou pede ação (reinstalar, parear de novo, dado que muda).
   - `### Dependências`: só quando a pessoa sente (segurança, compatibilidade).
3. Cada item numa linha, começando pelo nome da coisa em negrito: `- **Tela de sync:** …`.
4. Fora do texto: CI, refatoração, teste, lint, docs internas e nomes de classe ou arquivo.

## 1.1.0 · 23 nov 2026

Visual novo no padrão da marca Acerola, igual ao desktop.

### Novidades

- **Visual da marca:** tema gerado da marca, fontes e mascote.
- **Tela de erro:** uma tela só para qualquer erro, no lugar do aviso de travamento do sistema.
- **Estados vazio e carregando** no histórico, nos padrões, nas transferências e no leitor.

### Mudanças e melhorias

- **Tela de sync** em duas colunas no modo paisagem.
