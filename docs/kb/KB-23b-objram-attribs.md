# KB-23b — OBJRAM et attributs par colonne

> Source: driver MAME `src/mame/galaxian/` (galaxian_v.cpp, galaxian.h), **périmètre Galaxian d'origine** uniquement.
> Ferme une partie de GAP-01 (format VRAM/OBJRAM détaillé).

## Écriture OBJRAM : `galaxian_objram_w` (galaxian_v.cpp:514)

```c
void galaxian_state::galaxian_objram_w(offs_t offset, uint8_t data)
{
    // update any video up to the current scanline
    m_screen->update_partial(m_screen->vpos());

    // store the data
    m_spriteram[offset] = data;

    // the first $40 bytes affect the tilemap
    if (offset < 0x40)
    {
        // even entries control the scroll position
        if ((offset & 0x01) == 0)
        {
            m_bg_tilemap->set_scrolly(offset >> 1, data);
        }
        else
        {
            // odd entries control the color attribute for that column
            uint8_t x = offset >> 1;
            uint8_t attrib = (data & 7) | ((m_spriteram[x*2] & 0xf8));
            m_bg_tilemap->set_color_base(x, attrib);
        }
    }
}
```

### Structure de l'OBJRAM

L'OBJRAM fait **128 octets** (0x80) et est divisée en deux zones :

| Zone | Offset | Taille | Rôle |
|---|---:|---:|---|
| Tilemap attributs | `0x00`–`0x3F` | 64 o | Scroll + couleur par colonne (32 colonnes × 2 octets) |
| Sprites | `0x40`–`0x7F` | 64 o | Données sprites (8 sprites × 8 octets) |

### Zone tilemap (offsets 0x00–0x3F)

Pour chaque colonne x ∈ [0,31] :
- **octet pair** (`x*2`) : position de scroll vertical pour la colonne ;
- **octet impair** (`x*2+1`) : attribut de couleur (bits 0-2 = index palette).

L'écriture d'un octet impair met à jour l'attribut de couleur de la colonne correspondante.

## Lecture sprite-info : `sprites_draw` (galaxian_v.cpp:568)

```c
void galaxian_state::sprites_draw(screen_device &screen, bitmap_rgb32 &bitmap,
    const rectangle &cliprect, const uint8_t *spritebase)
{
    // ... clip handling ...
    
    for (int sprnum = 7; sprnum >= 0; sprnum--)
    {
        const uint8_t *base = &spritebase[sprnum * 4];

        uint8_t base0 = base[0];
        uint8_t sy = 240 - (base0 - (sprnum < 3));

        uint16_t code = base[1] & 0x3f;
        uint8_t flipx = base[1] & 0x40;
        uint8_t flipy = base[1] & 0x80;
        uint8_t color = base[2] & 7;

        const int hoffset = 1;
        uint8_t sx = base[3] + hoffset;

        // ... render sprite ...
    }
}
```

### Structure des données sprites (zone 0x40–0x7F)

Chaque sprite occupe **4 octets** dans l'OBJRAM :

| Octet | Bits | Signification |
|---:|---|---|
| `base[0]` | tous | Position Y (inversée : 240 - valeur) ; sprites 0-2 ont un décalage de -1 |
| `base[1]` | 0-5 | Code sprite (index dans ROM gfx1, bits 6-7 ignorés) |
| | 6 | Flip X (`0x40`) |
| | 7 | Flip Y (`0x80`) |
| `base[2]` | 0-2 | Index couleur palette (`& 7`) ; bits 3-7 ignorés |
| `base[3]` | tous | Position X (décalage +1 pour alignement sprite/tile) |

### Priorité de rendu

Les sprites sont rendus **à l'envers** (de 7 à 0) pour que les numéros plus bas aient la priorité :
```c
for (int sprnum = 7; sprnum >= 0; sprnum--)
```

## Déclaration OBJRAM (galaxian.h:421)

```c
required_shared_ptr<uint8_t> m_spriteram;
```

L'OBJRAM est un buffer partagé de 128 octets, déclaré dans le state de la machine.

## Résumé

| Élément | Valeur |
|---|---|
| OBJRAM size | 128 octets (0x80) |
| Zone tilemap | `0x00`–`0x3F` (64 o : scroll + couleur par colonne) |
| Zone sprites | `0x40`–`0x7F` (64 o : 8 sprites × 4 octets) |
| Scroll | Octet pair de chaque paire colonne → position Y |
| Couleur tilemap | Bits 0-2 de l'octet impair de chaque paire colonne |
| Sprite data | 4 octets/sprite : Y, code+flips, couleur, X |
| Priorité sprites | Rendu inversé (7→0) pour priorité aux numéros bas |
