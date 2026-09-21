# Third-party notices

**This file is generated. Do not edit it by hand** — run `cargo xtask third-party`, which reads the dependency tree and rewrites it. CI fails when it is out of date.

Booker itself is MIT licensed (see `LICENSE`). What a Booker build *contains* is listed below: the fonts it embeds and every Rust crate it is compiled from, with the licence each one carries. A crate listed under several licences is offered under any of them, at the user's choice, which is the usual Rust dual-licensing.

The layout engine is [Typst](https://typst.app), Apache-2.0, and appears below as the `typst-*` crates.

Source for any crate here is at <https://crates.io>, and for the MPL-2.0 crates specifically — whose licence asks that the source of those files be available — at the repository each crate names on its crates.io page. Booker uses them unmodified.

---

## Fonts

Booker embeds its fonts in the binary, so they travel with every build. Their notices, copied from `crates/booker-typst/fonts/NOTICE.txt`:

```
Fonts bundled with Booker
=========================

They are here so that a compile produces the same pages on every machine,
whether or not the machine has any fonts installed. All are unmodified.

From `typst-assets` 0.15.1 (https://github.com/typst/typst-assets), which is
where the Typst CLI takes its own defaults from:

* LibertinusSerif-{Regular,Italic,Bold,BoldItalic}.otf — SIL Open Font
  License 1.1. This is Typst's default text family, so a document that sets
  no font at all still renders.
* DejaVuSansMono.ttf — Bitstream Vera licence (see below). Typst's default
  family for `raw` (code) blocks.

From Google Fonts (https://github.com/google/fonts, directory `ofl/`), for
the four built-in themes; the files named `-Variable` are the variable-font
files published there as `<Family>[wght].ttf` or `<Family>[opsz,wght].ttf`,
renamed only because brackets make awkward paths:

* EBGaramond-Variable.ttf, EBGaramond-Italic-Variable.ttf — SIL Open Font
  License 1.1. Copyright 2017 The EB Garamond Project Authors
  (https://github.com/octaviopardo/EBGaramond12). The `novel` and `poetry`
  themes.
* SourceSerif4-Variable.ttf, SourceSerif4-Italic-Variable.ttf — SIL Open
  Font License 1.1. Copyright 2014 The Source Serif 4 Project Authors
  (https://github.com/adobe-fonts/source-serif). The `paper` theme's text.
* Inter-Variable.ttf — SIL Open Font License 1.1. Copyright 2020 The Inter
  Project Authors (https://github.com/rsms/inter). The `paper` theme's
  headings.
* Andika-{Regular,Italic,Bold,BoldItalic}.ttf — SIL Open Font License 1.1.
  Copyright (c) 2004-2022 SIL International (http://www.sil.org/), with
  Reserved Font Names "Andika" and "SIL". The `picture-book` theme: a face
  designed for people learning to read.

A project's own fonts, in `assets/fonts/`, are loaded on top of these and
win when a family name collides.

The full licence texts follow, as required by both licences. The SIL Open
Font License text below applies to every font above marked with it.

================================================================================
The SIL Open Font License Version 1.1 applies to:

* Libertinus Serif fonts in files/fonts/LibertinusSerif-*.otf
  Copyright © 2012-2024 The Libertinus Project Authors,
  with Reserved Font Name "Linux Libertine", "Biolinum", "STIX Fonts".

-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
redistributed and/or sold with any software, provided that each copy
contains the above copyright notice and this license. These can be
included either as stand-alone text files, human-readable headers or
in the appropriate machine-readable metadata fields within text or
binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
Name(s) unless explicit written permission is granted by the corresponding
Copyright Holder. This restriction only applies to the primary font name as
presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
Software shall not be used to promote, endorse or advertise any
Modified Version, except to acknowledge the contribution(s) of the
Copyright Holder(s) and the Author(s) or with their explicit written
permission.

5) The Font Software, modified or unmodified, in part or in whole,
must be distributed entirely under this license, and must not be
distributed under any other license. The requirement for fonts to
remain under this license does not apply to any document created
using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.
================================================================================

================================================================================
The terms below apply to:

* DejaVu fonts in files/fonts/DejaVu*.ttf
  (https://github.com/dejavu-fonts/dejavu-fonts)

Fonts are (c) Bitstream (see below). DejaVu changes are in public domain.
Glyphs imported from Arev fonts are (c) Tavmjong Bah (see below)


Bitstream Vera Fonts Copyright
------------------------------

Copyright (c) 2003 by Bitstream, Inc. All Rights Reserved. Bitstream Vera is
a trademark of Bitstream, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of the fonts accompanying this license ("Fonts") and associated
documentation files (the "Font Software"), to reproduce and distribute the
Font Software, including without limitation the rights to use, copy, merge,
publish, distribute, and/or sell copies of the Font Software, and to permit
persons to whom the Font Software is furnished to do so, subject to the
following conditions:

The above copyright and trademark notices and this permission notice shall
be included in all copies of one or more of the Font Software typefaces.

The Font Software may be modified, altered, or added to, and in particular
the designs of glyphs or characters in the Fonts may be modified and
additional glyphs or characters may be added to the Fonts, only if the fonts
are renamed to names not containing either the words "Bitstream" or the word
"Vera".

This License becomes null and void to the extent applicable to Fonts or Font
Software that has been modified and is distributed under the "Bitstream
Vera" names.

The Font Software may be sold as part of a larger software package but no
copy of one or more of the Font Software typefaces may be sold by itself.

THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF COPYRIGHT, PATENT,
TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL BITSTREAM OR THE GNOME
FOUNDATION BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, INCLUDING
ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL DAMAGES,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF
THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM OTHER DEALINGS IN THE
FONT SOFTWARE.

Except as contained in this notice, the names of Gnome, the Gnome
Foundation, and Bitstream Inc., shall not be used in advertising or
otherwise to promote the sale, use or other dealings in this Font Software
without prior written authorization from the Gnome Foundation or Bitstream
Inc., respectively. For further information, contact: fonts at gnome dot
org.

Arev Fonts Copyright
------------------------------

Copyright (c) 2006 by Tavmjong Bah. All Rights Reserved.

Permission is hereby granted, free of charge, to any person obtaining
a copy of the fonts accompanying this license ("Fonts") and
associated documentation files (the "Font Software"), to reproduce
and distribute the modifications to the Bitstream Vera Font Software,
including without limitation the rights to use, copy, merge, publish,
distribute, and/or sell copies of the Font Software, and to permit
persons to whom the Font Software is furnished to do so, subject to
the following conditions:

The above copyright and trademark notices and this permission notice
shall be included in all copies of one or more of the Font Software
typefaces.

The Font Software may be modified, altered, or added to, and in
particular the designs of glyphs or characters in the Fonts may be
modified and additional glyphs or characters may be added to the
Fonts, only if the fonts are renamed to names not containing either
the words "Tavmjong Bah" or the word "Arev".

This License becomes null and void to the extent applicable to Fonts
or Font Software that has been modified and is distributed under the
"Tavmjong Bah Arev" names.

The Font Software may be sold as part of a larger software package but
no copy of one or more of the Font Software typefaces may be sold by
itself.

THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL
TAVMJONG BAH BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.

Except as contained in this notice, the name of Tavmjong Bah shall not
be used in advertising or otherwise to promote the sale, use or other
dealings in this Font Software without prior written authorization
from Tavmjong Bah. For further information, contact: tavmjong @ free
. fr.

TeX Gyre DJV Math
-----------------
Fonts are (c) Bitstream (see below). DejaVu changes are in public domain.

Math extensions done by B. Jackowski, P. Strzelczyk and P. Pianowski
(on behalf of TeX users groups) are in public domain.

Letters imported from Euler Fraktur from AMSfonts are (c) American
Mathematical Society (see below).
Bitstream Vera Fonts Copyright
Copyright (c) 2003 by Bitstream, Inc. All Rights Reserved. Bitstream Vera
is a trademark of Bitstream, Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of the fonts accompanying this license (“Fonts”) and associated
documentation
files (the “Font Software”), to reproduce and distribute the Font Software,
including without limitation the rights to use, copy, merge, publish,
distribute,
and/or sell copies of the Font Software, and to permit persons  to whom
the Font Software is furnished to do so, subject to the following
conditions:

The above copyright and trademark notices and this permission notice
shall be
included in all copies of one or more of the Font Software typefaces.

The Font Software may be modified, altered, or added to, and in particular
the designs of glyphs or characters in the Fonts may be modified and
additional
glyphs or characters may be added to the Fonts, only if the fonts are
renamed
to names not containing either the words “Bitstream” or the word “Vera”.

This License becomes null and void to the extent applicable to Fonts or
Font Software
that has been modified and is distributed under the “Bitstream Vera”
names.

The Font Software may be sold as part of a larger software package but
no copy
of one or more of the Font Software typefaces may be sold by itself.

THE FONT SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS
OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF COPYRIGHT, PATENT,
TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL BITSTREAM OR THE GNOME
FOUNDATION
BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, INCLUDING ANY GENERAL,
SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL DAMAGES, WHETHER IN AN
ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF THE USE OR
INABILITY TO USE
THE FONT SOFTWARE OR FROM OTHER DEALINGS IN THE FONT SOFTWARE.
Except as contained in this notice, the names of GNOME, the GNOME
Foundation,
and Bitstream Inc., shall not be used in advertising or otherwise to promote
the sale, use or other dealings in this Font Software without prior written
authorization from the GNOME Foundation or Bitstream Inc., respectively.
For further information, contact: fonts at gnome dot org.

AMSFonts (v. 2.2) copyright

The PostScript Type 1 implementation of the AMSFonts produced by and
previously distributed by Blue Sky Research and Y&Y, Inc. are now freely
available for general use. This has been accomplished through the
cooperation
of a consortium of scientific publishers with Blue Sky Research and Y&Y.
Members of this consortium include:

Elsevier Science IBM Corporation Society for Industrial and Applied
Mathematics (SIAM) Springer-Verlag American Mathematical Society (AMS)

In order to assure the authenticity of these fonts, copyright will be
held by
the American Mathematical Society. This is not meant to restrict in any way
the legitimate use of the fonts, such as (but not limited to) electronic
distribution of documents containing these fonts, inclusion of these fonts
into other public domain or commercial font collections or computer
applications, use of the outline data to create derivative fonts and/or
faces, etc. However, the AMS does require that the AMS copyright notice be
removed from any derivative versions of the fonts which have been altered in
any way. In addition, to ensure the fidelity of TeX documents using Computer
Modern fonts, Professor Donald Knuth, creator of the Computer Modern faces,
has requested that any alterations which yield different font metrics be
given a different name.

$Id$
================================================================================
```

## Rust crates

Every crate compiled into a Booker build, grouped by licence. Generated from the dependency tree, so it describes this build rather than a build somebody documented once.

1028 crates under 14 licences.

### 0BSD

- adler2 2.0.1
- roman-numerals-rs 3.1.0

### Apache-2.0

- adler2 2.0.1
- anstream 1.0.0
- anstyle 1.0.14
- anstyle-parse 1.0.0
- anstyle-query 1.1.5
- anstyle-wincon 3.0.11
- anyhow 1.0.104
- approx 0.5.1
- arrayvec 0.7.8
- atomic-waker 1.1.2
- autocfg 1.5.1
- az 1.3.0
- base64 0.21.7
- base64 0.22.1
- base64 0.23.1
- biblatex 0.12.0
- bit-set 0.8.0
- bit-vec 0.8.0
- bitflags 1.3.2
- bitflags 2.13.2
- block-buffer 0.10.4
- bumpalo 3.20.3
- by_address 1.2.1
- bytemuck 1.25.2
- bytemuck_derive 1.12.1
- camino 1.2.6
- cargo-platform 0.1.9
- cargo_toml 0.22.3
- cc 1.4.7
- cfg-expr 0.15.8
- cfg-if 1.0.5
- chrono 0.4.45
- ciborium 0.2.2
- ciborium-io 0.2.2
- ciborium-ll 0.2.2
- citationberg 0.7.0
- clap 4.6.7
- clap_builder 4.6.7
- clap_derive 4.6.7
- clap_lex 1.1.1
- cobs 0.3.0
- codex 0.3.0
- color 0.3.3
- colorchoice 1.0.5
- comemo 0.5.1
- comemo-macros 0.5.1
- cookie 0.18.2
- core-foundation 0.10.1
- core-foundation 0.9.4
- core-foundation-sys 0.8.7
- core-graphics 0.25.0
- core-graphics-types 0.2.0
- cpufeatures 0.2.17
- crc32fast 1.5.2
- crossbeam-channel 0.5.17
- crossbeam-deque 0.8.8
- crossbeam-epoch 0.9.21
- crossbeam-utils 0.8.23
- crypto-common 0.1.7
- ctor 0.8.0
- ctor-proc-macro 0.0.7
- data-url 0.3.2
- dbus 0.9.12
- deranged 0.5.8
- digest 0.10.7
- dirs 6.0.0
- dirs-sys 0.5.0
- dispatch2 0.3.1
- displaydoc 0.2.7
- dpi 0.1.2
- dtoa 1.0.11
- dunce 1.0.5
- dyn-clone 1.0.20
- ecow 0.2.6
- either 1.18.0
- embed_plist 1.2.2
- equivalent 1.0.2
- erased-serde 0.4.10
- errno 0.3.14
- euclid 0.22.14
- fastrand 2.5.0
- fdeflate 0.3.7
- fearless_simd 0.4.1
- field-offset 0.3.6
- file-id 0.2.3
- filetime 0.2.29
- find-msvc-tools 0.1.13
- flate2 1.1.10
- fnv 1.0.7
- font-types 0.11.3
- foreign-types 0.5.0
- foreign-types-macros 0.2.4
- foreign-types-shared 0.3.1
- form_urlencoded 1.2.2
- futures-channel 0.3.34
- futures-core 0.3.34
- futures-executor 0.3.34
- futures-io 0.3.34
- futures-macro 0.3.34
- futures-sink 0.3.34
- futures-task 0.3.34
- futures-util 0.3.34
- getrandom 0.2.17
- getrandom 0.3.4
- getrandom 0.4.3
- gif 0.14.2
- glidesort 0.1.2
- glob 0.3.4
- guillotiere 0.7.0
- half 2.7.1
- hashbrown 0.12.3
- hashbrown 0.17.1
- hayagriva 0.10.1
- hayro 0.7.1
- hayro-ccitt 0.3.0
- hayro-cmap 0.1.0
- hayro-interpret 0.7.0
- hayro-jbig2 0.3.0
- hayro-jpeg2000 0.3.5
- hayro-postscript 0.1.0
- hayro-svg 0.7.0
- hayro-syntax 0.7.2
- hayro-write 0.7.0
- heck 0.4.1
- heck 0.5.0
- html5ever 0.38.0
- http 1.5.0
- httparse 1.10.1
- hyper-rustls 0.27.9
- hypher 0.1.8
- iana-time-zone 0.1.65
- ident_case 1.0.1
- idna 1.1.0
- idna_adapter 1.2.2
- image 0.25.10
- image-webp 0.2.4
- indexmap 1.9.3
- indexmap 2.14.2
- ipnet 2.12.2
- is_terminal_polyfill 1.70.2
- itoa 1.0.18
- json-patch 3.0.1
- jsonptr 0.6.3
- keyboard-types 0.7.0
- krilla 0.8.2
- krilla-svg 0.8.1
- kurbo 0.13.1
- libc 0.2.189
- libdbus-sys 0.2.7
- linebender_resource_handle 0.1.1
- linked-hash-map 0.5.6
- linux-raw-sys 0.12.1
- lock_api 0.4.14
- log 0.4.34
- markup5ever 0.38.0
- memmap2 0.9.11
- mime 0.3.17
- miniz_oxide 0.8.9
- miniz_oxide 0.9.1
- moxcms 0.8.1
- muda 0.19.3
- notify-debouncer-full 0.7.0
- notify-types 2.1.0
- num-bigint 0.4.8
- num-conv 0.2.2
- num-integer 0.1.47
- num-traits 0.2.19
- objc2-app-kit 0.3.2
- objc2-core-foundation 0.3.2
- objc2-core-graphics 0.3.2
- objc2-exception-helper 0.1.1
- objc2-osa-kit 0.3.2
- objc2-quartz-core 0.3.2
- objc2-web-kit 0.3.2
- object 0.39.1
- once_cell 1.21.4
- once_cell_polyfill 1.70.2
- openssl-probe 0.2.1
- osakit 0.3.1
- palette 0.7.7
- palette_derive 0.7.7
- palette_math 0.7.7
- parking_lot 0.12.5
- parking_lot_core 0.9.12
- paste 1.0.15
- pdf-writer 0.15.0
- peniko 0.6.1
- percent-encoding 2.3.2
- pic-scale 0.7.12
- pin-project-lite 0.2.17
- pixglyph 0.6.1
- pkg-config 0.3.34
- png 0.17.16
- png 0.18.1
- polycool 0.4.0
- portable-atomic 1.15.0
- postcard 1.1.3
- powerfmt 0.2.0
- ppv-lite86 0.2.21
- proc-macro-crate 1.3.1
- proc-macro-crate 2.0.0
- proc-macro-error 1.0.4
- proc-macro-error-attr 1.0.4
- proc-macro-hack 0.5.20+deprecated
- proc-macro2 1.0.107
- psm 0.1.32
- pxfm 0.1.30
- quick-error 2.0.1
- quote 1.0.47
- rand 0.8.8
- rand_chacha 0.3.1
- rand_core 0.6.4
- raw-window-handle 0.6.2
- rayon 1.12.0
- rayon-core 1.13.0
- read-fonts 0.39.2
- regex 1.13.1
- regex-automata 0.4.18
- regex-syntax 0.8.11
- reqwest 0.13.5
- resvg 0.47.0
- ring 0.17.14
- roxmltree 0.20.0
- roxmltree 0.21.1
- rustc-hash 2.1.3
- rustc_version 0.4.1
- rustix 1.1.5
- rustls 0.23.45
- rustls-native-certs 0.8.4
- rustls-pki-types 1.15.1
- rustls-platform-verifier 0.7.0
- ryu 1.0.23
- scopeguard 1.2.0
- security-framework 3.7.0
- security-framework-sys 2.17.0
- semver 1.0.28
- serde 1.0.229
- serde-untagged 0.1.9
- serde_core 1.0.229
- serde_derive 1.0.229
- serde_derive_internals 0.29.1
- serde_json 1.0.151
- serde_path_to_error 0.1.20
- serde_repr 0.1.21
- serde_spanned 0.6.9
- serde_spanned 1.1.1
- serde_with 3.23.0
- serde_with_macros 3.23.0
- serde_yaml 0.9.34+deprecated
- serialize-to-javascript 0.1.2
- serialize-to-javascript-impl 0.1.2
- servo_arc 0.4.3
- sha2 0.10.9
- shlex 2.0.1
- simplecss 0.2.2
- siphasher 1.0.3
- skrifa 0.42.1
- smallvec 1.16.1
- socket2 0.6.5
- softbuffer 0.4.8
- stable_deref_trait 1.2.1
- stacker 0.1.25
- string_cache 0.9.0
- string_cache_codegen 0.6.1
- subsetter 0.2.6
- svgtypes 0.16.1
- swift-rs 1.0.8
- syn 1.0.109
- syn 2.0.119
- syn 3.0.6
- sync_wrapper 1.0.2
- system-configuration 0.7.0
- system-configuration-sys 0.6.0
- system-deps 6.2.2
- tao 0.35.3
- tar 0.4.46
- tauri 2.11.6
- tauri-build 2.6.3
- tauri-codegen 2.6.3
- tauri-macros 2.6.3
- tauri-plugin 2.6.3
- tauri-plugin-dialog 2.7.3
- tauri-plugin-fs 2.5.2
- tauri-plugin-updater 2.11.0
- tauri-runtime 2.11.3
- tauri-runtime-wry 2.11.4
- tauri-utils 2.9.3
- tempfile 3.27.0
- tendril 0.5.1
- thin-vec 0.2.20
- thiserror 1.0.69
- thiserror 2.0.20
- thiserror-impl 1.0.69
- thiserror-impl 2.0.20
- time 0.3.55
- time-core 0.1.9
- time-macros 0.2.32
- tinyvec 1.13.3
- tokio-rustls 0.26.5
- toml 0.8.23
- toml 0.9.12+spec-1.1.0
- toml 1.1.6+spec-1.1.0
- toml_datetime 0.6.11
- toml_datetime 0.7.5+spec-1.1.0
- toml_datetime 1.1.1+spec-1.1.0
- toml_edit 0.19.15
- toml_edit 0.20.7
- toml_edit 0.22.27
- toml_parser 1.1.3+spec-1.1.0
- toml_write 0.1.2
- toml_writer 1.1.2+spec-1.1.0
- ttf-parser 0.25.1
- two-face 0.4.5
- typeid 1.0.3
- typenum 1.20.1
- typst 0.15.1
- typst-assets 0.15.1
- typst-eval 0.15.1
- typst-html 0.15.1
- typst-ide 0.15.1
- typst-kit 0.15.1
- typst-layout 0.15.1
- typst-library 0.15.1
- typst-macros 0.15.1
- typst-pdf 0.15.1
- typst-realize 0.15.1
- typst-render 0.15.1
- typst-svg 0.15.1
- typst-syntax 0.15.1
- typst-timing 0.15.1
- typst-utils 0.15.1
- unic-char-property 0.9.0
- unic-char-range 0.9.0
- unic-common 0.9.0
- unic-langid 0.9.6
- unic-langid-impl 0.9.6
- unic-langid-macros 0.9.6
- unic-langid-macros-impl 0.9.6
- unic-ucd-ident 0.9.0
- unic-ucd-version 0.9.0
- unicase 2.9.0
- unicode-bidi 0.3.18
- unicode-bidi-mirroring 0.4.0
- unicode-ccc 0.4.0
- unicode-ident 1.0.26
- unicode-math-class 0.1.0
- unicode-normalization 0.1.25
- unicode-properties 0.1.4
- unicode-script 0.5.8
- unicode-segmentation 1.13.3
- unicode-vo 0.1.0
- unscanny 0.1.0
- url 2.5.8
- usvg 0.47.0
- utf16_iter 1.0.5
- utf8_iter 1.0.4
- utf8parse 0.2.2
- uuid 1.26.1
- vello_common 0.0.8
- vello_cpu 0.0.8
- version_check 0.9.5
- wasmi 1.1.0
- wasmi_collections 1.1.0
- wasmi_core 1.1.0
- wasmi_ir 1.1.0
- wasmparser 0.239.0
- web_atoms 0.2.6
- weezl 0.1.12
- winapi 0.3.9
- window-vibrancy 0.6.0
- windows 0.61.3
- windows-collections 0.2.0
- windows-core 0.61.2
- windows-core 0.62.2
- windows-future 0.2.1
- windows-implement 0.60.2
- windows-interface 0.59.3
- windows-link 0.1.3
- windows-link 0.2.1
- windows-numerics 0.2.0
- windows-registry 0.6.1
- windows-result 0.3.4
- windows-result 0.4.1
- windows-strings 0.4.2
- windows-strings 0.5.1
- windows-sys 0.59.0
- windows-sys 0.60.2
- windows-sys 0.61.2
- windows-targets 0.52.6
- windows-targets 0.53.5
- windows-threading 0.1.0
- windows-version 0.1.7
- windows_x86_64_gnu 0.52.6
- windows_x86_64_gnu 0.53.1
- windows_x86_64_msvc 0.52.6
- windows_x86_64_msvc 0.53.1
- write-fonts 0.48.1
- write16 1.0.0
- wry 0.55.1
- xattr 1.6.1
- xmp-writer 0.3.3
- yaml-rust 0.4.5
- zerocopy 0.8.57
- zerocopy-derive 0.8.57
- zeroize 1.9.0
- zune-core 0.5.3
- zune-jpeg 0.5.15

### Apache-2.0 WITH LLVM-exception

- ar_archive_writer 0.5.3
- linux-raw-sys 0.12.1
- rustix 1.1.5
- target-lexicon 0.12.16
- wasmparser 0.239.0

### BSD-2-Clause

- arrayref 0.3.9
- kamadak-exif 0.6.1
- mutate_once 0.1.2
- zerocopy 0.8.57
- zerocopy-derive 0.8.57

### BSD-3-Clause

- alloc-no-stdlib 2.0.4
- alloc-stdlib 0.2.4
- brotli 8.0.4
- brotli-decompressor 5.0.3
- moxcms 0.8.1
- pic-scale 0.7.12
- pxfm 0.1.30
- subtle 2.6.1
- tiny-skia 0.12.0
- tiny-skia-path 0.12.0

### BSL-1.0

- ryu 1.0.23

### CC0-1.0

- dunce 1.0.5
- notify 8.2.0
- roman-numerals-rs 3.1.0

### ISC

- hyper-rustls 0.27.9
- inotify 0.11.5
- inotify-sys 0.1.8
- ring 0.17.14
- rustls 0.23.45
- rustls-native-certs 0.8.4
- rustls-webpki 0.103.15
- untrusted 0.9.0

### MIT

- adler2 2.0.1
- aho-corasick 1.1.5
- anstream 1.0.0
- anstyle 1.0.14
- anstyle-parse 1.0.0
- anstyle-query 1.1.5
- anstyle-wincon 3.0.11
- anyhow 1.0.104
- arrayvec 0.7.8
- atk 0.18.2
- atk-sys 0.18.2
- atomic-waker 1.1.2
- autocfg 1.5.1
- az 1.3.0
- base64 0.21.7
- base64 0.22.1
- base64 0.23.1
- biblatex 0.12.0
- bincode 1.3.3
- bit-set 0.8.0
- bit-vec 0.8.0
- bitflags 1.3.2
- bitflags 2.13.2
- block-buffer 0.10.4
- block2 0.6.2
- booker-app 0.2.0
- booker-cli 0.2.0
- booker-core 0.2.0
- booker-doc 0.2.0
- booker-project 0.2.0
- booker-typst 0.2.0
- brotli 8.0.4
- brotli-decompressor 5.0.3
- bumpalo 3.20.3
- by_address 1.2.1
- bytemuck 1.25.2
- bytemuck_derive 1.12.1
- byteorder 1.5.0
- byteorder-lite 0.1.0
- bytes 1.12.1
- cairo-rs 0.18.5
- cairo-sys-rs 0.18.2
- camino 1.2.6
- cargo-platform 0.1.9
- cargo_metadata 0.19.2
- cargo_toml 0.22.3
- cc 1.4.7
- cfb 0.7.3
- cfg-expr 0.15.8
- cfg-if 1.0.5
- chinese-number 0.7.8
- chinese-variant 1.1.6
- chrono 0.4.45
- citationberg 0.7.0
- clap 4.6.7
- clap_builder 4.6.7
- clap_derive 4.6.7
- clap_lex 1.1.1
- cobs 0.3.0
- color 0.3.3
- color_quant 1.1.0
- colorchoice 1.0.5
- comemo 0.5.1
- comemo-macros 0.5.1
- cookie 0.18.2
- core-foundation 0.10.1
- core-foundation 0.9.4
- core-foundation-sys 0.8.7
- core-graphics 0.25.0
- core-graphics-types 0.2.0
- core_maths 0.1.1
- cpufeatures 0.2.17
- crc32fast 1.5.2
- crossbeam-channel 0.5.17
- crossbeam-deque 0.8.8
- crossbeam-epoch 0.9.21
- crossbeam-utils 0.8.23
- crypto-common 0.1.7
- csv 1.4.0
- csv-core 0.1.13
- ctor 0.8.0
- ctor-proc-macro 0.0.7
- darling 0.24.1
- darling_core 0.24.1
- darling_macro 0.24.1
- data-url 0.3.2
- dbus 0.9.12
- deranged 0.5.8
- derive_more 2.1.1
- derive_more-impl 2.1.1
- digest 0.10.7
- dirs 6.0.0
- dirs-sys 0.5.0
- dispatch2 0.3.1
- displaydoc 0.2.7
- dlopen2 0.8.2
- dlopen2_derive 0.4.3
- dom_query 0.27.0
- dpi 0.1.2
- dtoa 1.0.11
- dyn-clone 1.0.20
- ecow 0.2.6
- either 1.18.0
- embed-resource 3.0.11
- embed_plist 1.2.2
- enum-ordinalize 4.4.2
- enum-ordinalize-derive 4.4.2
- equivalent 1.0.2
- erased-serde 0.4.10
- errno 0.3.14
- euclid 0.22.14
- fancy-regex 0.16.2
- fastrand 2.5.0
- fdeflate 0.3.7
- fearless_simd 0.4.1
- field-offset 0.3.6
- file-id 0.2.3
- filetime 0.2.29
- find-msvc-tools 0.1.13
- flate2 1.1.10
- float-cmp 0.9.0
- fnv 1.0.7
- font-types 0.11.3
- fontconfig-parser 0.5.8
- fontdb 0.23.0
- foreign-types 0.5.0
- foreign-types-macros 0.2.4
- foreign-types-shared 0.3.1
- form_urlencoded 1.2.2
- fsevent-sys 4.1.0
- futures-channel 0.3.34
- futures-core 0.3.34
- futures-executor 0.3.34
- futures-io 0.3.34
- futures-macro 0.3.34
- futures-sink 0.3.34
- futures-task 0.3.34
- futures-util 0.3.34
- gdk 0.18.2
- gdk-pixbuf 0.18.5
- gdk-pixbuf-sys 0.18.0
- gdk-sys 0.18.2
- gdkwayland-sys 0.18.2
- gdkx11 0.18.2
- gdkx11-sys 0.18.2
- generic-array 0.14.7
- getrandom 0.2.17
- getrandom 0.3.4
- getrandom 0.4.3
- gif 0.14.2
- gio 0.18.4
- gio-sys 0.18.1
- glib 0.18.5
- glib-macros 0.18.5
- glib-sys 0.18.1
- glidesort 0.1.2
- glob 0.3.4
- gobject-sys 0.18.0
- gtk 0.18.2
- gtk-sys 0.18.2
- gtk3-macros 0.18.2
- guillotiere 0.7.0
- half 2.7.1
- hashbrown 0.12.3
- hashbrown 0.17.1
- hayagriva 0.10.1
- hayro 0.7.1
- hayro-ccitt 0.3.0
- hayro-cmap 0.1.0
- hayro-interpret 0.7.0
- hayro-jbig2 0.3.0
- hayro-jpeg2000 0.3.5
- hayro-postscript 0.1.0
- hayro-svg 0.7.0
- hayro-syntax 0.7.2
- hayro-write 0.7.0
- heck 0.4.1
- heck 0.5.0
- html5ever 0.38.0
- http 1.5.0
- http-body 1.1.0
- http-body-util 0.1.5
- httparse 1.10.1
- hyper 1.11.1
- hyper-rustls 0.27.9
- hyper-util 0.1.20
- hypher 0.1.8
- iana-time-zone 0.1.65
- ico 0.5.0
- ident_case 1.0.1
- idna 1.1.0
- idna_adapter 1.2.2
- image 0.25.10
- image-webp 0.2.4
- imagesize 0.14.0
- indexmap 1.9.3
- indexmap 2.14.2
- infer 0.19.0
- ipnet 2.12.2
- is_terminal_polyfill 1.70.2
- itoa 1.0.18
- javascriptcore-rs 1.1.2
- javascriptcore-rs-sys 1.1.1
- json-patch 3.0.1
- jsonptr 0.6.3
- keyboard-types 0.7.0
- krilla 0.8.2
- krilla-svg 0.8.1
- kurbo 0.13.1
- libc 0.2.189
- libdbus-sys 0.2.7
- libm 0.2.16
- linebender_resource_handle 0.1.1
- linked-hash-map 0.5.6
- linux-raw-sys 0.12.1
- lipsum 0.9.1
- lock_api 0.4.14
- log 0.4.34
- markup5ever 0.38.0
- memchr 2.8.3
- memmap2 0.9.11
- memoffset 0.9.1
- mime 0.3.17
- minisign-verify 0.2.5
- miniz_oxide 0.8.9
- miniz_oxide 0.9.1
- mio 1.2.3
- muda 0.19.3
- new_debug_unreachable 1.0.6
- notify-debouncer-full 0.7.0
- notify-types 2.1.0
- num-bigint 0.4.8
- num-conv 0.2.2
- num-integer 0.1.47
- num-traits 0.2.19
- objc2 0.6.4
- objc2-app-kit 0.3.2
- objc2-core-foundation 0.3.2
- objc2-core-graphics 0.3.2
- objc2-encode 4.1.0
- objc2-exception-helper 0.1.1
- objc2-foundation 0.3.2
- objc2-osa-kit 0.3.2
- objc2-quartz-core 0.3.2
- objc2-web-kit 0.3.2
- object 0.39.1
- once_cell 1.21.4
- once_cell_polyfill 1.70.2
- openssl-probe 0.2.1
- osakit 0.3.1
- palette 0.7.7
- palette_derive 0.7.7
- palette_math 0.7.7
- pango 0.18.3
- pango-sys 0.18.0
- parking_lot 0.12.5
- parking_lot_core 0.9.12
- paste 1.0.15
- pdf-writer 0.15.0
- peniko 0.6.1
- percent-encoding 2.3.2
- phf 0.13.1
- phf_codegen 0.13.1
- phf_generator 0.13.1
- phf_macros 0.13.1
- phf_shared 0.13.1
- pico-args 0.5.0
- pin-project-lite 0.2.17
- pkg-config 0.3.34
- plist 1.10.1
- png 0.17.16
- png 0.18.1
- polycool 0.4.0
- portable-atomic 1.15.0
- postcard 1.1.3
- powerfmt 0.2.0
- ppv-lite86 0.2.21
- precomputed-hash 0.1.1
- proc-macro-crate 1.3.1
- proc-macro-crate 2.0.0
- proc-macro-error 1.0.4
- proc-macro-error-attr 1.0.4
- proc-macro-hack 0.5.20+deprecated
- proc-macro2 1.0.107
- psm 0.1.32
- pulldown-cmark 0.13.4
- quick-error 2.0.1
- quick-xml 0.38.4
- quick-xml 0.42.0
- quote 1.0.47
- rand 0.8.8
- rand_chacha 0.3.1
- rand_core 0.6.4
- raw-window-handle 0.6.2
- rayon 1.12.0
- rayon-core 1.13.0
- read-fonts 0.39.2
- regex 1.13.1
- regex-automata 0.4.18
- regex-syntax 0.8.11
- reqwest 0.13.5
- resvg 0.47.0
- rfd 0.16.0
- rgb 0.8.53
- roxmltree 0.20.0
- roxmltree 0.21.1
- rust_decimal 1.43.0
- rustc-hash 2.1.3
- rustc_version 0.4.1
- rustix 1.1.5
- rustls 0.23.45
- rustls-native-certs 0.8.4
- rustls-pki-types 1.15.1
- rustls-platform-verifier 0.7.0
- rustybuzz 0.20.1
- same-file 1.0.6
- schannel 0.1.29
- schemars 0.8.22
- schemars_derive 0.8.22
- scopeguard 1.2.0
- security-framework 3.7.0
- security-framework-sys 2.17.0
- semver 1.0.28
- serde 1.0.229
- serde-untagged 0.1.9
- serde_core 1.0.229
- serde_derive 1.0.229
- serde_derive_internals 0.29.1
- serde_json 1.0.151
- serde_path_to_error 0.1.20
- serde_repr 0.1.21
- serde_spanned 0.6.9
- serde_spanned 1.1.1
- serde_with 3.23.0
- serde_with_macros 3.23.0
- serde_yaml 0.9.34+deprecated
- serialize-to-javascript 0.1.2
- serialize-to-javascript-impl 0.1.2
- servo_arc 0.4.3
- sha2 0.10.9
- shlex 2.0.1
- simd-adler32 0.3.10
- simplecss 0.2.2
- siphasher 1.0.3
- skrifa 0.42.1
- slab 0.4.12
- smallvec 1.16.1
- socket2 0.6.5
- softbuffer 0.4.8
- soup3 0.5.0
- soup3-sys 0.5.0
- spin 0.9.9
- stable_deref_trait 1.2.1
- stacker 0.1.25
- strict-num 0.1.1
- string_cache 0.9.0
- string_cache_codegen 0.6.1
- strsim 0.11.1
- strum 0.27.2
- strum_macros 0.27.2
- subsetter 0.2.6
- svgtypes 0.16.1
- swift-rs 1.0.8
- syn 1.0.109
- syn 2.0.119
- syn 3.0.6
- synstructure 0.14.0
- syntect 5.3.0
- system-configuration 0.7.0
- system-configuration-sys 0.6.0
- system-deps 6.2.2
- tar 0.4.46
- tauri 2.11.6
- tauri-build 2.6.3
- tauri-codegen 2.6.3
- tauri-macros 2.6.3
- tauri-plugin 2.6.3
- tauri-plugin-dialog 2.7.3
- tauri-plugin-fs 2.5.2
- tauri-plugin-updater 2.11.0
- tauri-runtime 2.11.3
- tauri-runtime-wry 2.11.4
- tauri-utils 2.9.3
- tauri-winres 0.3.6
- tempfile 3.27.0
- tendril 0.5.1
- termcolor 1.4.1
- thin-vec 0.2.20
- thiserror 1.0.69
- thiserror 2.0.20
- thiserror-impl 1.0.69
- thiserror-impl 2.0.20
- time 0.3.55
- time-core 0.1.9
- time-macros 0.2.32
- tinyvec 1.13.3
- tokio 1.53.1
- tokio-rustls 0.26.5
- tokio-util 0.7.19
- toml 0.8.23
- toml 0.9.12+spec-1.1.0
- toml 1.1.6+spec-1.1.0
- toml_datetime 0.6.11
- toml_datetime 0.7.5+spec-1.1.0
- toml_datetime 1.1.1+spec-1.1.0
- toml_edit 0.19.15
- toml_edit 0.20.7
- toml_edit 0.22.27
- toml_parser 1.1.3+spec-1.1.0
- toml_write 0.1.2
- toml_writer 1.1.2+spec-1.1.0
- tower 0.5.3
- tower-http 0.6.11
- tower-layer 0.3.3
- tower-service 0.3.3
- tracing 0.1.44
- tracing-core 0.1.36
- try-lock 0.2.5
- ts-rs 9.0.1
- ts-rs-macros 9.0.1
- ttf-parser 0.25.1
- two-face 0.4.5
- typed-arena 2.0.2
- typeid 1.0.3
- typenum 1.20.1
- unic-char-property 0.9.0
- unic-char-range 0.9.0
- unic-common 0.9.0
- unic-langid 0.9.6
- unic-langid-impl 0.9.6
- unic-langid-macros 0.9.6
- unic-langid-macros-impl 0.9.6
- unic-ucd-ident 0.9.0
- unic-ucd-version 0.9.0
- unicase 2.9.0
- unicode-bidi 0.3.18
- unicode-bidi-mirroring 0.4.0
- unicode-ccc 0.4.0
- unicode-ident 1.0.26
- unicode-math-class 0.1.0
- unicode-normalization 0.1.25
- unicode-properties 0.1.4
- unicode-script 0.5.8
- unicode-segmentation 1.13.3
- unicode-vo 0.1.0
- unsafe-libyaml 0.2.11
- unscanny 0.1.0
- url 2.5.8
- urlpattern 0.3.0
- usvg 0.47.0
- utf16_iter 1.0.5
- utf8_iter 1.0.4
- utf8parse 0.2.2
- uuid 1.26.1
- vello_common 0.0.8
- vello_cpu 0.0.8
- version-compare 0.2.1
- version_check 0.9.5
- vswhom 0.1.0
- vswhom-sys 0.1.3
- walkdir 2.5.0
- want 0.3.1
- wasmi 1.1.0
- wasmi_collections 1.1.0
- wasmi_core 1.1.0
- wasmi_ir 1.1.0
- wasmparser 0.239.0
- web_atoms 0.2.6
- webkit2gtk 2.0.2
- webkit2gtk-sys 2.0.2
- webview2-com 0.38.2
- webview2-com-macros 0.8.1
- webview2-com-sys 0.38.2
- weezl 0.1.12
- winapi 0.3.9
- winapi-util 0.1.11
- window-vibrancy 0.6.0
- windows 0.61.3
- windows-collections 0.2.0
- windows-core 0.61.2
- windows-core 0.62.2
- windows-future 0.2.1
- windows-implement 0.60.2
- windows-interface 0.59.3
- windows-link 0.1.3
- windows-link 0.2.1
- windows-numerics 0.2.0
- windows-registry 0.6.1
- windows-result 0.3.4
- windows-result 0.4.1
- windows-strings 0.4.2
- windows-strings 0.5.1
- windows-sys 0.59.0
- windows-sys 0.60.2
- windows-sys 0.61.2
- windows-targets 0.52.6
- windows-targets 0.53.5
- windows-threading 0.1.0
- windows-version 0.1.7
- windows_x86_64_gnu 0.52.6
- windows_x86_64_gnu 0.53.1
- windows_x86_64_msvc 0.52.6
- windows_x86_64_msvc 0.53.1
- winnow 0.5.40
- winnow 0.7.15
- winnow 1.0.4
- winreg 0.55.0
- write-fonts 0.48.1
- write16 1.0.0
- wry 0.55.1
- x11 2.21.0
- x11-dl 2.21.0
- xattr 1.6.1
- xmlwriter 0.1.0
- xmp-writer 0.3.3
- xtask 0.2.0
- yaml-rust 0.4.5
- zerocopy 0.8.57
- zerocopy-derive 0.8.57
- zeroize 1.9.0
- zip 4.6.1
- zmij 1.0.23
- zune-core 0.5.3
- zune-jpeg 0.5.15

### MIT-0

- dunce 1.0.5

### MPL-2.0

- cssparser 0.36.0
- cssparser-macros 0.6.1
- dtoa-short 0.3.5
- option-ext 0.2.0
- selectors 0.36.1

### Unicode-3.0

- icu_collator 2.3.1
- icu_collator_data 2.3.0
- icu_collections 2.3.0
- icu_locale 2.3.1
- icu_locale_core 2.3.0
- icu_locale_data 2.3.0
- icu_locale_fallback 2.3.0
- icu_locale_fallback_data 2.3.0
- icu_normalizer 2.3.0
- icu_normalizer_data 2.3.0
- icu_properties 2.3.0
- icu_properties_data 2.3.0
- icu_provider 2.3.1
- icu_provider_blob 2.3.0
- icu_segmenter 2.3.0
- icu_segmenter_data 2.3.0
- litemap 0.8.3
- potential_utf 0.1.6
- tinystr 0.8.4
- unicode-ident 1.0.26
- writeable 0.6.4
- yoke 0.8.3
- yoke-derive 0.8.3
- zerofrom 0.1.8
- zerofrom-derive 0.1.8
- zerotrie 0.2.5
- zerovec 0.11.8
- zerovec-derive 0.11.6

### Unlicense

- aho-corasick 1.1.5
- byteorder 1.5.0
- byteorder-lite 0.1.0
- csv 1.4.0
- csv-core 0.1.13
- memchr 2.8.3
- same-file 1.0.6
- termcolor 1.4.1
- walkdir 2.5.0
- winapi-util 0.1.11

### Zlib

- bytemuck 1.25.2
- bytemuck_derive 1.12.1
- dispatch2 0.3.1
- foldhash 0.2.0
- miniz_oxide 0.8.9
- miniz_oxide 0.9.1
- objc2-app-kit 0.3.2
- objc2-core-foundation 0.3.2
- objc2-core-graphics 0.3.2
- objc2-exception-helper 0.1.1
- objc2-osa-kit 0.3.2
- objc2-quartz-core 0.3.2
- objc2-web-kit 0.3.2
- raw-window-handle 0.6.2
- slotmap 1.1.1
- tinyvec 1.13.3
- zlib-rs 0.6.8
- zune-core 0.5.3
- zune-jpeg 0.5.15
