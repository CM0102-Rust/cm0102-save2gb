//! DLL só do save do CM 01/02 3.9.68: saves acima de 2 GB (até 4 GB), sem alterar o exe.
//!
//! Entra no jogo como `ddraw.dll` (o exe só importa `DirectDrawCreate`, e o Windows procura a DLL
//! primeiro na pasta do exe). Ao carregar, confere a assinatura do código de que depende
//! (`TRECHOS`, a mesma de `engenharia-reversa/assinatura_save.py`) e, se bater com o 3.9.68, desvia
//! as três rotinas do contêiner (`zopen` 0x9326e0, `zseek` 0x933080, `ztell` 0x933170) para as
//! versões em Rust de `rust/src/conteiner.rs`. Se não bater, não toca em nada. O `DirectDrawCreate`
//! é repassado a `ddraw_chain.dll` na pasta do jogo (outro wrapper de vídeo renomeado), se existir,
//! senão à `ddraw.dll` do Windows. O registro vai para `save2gb.log`, ao lado do exe.

#![allow(linker_messages)]

use core::ffi::c_void;

mod conteiner;

/// Os contadores de ramo do projeto: aqui não contam nada.
mod comparacao {
    pub fn ramo(_: &'static str) {}
}

mod registro {
    use std::io::Write;
    pub fn linha(texto: &str) {
        let caminho = std::env::current_exe().map(|p| p.with_file_name("save2gb.log")).unwrap_or_else(|_| "save2gb.log".into());
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(caminho) {
            let _ = writeln!(f, "{}", texto);
        }
    }
}

#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryW(nome: *const u16) -> *mut c_void;
    fn GetProcAddress(modulo: *mut c_void, nome: *const u8) -> *mut c_void;
    fn GetSystemDirectoryW(buf: *mut u16, tam: u32) -> u32;
    fn VirtualProtect(end: *mut c_void, tam: usize, nova: u32, antiga: *mut u32) -> i32;
    fn FlushInstructionCache(processo: *mut c_void, end: *const c_void, tam: usize) -> i32;
    fn GetCurrentProcess() -> *mut c_void;
}

/// O contêiner inteiro (zipdir.cpp) e o começo das funções da CRT e do jogo que as rotinas chamam.
const TRECHOS: [(usize, usize); 15] = [
    (0x9326e0, 0x1180), (0x944c94, 32), (0x944c9f, 32), (0x944cf1, 32), (0x944cff, 32), (0x945a09, 32),
    (0x945b00, 32), (0x945bfe, 32), (0x945ce6, 32), (0x946a22, 32), (0x94af9e, 32), (0x94fded, 32),
    (0x94c552, 32), (0x90d130, 32), (0x5e8290, 32),
];
/// FNV-1a de 64 bits dos trechos no 3.9.68 (cm0102.exe e as cópias com NickPatcher conferidas).
const ASSINATURA: u64 = 0x232589cfe29f6240;

unsafe fn assinatura() -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for (va, n) in TRECHOS {
        for i in 0..n {
            h = (h ^ *((va + i) as *const u8) as u64).wrapping_mul(0x100000001b3);
        }
    }
    h
}

/// Salto (E9) da entrada da rotina original para a versão em Rust; a original não é mais chamada.
unsafe fn desviar(de: usize, para: usize) {
    let mut antiga = 0u32;
    VirtualProtect(de as *mut c_void, 5, 0x40, &mut antiga);
    *(de as *mut u8) = 0xe9;
    core::ptr::write_unaligned((de + 1) as *mut i32, para.wrapping_sub(de + 5) as i32);
    VirtualProtect(de as *mut c_void, 5, antiga, &mut antiga);
    FlushInstructionCache(GetCurrentProcess(), de as *const c_void, 5);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(_modulo: *mut c_void, motivo: u32, _reservado: *mut c_void) -> i32 {
    if motivo == 1 {
        let a = assinatura();
        if a == ASSINATURA {
            desviar(0x9326e0, conteiner::abrir_no_conteiner as *const () as usize);
            desviar(0x933080, conteiner::buscar_no_conteiner as *const () as usize);
            desviar(0x933170, conteiner::posicao_no_conteiner as *const () as usize);
            registro::linha("save2gb 1.0: ligado (saves ate 4 GB)");
        } else {
            registro::linha(&format!("save2gb 1.0: exe diferente do 3.9.68 nos trechos do save (assinatura {:016x}); nada foi alterado", a));
        }
    }
    1
}

type FnDirectDrawCreate = unsafe extern "system" fn(*mut c_void, *mut *mut c_void, *mut c_void) -> i32;

/// A ddraw.dll seguinte: `ddraw_chain.dll` na pasta do exe, se existir; senão a do Windows (pelo
/// caminho completo: pelo nome, o Windows devolveria esta).
unsafe fn proxima_ddraw() -> Option<FnDirectDrawCreate> {
    let local = std::env::current_exe().ok().map(|p| p.with_file_name("ddraw_chain.dll")).filter(|p| p.exists());
    let caminho: Vec<u16> = match local {
        Some(p) => p.to_string_lossy().encode_utf16().chain([0]).collect(),
        None => {
            let mut buf = [0u16; 260];
            let n = GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) as usize;
            if n == 0 || n > 240 {
                return None;
            }
            buf[..n].iter().copied().chain("\\ddraw.dll\0".encode_utf16()).collect()
        }
    };
    let m = LoadLibraryW(caminho.as_ptr());
    if m.is_null() {
        return None;
    }
    let f = GetProcAddress(m, b"DirectDrawCreate\0".as_ptr());
    (!f.is_null()).then(|| core::mem::transmute::<*mut c_void, FnDirectDrawCreate>(f))
}

#[no_mangle]
pub unsafe extern "system" fn DirectDrawCreate(guid: *mut c_void, saida: *mut *mut c_void, externo: *mut c_void) -> i32 {
    match proxima_ddraw() {
        Some(f) => f(guid, saida, externo),
        None => {
            registro::linha("save2gb: nao consegui carregar a ddraw.dll seguinte");
            0x8000_4005_u32 as i32
        }
    }
}
