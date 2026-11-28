# Versões do desktop

Cada versão é uma seção `## X.Y.Z · data`, escrita na `release/X.Y.Z` junto com o bump. O
`desktop-release.yml` publica essa seção como texto da release do GitHub (a página de onde se
baixam o EXE, o MSI e o MSIX); sem ela, a release não sai.

## Como montar o texto da release

Quem lê é quem vai instalar o app no Windows. Escreva para essa pessoa, não para quem programa.

1. **Destaques**: uma a três linhas com o que mais importa nesta versão.
2. Depois, só as seções que tiverem algo, nesta ordem:
   - `### Novidades`: o que a pessoa passa a conseguir fazer.
   - `### Correções`: o que estava errado e agora funciona.
   - `### Mudanças e melhorias`: o que ficou diferente ou mais rápido.
   - `### Atenção`: o que quebra ou pede ação (reinstalar, parear de novo, dado que muda).
   - `### Dependências`: só quando a pessoa sente (segurança, compatibilidade).
3. Cada item numa linha, começando pelo nome da coisa em negrito: `- **Tela de rede:** …`.
4. Fora do texto: CI, refatoração, teste, lint, docs internas e nomes de classe ou arquivo.

## 1.1.0 · 23 nov 2026

Visual novo no padrão da marca Acerola.

### Novidades

- **Visual da marca:** tema, telas, diálogos, formulários e uma tela de erro única.
- **Quatro estados** (carregando, vazio, erro e pronto) no histórico e no quadrinho.

### Mudanças e melhorias

- **Erros** sem texto técnico cru, e todos os textos fixos traduzíveis.
- **Tela de rede** em duas colunas, com o QR fixo.

### Atenção

- **Identificador do app** passa a ser `app.acerola.reader`; os dados migram sozinhos.
