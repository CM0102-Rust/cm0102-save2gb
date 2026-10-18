//! O contêiner do jogo (`si/code/zipdir.cpp`): o formato dos `.sav` e `.qst`. Rotinas da 158ª leva
//! do projeto CM 01/02 em Rust (o save acima de 2 GB). O mesmo arquivo entra na `ddraw.dll` do
//! projeto, por isso não depende de nada além de `crate::comparacao::ramo` e `crate::registro::linha`.

use core::ffi::c_void;

#[link(name = "kernel32")]
extern "system" {
    fn SetFilePointer(h: *mut c_void, baixo: i32, alto: *mut i32, metodo: u32) -> u32;
    fn GetLastError() -> u32;
}

/// A caixa de erro do jogo para o arquivo e a linha (a mesma de `motor::aviso_em`).
unsafe fn aviso_em(arquivo: usize, linha: u32) {
    let formatar: unsafe extern "C" fn(usize, usize, ...) -> i32 = core::mem::transmute(0x944c9fusize);
    let separar: unsafe extern "C" fn(usize, usize, usize, *mut u8, *mut u8) = core::mem::transmute(0x944cffusize);
    let juntar: unsafe extern "C" fn(*mut u8, *mut u8, u32) -> usize = core::mem::transmute(0x90d130usize);
    let caixa: unsafe extern "C" fn(usize, usize, u32) = core::mem::transmute(0x5e8290usize);
    let (mut nome, mut ext) = ([0u8; 256], [0u8; 256]);
    separar(arquivo, 0, 0, nome.as_mut_ptr(), ext.as_mut_ptr());
    let s = juntar(nome.as_mut_ptr(), ext.as_mut_ptr(), linha);
    formatar(0xae24d0, 0x9870e8, s);
    caixa(0x9870e0, 0xae24d0, 0);
    *(0xb67a34 as *mut u32) = 0;
}

/// O `fseek(f, pos, SEEK_SET)` da CRT do jogo (0x946a22) com a posicao em 64 bits: as mesmas marcas
/// no `FILE` (tira o fim de arquivo, esvazia o buffer pela 0x94af9e, acerta leitura/escrita e o
/// tamanho do buffer) e o `_lseek` (0x94c552) por `SetFilePointer` com a parte alta, limpando o
/// fim de arquivo na tabela de arquivos ([0xde42e0], 8 bytes por arquivo). 0 ou -1, como a original.
unsafe fn buscar_64(f: usize, pos: u64) -> i32 {
    let esvaziar: unsafe extern "C" fn(usize) -> i32 = core::mem::transmute(0x94af9eusize);
    let manejador: unsafe extern "C" fn(i32) -> isize = core::mem::transmute(0x94fdedusize);
    let marcas = (f + 0xc) as *mut u32;
    if *marcas & 0x83 == 0 {
        *(0xde2f8c as *mut u32) = 0x16;
        return -1;
    }
    *marcas &= !0x10;
    esvaziar(f);
    if *marcas & 0x80 != 0 {
        *marcas &= !3;
    } else if *marcas & 1 != 0 && *marcas & 8 != 0 && *marcas & 0x400 == 0 {
        *((f + 0x18) as *mut u32) = 0x200;
    }
    let fd = *((f + 0x10) as *const i32);
    if fd < 0 || fd as u32 >= *(0xde43e0 as *const u32) {
        *(0xde2f8c as *mut u32) = 9;
        return -1;
    }
    let info = *((0xde42e0 + 4 * (fd as usize >> 5)) as *const usize) + 8 * (fd as usize & 0x1f);
    let h = manejador(fd);
    if h == -1 {
        *(0xde2f8c as *mut u32) = 9;
        return -1;
    }
    let mut alto = (pos >> 32) as i32;
    let baixo = SetFilePointer(h as *mut c_void, pos as u32 as i32, &mut alto, 0);
    if baixo == u32::MAX && GetLastError() != 0 {
        return -1;
    }
    *((info + 4) as *mut u8) &= !2;
    0
}

/// O `ftell` da CRT do jogo (0x946aae) em 64 bits, para arquivos binários (os do contêiner): a
/// posição do `_lseek` menos o que ainda está no buffer de leitura (`_cnt`, +4) ou mais o que está
/// no de escrita (`_ptr` − `_base`, +0/+8). A original devolve -1 acima de 2 GB.
unsafe fn posicao_64(f: usize) -> u64 {
    let manejador: unsafe extern "C" fn(i32) -> isize = core::mem::transmute(0x94fdedusize);
    let cnt = (f + 4) as *mut i32;
    if *cnt < 0 {
        *cnt = 0;
    }
    let fd = *((f + 0x10) as *const i32) as usize;
    let h = manejador(fd as i32);
    let mut alto = 0i32;
    let baixo = SetFilePointer(h as *mut c_void, 0, &mut alto, 1);
    // como o _lseek da original: limpa o fim de arquivo na tabela de arquivos
    *((*((0xde42e0 + 4 * (fd >> 5)) as *const usize) + 8 * (fd & 0x1f) + 4) as *mut u8) &= !2;
    let pos = ((alto as u32 as u64) << 32) | baixo as u64;
    let marcas = *((f + 0xc) as *const u32);
    if marcas & 1 != 0 || marcas & 0x108 == 0 {
        pos.wrapping_sub(*cnt as u64)
    } else {
        pos + (*(f as *const usize) - *((f + 8) as *const usize)) as u64
    }
}

