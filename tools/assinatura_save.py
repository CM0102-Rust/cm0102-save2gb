"""Assinatura do código de que a DLL do save depende (save/src/lib.rs, TRECHOS e ASSINATURA).

A DLL só desvia as três rotinas do contêiner se estes trechos do exe carregado forem idênticos aos
do 3.9.68: o contêiner inteiro (zipdir.cpp) e o começo das funções da CRT e do jogo que ela chama.
Uso: python assinatura_save.py <exe> [<exe> ...]   (imprime a FNV-1a de 64 bits de cada exe)
"""
import sys

import pefile

TRECHOS = [(0x9326e0, 0x1180)] + [(a, 32) for a in (
    0x944c94, 0x944c9f, 0x944cf1, 0x944cff, 0x945a09, 0x945b00, 0x945bfe, 0x945ce6,
    0x946a22, 0x94af9e, 0x94fded, 0x94c552, 0x90d130, 0x5e8290)]


def assinatura(caminho):
    pe = pefile.PE(caminho, fast_load=True)
    h = 0xcbf29ce484222325
    for va, n in TRECHOS:
        for b in pe.get_data(va - 0x400000, n):
            h = ((h ^ b) * 0x100000001b3) & 0xffffffffffffffff
    return h


if __name__ == '__main__':
    for exe in sys.argv[1:]:
        print('%016x  %s' % (assinatura(exe), exe))
