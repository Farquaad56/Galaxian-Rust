# KB-04 — Registres d'écriture (1 bit par adresse, D0)

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

`/DRIVER` (6000-6007, A0-A2 = adresse du bit) :

| Adr | Fonction | | Adr | Fonction |
|---|---|---|---|---|
| 6000 | lampe 1P START | | 6004 | résistance 1M (555 @9R) |
| 6001 | lampe 2P START | | 6005 | 470k |
| 6002 | COIN LOCKOUT | | 6006 | 220k |
| 6003 | COIN COUNTER | | 6007 | 100k |

`/SOUND` (6800-6807) : 6800 FS1 · 6801 FS2 · 6802 FS3 · 6803 HIT · 6804 n/c · 6805 FIRE · 6806 VOL1 · 6807 VOL2.

`LATCH` (7000-77ff) : 7001 **NMI ON** · 7004 **STARS ON** · 7006 **HFLIP** · 7007 **VFLIP**. (7000, 7002, 7003, 7005 : non utilisés dans la doc fournie.)

Fonctions I/O :
```cpp
start_lamp_w(offset,d): m_lamps[offset] = d & 1;
coin_lock_w(d):  coin_lockout_global_w(~d & 1);
coin_count_0_w(d): coin_counter_w(0, d & 1);
```