/// { 0x9326e0 } (cdecl, si/code/zipdir.cpp): `zopen(caminho, nome, modo, a4, tipo)` abre o item `nome`
/// do contêiner do jogo (o formato do `.sav`, do banco e dos históricos): 12 bytes de cabeçalho em
/// +0x19 (tipo 3/4, a4, quantos; os antigos 1/2 viram 3/4 e trazem 250 itens), o índice de 1000 itens
/// de 0x10c bytes em +0x25 (posição, tamanho, nome) e os buffers de leitura (+0x41705) e escrita
/// (+0x72445). Com `modo` 1 abre para gravar ("r+b", ou cria com "w+b" e o cabeçalho), senão para ler
/// ("rb"). Lendo, acha o item e vai à posição dele; gravando, acrescenta o item no fim (no máximo 999).
/// Diferença da original, de propósito (o save acima de 2 GB): a posição do item é lida e gravada
/// como 32 bits sem sinal e o arquivo é percorrido em 64 bits; a original usa `fseek`/`ftell` com
/// sinal (acima de 2 GB a posição vira -1 ao gravar e a busca falha ao ler: `comp_stats.cpp 6000/6007`).
/// Acima de 4 GB o índice não cabe: o item é recusado (como um contêiner cheio) e o save falha sem
/// estragar o arquivo.
pub unsafe extern "C" fn abrir_no_conteiner(caminho: usize, nome: *const u8, modo: u32, a4: u32, tipo: u32) -> usize {
    let novo: unsafe extern "C" fn(u32) -> usize = core::mem::transmute(0x944cf1usize);
    let liberar: unsafe extern "C" fn(usize) = core::mem::transmute(0x944c94usize);
    let abrir: unsafe extern "C" fn(usize, usize) -> usize = core::mem::transmute(0x945b00usize);
    let fechar: unsafe extern "C" fn(usize) -> i32 = core::mem::transmute(0x945a09usize);
    let ler: unsafe extern "C" fn(usize, u32, u32, usize) -> u32 = core::mem::transmute(0x945bfeusize);
    let escrever: unsafe extern "C" fn(usize, u32, u32, usize) -> u32 = core::mem::transmute(0x945ce6usize);
    let buscar: unsafe extern "C" fn(usize, i32, i32) -> i32 = core::mem::transmute(0x946a22usize);
    let formatar: unsafe extern "C" fn(usize, usize, ...) -> i32 = core::mem::transmute(0x944c9fusize);
    let u = |p: usize, o: usize| core::ptr::read_unaligned((p + o) as *const u32);
    let por = |p: usize, o: usize, v: u32| core::ptr::write_unaligned((p + o) as *mut u32, v);
    let item = |z: usize, i: u32| z + 0x25 + 0x10c * i as usize;
    if a4 == 0x14 {
        return 0;
    }
    let z = novo(0xa3191);
    if z == 0 {
        aviso_em(0xad6ab0, 0x52);
        return 0;
    }
    por(z, 0xa318d, 0);
    por(z, 0xa3189, 0);
    por(z, 0xa3185, 0);
    por(z, 4, modo);
    let ler_indice = |z: usize, f: usize| {
        if ler(z + 0x19, 0xc, 1, f) != 0 {
            let mut antigo = false;
            if u(z, 0x19) == 1 {
                por(z, 0x19, 3);
                antigo = true;
            }
            if u(z, 0x19) == 2 {
                por(z, 0x19, 4);
                antigo = true;
            }
            ler(z + 0x25, 0x10c, if antigo { 0xfa } else { 0x3e8 }, f);
        }
    };
    let f;
    if modo == 1 {
        let g = abrir(caminho, 0x9c7848);
        por(z, 0, g as u32);
        if g == 0 {
            let g = abrir(caminho, 0xad6aac);
            por(z, 0, g as u32);
            if g == 0 {
                liberar(z);
                return 0;
            }
            por(z, 0x19, tipo);
            por(z, 0x1d, a4);
            por(z, 0x21, 0);
            crate::comparacao::ramo("conteiner criado");
            escrever(z + 0x19, 0x416ec, 1, g);
            f = g;
        } else {
            ler_indice(z, g);
            f = g;
        }
    } else {
        let g = abrir(caminho, 0x98f928);
        por(z, 0, g as u32);
        if g == 0 {
            liberar(z);
            return 0;
        }
        ler_indice(z, g);
        f = g;
    }
    let desistir = |z: usize| {
        fechar(u(z, 0) as usize);
        por(z, 0, 0);
        liberar(z);
        0usize
    };
    let t = u(z, 0x19);
    if t != 3 && t != 4 {
        return desistir(z);
    }
    // o item com esse nome
    let quantos = u(z, 0x21) as i32;
    let mut achado = None;
    let mut i = 0i32;
    while i < quantos {
        let a = core::ffi::CStr::from_ptr((item(z, i as u32) + 8) as *const core::ffi::c_char);
        if a.to_bytes() == core::ffi::CStr::from_ptr(nome as *const core::ffi::c_char).to_bytes() {
            achado = Some(i as u32);
            break;
        }
        i += 1;
    }
    por(z, 0x14, 0);
    por(z, 0x10, 0);
    if modo == 0 {
        match achado {
            Some(k) => {
                crate::comparacao::ramo("conteiner lido");
                por(z, 0xc, k);
                // a posição sem sinal: o item pode estar entre 2 e 4 GB
                buscar_64(f, u(item(z, k), 0) as u64);
                por(z, 8, 0);
                z
            }
            None => {
                formatar(0xde1f64, 0xad6a90, nome);
                desistir(z)
            }
        }
    } else {
        if achado.is_some() {
            formatar(0xde1f64, 0xad6a70, nome);
            return desistir(z);
        }
        if quantos >= 0x3e7 {
            return desistir(z);
        }
        crate::comparacao::ramo("conteiner gravado");
        let k = quantos as u32;
        por(z, 0xc, k);
        por(z, 0x21, k + 1);
        let n = core::ffi::CStr::from_ptr(nome as *const core::ffi::c_char).to_bytes_with_nul();
        core::ptr::copy_nonoverlapping(n.as_ptr(), (item(z, k) + 8) as *mut u8, n.len());
        buscar(f, 0, 2);
        let pos = posicao_64(f);
        if pos > u32::MAX as u64 {
            // ponytail: acima de 4 GB o indice de 32 bits nao cabe; formato estendido numa 2a fase
            crate::registro::linha(&format!("conteiner: item alem de 4 GB recusado ({} bytes)", pos));
            por(z, 0x21, k);
            return desistir(z);
        }
        por(item(z, k), 0, pos as u32);
        por(item(z, k), 4, 0);
        por(z, 8, 0);
        z
    }
}

