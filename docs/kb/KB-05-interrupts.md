# KB-05 — Interruptions

> Source : driver MAME `src/mame/galaxian/` (galaxian.cpp, galaxian_v.cpp, galaxian.h, galaxian_a.cpp, galaxian_a.h), périmètre **Galaxian d'origine** uniquement.

```cpp
void vblank_interrupt_w(int state) {            // appelée sur changement de VBLANK
    if (state && m_irq_enabled) m_maincpu->set_input_line(m_irq_line, ASSERT_LINE);
}
void irq_enable_w(uint8_t data) {               // écriture à 7001
    m_irq_enabled = data & 1;
    if (!m_irq_enabled) m_maincpu->set_input_line(m_irq_line, CLEAR_LINE);
}
```
- `m_irq_line = INPUT_LINE_NMI` par défaut (galaxian.h) → **NMI Z80**.
- Une bascule (6F) est maintenue en preset par « NMI ON » ; tant que le jeu n'a pas écrit 1 à 7001, rien ne se produit.
- Fréquence : 60,6 Hz, à l'entrée en VBLANK (début de la ligne 240 selon VBSTART — **à confirmer by trace MAME**, T2.3).
- Le Z80 prend the NMI on front : the émulateur doit modéliser the line as a level and detect the **front montant**.
