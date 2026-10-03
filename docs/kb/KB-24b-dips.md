# KB-24b — DIP switches (Galaxian d'origine)

> Source: driver MAME `src/mame/galaxian/` (galaxian.cpp), **périmètre Galaxian d'origine** uniquement.
> Ferme une partie de GAP-05 (DIPs).

## Définition des DIPs : dans `INPUT_PORTS_START( galaxian )` (galaxian.cpp:3072)

Les DIP switches sont définis dans les ports IN0, IN1 et IN2.

### Cabinet (IN0 bit 5, masque `0x20`)
- `0x00` : Upright (cabine verticale) — défaut
- `0x20` : Cocktail (table cocktail)

### Coinage (IN1 bits 6-7, masque `0xc0`)
- `0x40` : 2 coins / 1 credit
- `0x00` : 1 coin / 1 credit — défaut
- `0x80` : 1 coin / 2 credits
- `0xc0` : Free Play

### Bonus Life (IN2 bits 0-1, masque `0x03`)
- `0x00` : 7000 points — défaut
- `0x01` : 10000 points
- `0x02` : 12000 points
- `0x03` : 20000 points

### Lives (IN2 bit 2, masque `0x04`)
- `0x00` : 2 vies
- `0x04` : 3 vies — défaut

## Résumé des DIPs

| DIP | Port/Bits | Masque | Options | Défaut |
|---:|---|---:|---|---|
| Cabinet | IN0 bit 5 | `0x20` | Upright/Cocktail | Upright |
| Coinage | IN1 bits 6-7 | `0xc0` | 4 options monnaie | 1C/1C |
| Bonus Life | IN2 bits 0-1 | `0x03` | 4 scores bonus | 7000 |
| Lives | IN2 bit 2 | `0x04` | 2 ou 3 vies | 3 |
