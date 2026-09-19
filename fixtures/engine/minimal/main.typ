// The fixture the Wave 0 engine tests compile.
//
// It is deliberately boring: an A5 page, the bundled text face, explicit page
// breaks so that the page count does not move when font metrics do, and one
// included file so that multi-file projects are exercised too.
//
// From Wave 1 a file like this is generated from the document model rather
// than written by hand; until then the engine takes Typst source directly.

#set page(
  width: 148mm,
  height: 210mm,
  margin: (x: 18mm, y: 20mm),
  numbering: "1",
)
#set text(font: "Libertinus Serif", size: 11pt, lang: "en")
#set par(justify: true)

#align(center + horizon)[
  #text(size: 24pt)[The Secret Garden of Mia]

  #v(6mm)

  #text(size: 12pt, style: "italic")[a fixture, not a book]
]

#pagebreak()

= Chapter One

Mia found the door behind the ivy on a Tuesday, which is the least likely day
for a door to appear behind anything. It was painted the green of old bottles,
and the handle had been worn smooth by hands that were not hers.

She did not open it that day. She went home, ate her soup, and thought about
_hinges_, and about how a thing can be both shut and waiting.

#include "content/chapter-two.typ"
