# KB-24a — Entrées IN0/IN1/IN2 (Galaxian d'origine)

> Source: driver MAME `src/mame/galaxian/` (galaxian.cpp), **périmètre Galaxian d'origine** uniquement.
> Ferme une partie de GAP-05 (ports IN0/IN1/IN2 + DIPs).

## Définition des ports : `INPUT_PORTS_START( galaxian )` (galaxian.cpp:3072)

```c
static INPUT_PORTS_START( galaxian )
    PORT_START("IN0")
        PORT_BIT( 0x01, IP_ACTIVE_HIGH, IPT_COIN1 )
        PORT_BIT( 0x02, IP_ACTIVE_HIGH, IPT_COIN2 )
        PORT_BIT( 0x04, IP_ACTIVE_HIGH, IPT_JOYSTICK_LEFT ) PORT_2WAY
        PORT_BIT( 0x08, IP_ACTIVE_HIGH, IPT_JOYSTICK_RIGHT ) PORT_2WAY
        PORT_BIT( 0x10, IP_ACTIVE_HIGH, IPT_BUTTON1 )
        PORT_DIPNAME( 0x20, 0x00, DEF_STR( Cabinet ) )
            PORT_DIPSETTING(    0x00, DEF_STR( Upright ) )
            PORT_DIPSETTING(    0x20, DEF_STR( Cocktail ) )
        PORT_SERVICE( 0x40, IP_ACTIVE_HIGH )
        PORT_BIT( 0x80, IP_ACTIVE_HIGH, IPT_SERVICE1 )

    PORT_START("IN1")
        PORT_BIT( 0x01, IP_ACTIVE_HIGH, IPT_START1 )
        PORT_BIT( 0x02, IP_ACTIVE_HIGH, IPT_START2 )
        PORT_BIT( 0x04, IP_ACTIVE_HIGH, IPT_JOYSTICK_LEFT ) PORT_2WAY PORT_COCKTAIL
        PORT_BIT( 0x08, IP_ACTIVE_HIGH, IPT_JOYSTICK_RIGHT ) PORT_2WAY PORT_COCKTAIL
        PORT_BIT( 0x10, IP_ACTIVE_HIGH, IPT_BUTTON1 ) PORT_COCKTAIL
        PORT_BIT( 0x20, IP_ACTIVE_HIGH, IPT_UNUSED )
        PORT_DIPNAME( 0xc0, 0x00, DEF_STR( Coinage ) )
            PORT_DIPSETTING(    0x40, DEF_STR( 2C_1C ) )
            PORT_DIPSETTING(    0x00, DEF_STR( 1C_1C ) )
            PORT_DIPSETTING(    0x80, DEF_STR( 1C_2C ) )
            PORT_DIPSETTING(    0xc0, DEF_STR( Free_Play ) )

    PORT_START("IN2")
        PORT_DIPNAME( 0x03, 0x00, DEF_STR( Bonus_Life ) )
            PORT_DIPSETTING(    0x00, "7000" )
            PORT_DIPSETTING(    0x01, "10000" )
            PORT_DIPSETTING(    0x02, "12000" )
            PORT_DIPSETTING(    0x03, "20000" )
        PORT_DIPNAME( 0x04, 0x04, DEF_STR( Lives ) )
            PORT_DIPSETTING(    0x00, "2" )
            PORT_DIPSETTING(    0x04, "3" )
        PORT_DIPUNUSED( 0x08, 0x00 )
        PORT_BIT( 0xf0, IP_ACTIVE_HIGH, IPT_UNUSED )
INPUT_PORTS_END
```

## Port IN0 (contrôles joueur + service)

| Bit | Valeur | Signal | Type |
|---:|---:|---|---|
| 0 | `0x01` | Coin 1 | Bouton monnaie |
| 1 | `0x02` | Coin 2 | Bouton monnaie |
| 2 | `0x04` | Joystick gauche | Analogique 2 directions |
| 3 | `0x08` | Joystick droit | Analogique 2 directions |
| 4 | `0x10` | Bouton tir | Bouton feu |
| 5 | `0x20` | Cabinet (DIP) | Upright/Cocktail |
| 6 | `0x40` | Service | Bouton service |
| 7 | `0x80` | Service 1 | Bouton service secondaire |

## Port IN1 (contrôles cocktail + coinage)

| Bit | Valeur | Signal | Type |
|---:|---:|---|---|
| 0 | `0x01` | Start 1 | Bouton start joueur 1 |
| 1 | `0x02` | Start 2 | Bouton start joueur 2 |
| 2 | `0x04` | Joystick gauche (cocktail) | Analogique 2 directions |
| 3 | `0x08` | Joystick droit (cocktail) | Analogique 2 directions |
| 4 | `0x10` | Bouton tir (cocktail) | Bouton feu |
| 5 | `0x20` | Non utilisé | — |
| 6-7 | `0xc0` | Coinage (DIP) | Configuration monnaie |

## Port IN2 (DIPs de jeu)

| Bits | Valeur | Signal | Type |
|---:|---:|---|---|
| 0-1 | `0x03` | Bonus Life (DIP) | Score bonus vie |
| 2 | `0x04` | Lives (DIP) | Nombre de vies initiales |
| 3 | `0x08` | Non utilisé (DIP) | — |
| 4-7 | `0xf0` | Non utilisés | — |

## DIPs détaillés

### Cabinet (IN0 bit 5, `0x20`)
- `0x00` : Upright (cabine verticale)
- `0x20` : Cocktail (table cocktail)

### Coinage (IN1 bits 6-7, `0xc0`)
- `0x40` : 2 coins / 1 credit
- `0x00` : 1 coin / 1 credit (défaut)
- `0x80` : 1 coin / 2 credits
- `0xc0` : Free Play

### Bonus Life (IN2 bits 0-1, `0x03`)
- `0x00` : 7000 points
- `0x01` : 10000 points
- `0x02` : 12000 points
- `0x03` : 20000 points

### Lives (IN2 bit 2, `0x04`)
- `0x00` : 2 vies
- `0x04` : 3 vies (défaut)

## Résumé

| Port | Rôle principal | Bits utilisés |
|---:|---|---:|
| IN0 | Contrôles joueur + service | 8/8 |
| IN1 | Contrôles cocktail + coinage | 7/8 |
| IN2 | DIPs de jeu (bonus, vies) | 4/8 |
