# Booker - best book authoring software for average cases

Booker is a desktop application that allows to author books and papers.
It's different from the LaTeX software since it should be more user friendly
and not specifically tailored towards science, it should be less complex
than InDesign, it should be much less clunky than Scribus.

The audience: amateurs that want to make a book (kids, fiction) without
fighting too much with the image layouts and endless settings.

## General requirements

* The format used by the software should be version control friendly. Ideal usecase
  is when a book can be saved in a github repository
* It should be possible to edit the objects with external tools if needed.
* It should not require duplicating text content. Users should be able to keep
  editing the text where they do it usually (we can put some requirements there)
  and the changes should sync naturally to the published version.

  As an example (not necesserily the way to go): User puts a book.md into the repo
  and uses it as a source for a text frame. If they need fancy styling for anything,
  they can always use HTML insterts
* It should be very straight forward to author a book with
  - Chapters
  - Headings
  - Table of contents
  - Images (replicated on all pages or unique to a page) with different overflow options
  - Different page formats, margins, font sizes
* From the instruments it should be possible to define styles for different text types
  - E.g. All headings will be bold, 18pt Arial with green background
* It should be possible to define styles like: the first letter of the text in the chapter
  should be styled differently or replaced with an image/ the first line should be styled differently.
* Pages should have inherited and overridable properties (color, background). This should
  be flexible, just imagine a user that wants to have all odd pages to have a certain
  styling. This should be supported without manual changes needed.
* It should be possible to define things like: every chapter starts from a new page, or
  this specific text block should be always on the new page
* it should be possible to anchor certain images to a piece of text. Imagine there is a
  text talking about flowers, it should be possible to add an image, align it to the top
  of the page and define something like (this photo should always be on the same, previous
  or next page relatively to the phrase)
* Free placement must stay available: a user should be able to put an image at an
  absolute position on a page and to set the position and size of text frames.
  Imagine a kids book where the text sits in a different place on every page, or a
  page with one big image on top and the text below. Flowing text is the default,
  not the only option.
* The output should be as simple as an HTML or ready to print PDF.

## Functional requirements

* This is a cross platform application
* Keep the application extensible, but ideally the complexity (if needed) should go into
  the user project rather then to the application.
