r"""Filler to test saves above 2 GB without playing for years (CM 01/02 3.9.68).

The save takes along every *.tmp file in the game folder, in name order. A large
"0000_enchimento.tmp" goes in first and pushes the career's real parts beyond 2 GB.

Usage:  python enchimento.py <game folder> [size in MB, default 2200]   (creates the file)
        python enchimento.py <game folder> --apagar                     (deletes it)
Create it with the career already loaded, right before saving (the game deletes the
.tmp files when it loads a save and when it creates a career). Delete it afterwards.
Needs about twice the size in free disk space (the filler plus the save).
"""
import os
import shutil
import sys

NOME = '0000_enchimento.tmp'


def main():
    pasta = sys.argv[1]
    alvo = os.path.join(pasta, NOME)
    if sys.argv[2:] == ['--apagar']:
        if os.path.exists(alvo):
            os.remove(alvo)
        print('deleted')
        return
    mb = int(sys.argv[2]) if len(sys.argv) > 2 else 2200
    if shutil.disk_usage(pasta).free < (2 * mb + 4096) << 20:
        sys.exit('not enough free disk space')
    # a random block repeated: the save's RLE compression only merges runs of equal bytes
    bloco = os.urandom(16 << 20)
    with open(alvo, 'wb') as f:
        for _ in range(mb // 16):
            f.write(bloco)
    print(f'{alvo}: {os.path.getsize(alvo):,} bytes')


if __name__ == '__main__':
    main()
