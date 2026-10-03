# KB-25a — Set de ROMs (Galaxian d'origine)

> Source: driver MAME `src/mame/galaxian/` (galaxian.cpp), **périmètre Galaxian d'origine** uniquement.
> Ferme une partie de GAP-03 (table ROMs).

## Définition des ROMs : `ROM_START( galaxian )` (galaxian.cpp:9760)

```c
ROM_START( galaxian )
    ROM_REGION( 0x4000, "maincpu", 0 )
        ROM_LOAD( "galmidw.u",    0x0000, 0x0800, CRC(745e2d61) SHA1(e65f74e35b1bfaccd407e168ea55678ae9b68edf) )
        ROM_LOAD( "galmidw.v",    0x0800, 0x0800, CRC(9c999a40) SHA1(02fdcd95d8511e64c0d2b007b874112d53e41045) )
        ROM_LOAD( "galmidw.w",    0x1000, 0x0800, CRC(b5894925) SHA1(0046b9ed697a34d088de1aead8bd7cbe526a2396) )
        ROM_LOAD( "galmidw.y",    0x1800, 0x0800, CRC(6b3ca10b) SHA1(18d8714e5ef52f63ba8888ecc5a25b17b3bf17d1) )
        ROM_LOAD( "7l",           0x2000, 0x0800, CRC(1b933207) SHA1(8b44b0f74420871454e27894d0f004859f9e59a9) )

    ROM_REGION( 0x1000, "gfx1", 0 )
        ROM_LOAD( "1h.bin",       0x0000, 0x0800, CRC(39fb43a4) SHA1(4755609bd974976f04855d51e08ec0d62ab4bc07) )
        ROM_LOAD( "1k.bin",       0x0800, 0x0800, CRC(7e3f56a2) SHA1(a9795d8b7388f404f3b0e2c6ce15d713a4c5bafa) )

    ROM_REGION( 0x0020, "proms", 0 )
        ROM_LOAD( "6l.bpr",       0x0000, 0x0020, CRC(c3ac9467) SHA1(f382ad5a34d282056c78a5ec00c30ec43772bae2) )
ROM_END
```

## Région maincpu (0x4000 = 16 KiB)

| Fichier | Offset | Taille | CRC | SHA1 |
|---:|---:|---:|---:|---:|
| `galmidw.u` | `0x0000` | 2 KiB | `745e2d61` | `e65f74e3...` |
| `galmidw.v` | `0x0800` | 2 KiB | `9c999a40` | `02fdcd95...` |
| `galmidw.w` | `0x1000` | 2 KiB | `b5894925` | `0046b9ed...` |
| `galmidw.y` | `0x1800` | 2 KiB | `6b3ca10b` | `18d8714e...` |
| `7l` | `0x2000` | 2 KiB | `1b933207` | `8b44b0f7...` |

## Région gfx1 (0x1000 = 4 KiB)

| Fichier | Offset | Taille | CRC | SHA1 |
|---:|---:|---:|---:|---:|
| `1h.bin` | `0x0000` | 2 KiB | `39fb43a4` | `4755609b...` |
| `1k.bin` | `0x0800` | 2 KiB | `7e3f56a2` | `a9795d8b...` |

## Région proms (0x0020 = 32 octets)

| Fichier | Offset | Taille | CRC | SHA1 |
|---:|---:|---:|---:|---:|
| `6l.bpr` | `0x0000` | 32 o | `c3ac9467` | `f382ad5a...` |

## Résumé

| Région | Taille | Fichiers | Rôle |
|---:|---:|---:|---|
| maincpu | 16 KiB | 5 ROMs Z80 | Code programme |
| gfx1 | 4 KiB | 2 ROMs gfx | Tuiles + sprites |
| proms | 32 o | 1 PROM | Palette couleurs |