/// { 0x933080 } (cdecl, zipdir.cpp): `zseek(z, desloc, origem)` no item aberto para leitura
/// (origem 0 = começo do item, 1 = posição atual, 2 = fim; outra = posição 0 do arquivo). Fora do
/// item, ou aberto para gravar, devolve -1. No tipo 4 (comprimido) volta ao começo do item e lê
/// até o ponto; no tipo 3 vai direto com `fseek`. Diferença de propósito (o save acima de 2 GB):
/// posições sem sinal e busca em 64 bits.
pub unsafe extern "C" fn buscar_no_conteiner(z: usize, desloc: i32, origem: u32) -> i32 {
    let ler: unsafe extern "C" fn(*mut u8, u32, u32, usize) -> u32 = core::mem::transmute(0x932c30usize);
    let u = |o: usize| core::ptr::read_unaligned((z + o) as *const u32);
    if z == 0 || u(4) != 0 {
        return -1;
    }
    let item = 0x25 + 0x10c * u(0xc) as usize;
    let (pos, tam) = (u(item), u(item + 4));
    let alvo = match origem {
        0 => pos.wrapping_add(desloc as u32),
        1 => (posicao_64(u(0) as usize) as u32).wrapping_add(desloc as u32),
        2 => pos.wrapping_add(tam).wrapping_add(desloc as u32),
        _ => 0,
    };
    if alvo >= pos.wrapping_add(tam) || alvo < pos {
        return -1;
    }
    core::ptr::write_unaligned((z + 8) as *mut u32, alvo - pos);
    if u(0x19) != 4 {
        crate::comparacao::ramo("conteiner busca direta");
        return buscar_64(u(0) as usize, alvo as u64);
    }
    crate::comparacao::ramo("conteiner busca relendo");
    buscar_64(u(0) as usize, pos as u64);
    core::ptr::write_unaligned((z + 0x14) as *mut u32, 0);
    core::ptr::write_unaligned((z + 0x10) as *mut u32, 0);
    // como a original: relê o z+8 a cada byte lido
    let mut b = 0u8;
    let mut i = 0i32;
    while i < u(8) as i32 {
        ler(&mut b, 1, 1, z);
        i += 1;
    }
    0
}

/// { 0x933170 } (cdecl, zipdir.cpp): `ztell(z)`. No tipo 4 devolve a posição do item mais o
/// deslocamento (z+8); no tipo 3, o `ftell` menos a posição do item (em 64 bits, aqui).
pub unsafe extern "C" fn posicao_no_conteiner(z: usize) -> i32 {
    if z == 0 {
        return -1;
    }
    let u = |o: usize| core::ptr::read_unaligned((z + o) as *const u32);
    let pos = u(0x25 + 0x10c * u(0xc) as usize);
    crate::comparacao::ramo("conteiner posicao");
    if u(0x19) == 4 {
        return pos.wrapping_add(u(8)) as i32;
    }
    (posicao_64(u(0) as usize) as u32).wrapping_sub(pos) as i32
}
