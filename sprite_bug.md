# oui
on a `0x10 0x30 0x10 0xb8` comme sprite, `x=0x3a`, `y=0x1a`.

On va donc faire un déroulé étape par étape pour voir comment fix les bords de notre écran...

On veut écrire en 0x3a : (3a/8 = 7) <= fb_x <= 8 &rarr; on filtre les 7ème et 8ème bytes
> sauf que fb_x est modulo 8 et ne peut donc aller que jusqu'à 7 ? ~~Pour l'instant passons, on va voir si ça a un réel impact sur la suite...~~ &rarr; seulement la moitié des slots demandés sont sélectionnés : on ne dessine donc que la moitié du sprite :(une ligne sur 2 :(((

et (1a = 26) <= fb_y < 30 &rarr; on retrouve nos deux pixels de marge en bas, de ce côté tout semble aller bien...

- `x%8 = 2` : on va donc tout décaler de 2 (à priori) 
    &rarr; 0x10.shr(2) = 0x04 (jusque là ok)  
  et au niveau du second byte (inexistant) on n'écrit rien.
- 0x30.shr(2) = 0x0c et 0x30.shr(6) = 0 (but not on my fb)
- idem que le 1

&rarr; visiblement, quelque chose dans notre indexage (probablement le filtre) fait qu'on sélectionne une ligne de trop - qui se fait "consommer" - sur deux
