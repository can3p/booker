# {{title}}

This is the first section of your paper. Write it in ordinary Markdown: a
blank line starts a new paragraph, and a line starting with `#` is a heading.
The paper theme sets the text in Source Serif, the headings in Inter, and
lets each section follow the last without starting a new page.

## A second-level heading

Use `##` for sections and `###` for the parts inside them. Lists, quotes and
tables work as you would expect:

- a list item
- another, with *italic* and **bold** in it

> A quotation is set in from both sides, like this one.

| Measurement | Value |
|:------------|------:|
| Width       | 21 cm |
| Height      | 29.7 cm |

## Where to go next

Add a file to `content/` for each part of the paper and list it in
`book.toml`, or keep everything in this one file. Run `booker build .` to
make the PDF, and `booker check .` to see anything Booker thinks is wrong.
