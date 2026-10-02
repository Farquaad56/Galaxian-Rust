# KB-24 — Entrées / DIP switches (set `galaxian`)

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp), périmètre **Galaxian d'origine** uniquement. Citations relatives à `docs/mame_src/`.

Tous les bits sont **active-high** (`IP_ACTIVE_HIGH`). Ports lus directement en memory map (pas de PPI8255 sur le set original) : IN0 @ 0x6000, IN1 @ 0x6800, IN2 @ 0x7000, chacun mirroré sur 0x07ff (galaxian.cpp:1756, 1761, 1763). Le PPI8255 n'existe que sur les Konami-derived sets (`konami_base` branche IN0/IN1/IN2 sur `m_ppi8255[0]`, galaxian.cpp:7529-7533) — « differs ».

## IN0 (galaxian.cpp:3072-3094)

| bit | fonction |
|---|---|
| 0x01 | COIN1 (galaxian.cpp:3075) |
| 0x02 | COIN2 (galaxian.cpp:3076) |
| 0x04 | JOYSTICK_LEFT, 2-way (galaxian.cpp:3077) |
| 0x08 | JOYSTICK_RIGHT, 2-way (galaxian.cpp:3078) |
| 0x10 | BUTTON1 = fire (galaxian.cpp:3079) |
| 0x20 | DIP **Cabinet** : 0=Upright, 0x20=Cocktail (galaxian.cpp:3080-3082) |
| 0x40 | SERVICE (`PORT_SERVICE`, galaxian.cpp:3083) |
| 0x80 | SERVICE1 (galaxian.cpp:3084) |

## IN1 (galaxian.cpp:3085-3096)

| bit | fonction |
|---|---|
| 0x01 | START1 (galaxian.cpp:3087) |
| 0x02 | START2 (galaxian.cpp:3088) |
| 0x04 | JOYSTICK_LEFT, 2-way, cocktail only (galaxian.cpp:3089) |
| 0x08 | JOYSTICK_RIGHT, 2-way, cocktail only (galaxian.cpp:3090) |
| 0x10 | BUTTON1 = fire, cocktail only (galaxian.cpp:3091) |
| 0x20 | UNUSED (galaxian.cpp:3092) |
| 0xc0 | DIP **Coinage** : 0x40=2C/1C, 0x00=1C/1C, 0x80=1C/2C, 0xc0=Free Play (galaxian.cpp:3093-3096) |

## IN2 — DIP switches (galaxian.cpp:3098-3108)

| bits | fonction |
|---|---|
| 0x03 | **Bonus Life** : 0=7000, 1=10000, 2=12000, 3=20000 (galaxian.cpp:3099-3103) |
| 0x04 | **Lives** : 0=2, 0x04=3 (default 0x04) (galaxian.cpp:3104-3106) |
| 0x08 | DIPUNUSED (galaxian.cpp:3107) |
| 0xf0 | UNUSED (galaxian.cpp:3108) |

## Écrites associées (memory map, galaxian_map_base)

- Start lamp : write 0x6000–0x6001 (galaxian.cpp:1757) ; coin lockout : 0x6002 (galaxian.cpp:1758) ; coin count 0 : 0x6003 (galaxian.cpp:1759).
- IRQ enable : 0x7001 (galaxian.cpp:1764) ; stars enable : 0x7004 (galaxian.cpp:1765) ; flip screen X/Y : 0x7006/0x7007 (galaxian.cpp:1766-1767).

## « differs » (autres sets, non normatif)

Les clones/bootlegs incluent `PORT_INCLUDE(galaxian)` puis modifient IN1/IN2 : ex. `galaxianmo` bonus life 0/3000/4000/5000 (galaxian.cpp:3111-3120), `galaxrf` adds Player Bullet Speed on bit 0x08 (galaxian.cpp:3134-3149), `superg` lives 2/5 (galaxian.cpp:3151-3163).
