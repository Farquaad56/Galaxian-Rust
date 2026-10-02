# KB-26 — Rendu : ordre de composition (confirmé MAME)

> Source : driver MAME `src/mame/galaxian/` (`galaxian_v.cpp`), jeu **Galaxian** (set `galaxian`). Chaque affirmation cite un numéro de ligne exact.

## Ordre de composition effectif (tranché par le code)

```
fond noir/étoiles  →  tilemap  →  sprites (priorité ↑)  →  coquilles/missiles
```

Ceci est **confirmé** par la fonction `screen_update_galaxian` ([galaxian_v.cpp:460](../mame_src/galaxian_v.cpp#L460)) :

| # | Couche | Fonction / appel | Ligne MAME | Détail critique |
|---|--------|-----------------|-----------|-----------------|
| 1 | Fond + étoiles | `m_draw_background_ptr(bitmap, cliprect)` | [galaxian_v.cpp:463](../mame_src/galaxian_v.cpp#L463) | Pointeur fonction par défaut vers `galaxian_draw_background` ([galaxian.cpp:8833](../mame_src/galaxian.cpp#L8833)) |
| 1a | Fond noir | `bitmap.fill(rgb_t::black(), cliprect)` (dans `galaxian_draw_background`) | [galaxian_v.cpp:952-953](../mame_src/galaxian_v.cpp#L952) | Avant tout pixel coloré ; écrête les artefacts de frame précédente |
| 1b | Étoiles | `galaxian_draw_stars(bitmap, cliprect, maxx)` (dans `galaxian_draw_background`) | [galaxian_v.cpp:955](../mame_src/galaxian_v.cpp#L955) | Ligne 932-947 : boucle scanline par scanline ; appel à `stars_draw_row` qui écrit directement dans la bitmap via `bitmap.pix()` |
| 2 | Tilemap (fonds + ennemis) | `m_bg_tilemap->draw(screen, bitmap, cliprect, 0, 0)` | [galaxian_v.cpp:466](../mame_src/galaxian_v.cpp#L466) | Tuile par tuile ; transparent pén 0 (défini à [galaxian_v.cpp:414](../mame_src/galaxian_v.cpp#L414)) |
| 3 | Sprites (avant-train) | `sprites_draw(...)` dans la boucle [galaxian_v.cpp:469-470](../mame_src/galaxian_v.cpp#L469) | 8 sprites × générateur ; priorité par indice croissant |
| 3a | Priorité bas→haut | `for (int sprnum = 7; sprnum >= 0; sprnum--)` dans `sprites_draw` | [galaxian_v.cpp:577](../mame_src/galaxian_v.cpp#L577) | Les indices faibles (`sprnum=0`) écrasent en premier ; le dernier gagnant (plus prioritaire) s'écrit après les précédents — l'ordre de superposition final est bien « bas → haut » en priorité d'affichage |
| 4 | Coquilles / missile | `bullets_draw(screen, bitmap, cliprect, &m_spriteram[m_bullets_base])` | [galaxian_v.cpp:473-474](../mame_src/galaxian_v.cpp#L473) | Détection ligne par scanline ; entrées 0–2 (coquilles), entrée 7 (missile) — voir KB-23 |

> **Note GAP-01** : l'ordre « étoiles / tilemap / sprites / coquilles » est celui-là même dans lequel les appels interviennent dans `screen_update_galaxian`. L'hypothèse de la doc (KB-14, non produite) avait identifié le même enchaînement ; cette fiche le **confirme** par lecture directe.

## Détail des priorités sprites (`sprites_draw`)

`[galaxian_v.cpp:568-619](../mame_src/galaxian_v.cpp#L568)` :

- La boucle `for (int sprnum = 7; sprnum >= 0; sprnum--)` tire les entrées d'OBJRAM de l'indice 7 vers 0.
- Chaque sprite est écrit via `m_gfxdecode->gfx(1)->transpen(...)` ([galaxian_v.cpp:614](../mame_src/galaxian_v.cpp#L614)) avec transparence sur le canal alpha du bitmap cible (`bitmap` en mode ARGB).
- L'ordre **descendant** de l'indice (7 vers 0) signifie que les sprites d'indice faible sont tirés en dernier et donc au-dessus des précédents : **priorité « bas indice = avant plan »** est confirmée.

## Résumé du flux par scanline

1. `bitmap.fill(kill_black, ...)` — fond noir complet ;
2. `stars_draw_row` incruste les étoiles pixel par ligne ([galaxian_v.cpp:869-914](../mame_src/galaxian_v.cpp#L869)) ;
3. `tilemap->draw(...)` applique le fond animé (tuiles 8×8) avec transparence ;
4. Les 8 sprites sont composés, indice 7 d'abord ;
5. Les coquilles/missile sont tracés en dernier sur tout le reste.

## Vérification contre KB-14 (hypothèse dérivée de la doc)

La fiche KB-14, dite « doc-derived render-order hypothesis », énonçait :

> 1. fond noir ; 2. étoiles ; 3. tilemap ; 4. sprites (ordre of priorité : n° faible devant) ; 5. shells/missile. L'ordre exact of superposition étoiles/tilemap/sprites/shells est à valider contre MAME.

**Cette fiche confirme chaque affirmation de KB-14** par lecture directe de `screen_update_galaxian` ([galaxian_v.cpp:460](../mame_src/galaxian_v.cpp#L460)) et des deux fonctions de rendu appelées (`stars_draw_row`, `sprites_draw`, `bullets_draw`). Le seul point d'ajustement mineur : la priorité « faible/devant » est en réalité **« indice faible = dernier dessiné = au premier plan »** (donc l'inverse littéral de ce que le nom seul suggère) — à noter explicitement dans l'implémentation.

## Clôture GAP-01

GAP-01 exigeait « L'ordre exact of superposition étoiles/tilemap/sprites/shells est à valider contre MAME ». Ce point est clos : la source MAME confirme l'ordre et en précise la priorité sprites (indices faibles au premier plan).
