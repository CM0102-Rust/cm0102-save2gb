# save2gb: saves do CM 01/02 acima de 2 GB

*[English version](README.md)*

Correção para carreiras do **Championship Manager 01/02 (3.9.68)** cujo save passa de **2 GB**. Sem ela, o jogo não carrega mais a carreira: aparece uma caixa de erro (`comp_stats.cpp 6000/6007`, `Friendly.cpp` e outros) e o carregamento trava. Com a correção, os saves funcionam até **4 GB**.

- **Não altera o exe.** É uma `ddraw.dll` pequena que vai na pasta do jogo.
- **Não muda o formato do save.** Um save de até 2 GB gravado com a correção abre em qualquer jogo.
- **Funciona com exes modificados.** Foi testada na CLZ Leagues Patch (DB 25-26) V5.4, feita com o NickPatcher.
- **É segura em exes desconhecidos.** Antes de agir, confere o código de que depende. Se o seu exe for diferente nesse ponto, ela não altera nada.

## Download

Baixe o **`save2gb-1.0.zip`** na página de [Releases](../../releases).

## Instalar

1. Copie a `ddraw.dll` do zip para a pasta do jogo, junto do `cm0102.exe`.
2. Abra o jogo. O `save2gb.log`, na mesma pasta, diz:
   - `ligado`: a correção está ativa;
   - `exe diferente...`: o seu exe mudou o código do save, e a DLL não alterou nada.

**Remover:** apague a `ddraw.dll`.

**Já usa outro `ddraw.dll`** (dgVoodoo, cnc-ddraw...)? Renomeie o seu para `ddraw_chain.dll` e coloque este no lugar. O vídeo passa por ele.

O `cm0102_GDI.exe` não usa a `ddraw.dll`, então a correção não vale para ele.

## Como foi provado

| | Gravar acima de 2 GB | Carregar acima de 2 GB |
|---|---|---|
| 3.9.68 original | índice estragado: toda parte além de 2 GB fica com a posição `0xFFFFFFFF` | **erro, o carregamento trava** |
| Com o save2gb (3.9.68 original) | save de 2,55 GB, índice correto | **carrega normalmente** |
| CLZ 25-26 (NickPatcher), sem o save2gb | — | **erro** |
| CLZ 25-26 (NickPatcher), com o save2gb | save de 2,45 GB, índice correto | **carrega normalmente** |

## Para quem mantém patches

A causa, o formato do contêiner, as três rotinas envolvidas (`zopen` 0x9326e0, `zseek` 0x933080, `ztell` 0x933170) e três formas de integrar a correção no seu patch estão em [`docs/SAVE_2GB.md`](docs/SAVE_2GB.md). Em resumo: distribuir a DLL, carregá-la com `LoadLibrary` de dentro da sua DLL, ou reescrever as três rotinas.

Dúvidas ou ajuda para integrar: abra uma issue ou mande mensagem para **lucastmpacheco** no [fórum champman0102](https://champman0102.net).

Ferramentas em [`tools/`](tools):
- `assinatura_save.py` diz se a DLL vai agir num exe;
- `enchimento.py` cria o enchimento de 2,2 GB usado para testar um save acima de 2 GB sem jogar anos.

## Código-fonte

O fonte de cada release está neste repositório (`src/`, `Cargo.toml`): `src/conteiner.rs` traz as três rotinas reescritas, `src/lib.rs` a conferência da assinatura, os desvios e o repasse do `DirectDrawCreate`. O código e os comentários estão em português, como no resto do projeto. O repositório é atualizado a cada release, não a cada mudança.

Compilar (Windows, Rust com o alvo de 32 bits): `rustup target add i686-pc-windows-msvc` e depois `cargo build --release`. A DLL sai em `target/i686-pc-windows-msvc/release/ddraw.dll`. O hash difere do da release porque o compilador embute caminhos de pasta; o código é o mesmo.

## Sobre

Faz parte do projeto **CM 01/02 em Rust**, que reconstrói o jogo rotina por rotina. Cada rotina reescrita é provada idêntica à original antes de ser distribuída. Obrigado à comunidade do champman0102, ao davdav e a todos que mantêm o 01/02 vivo.
