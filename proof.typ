// eipi.boo — a proof of the most beautiful equation
// e^(iπ) + 1 = 0

#set page(
  width: 6in,
  height: auto,
  margin: (x: 0.6in, top: 0.7in, bottom: 0.8in),
  fill: rgb("#faf4ed"),
)

#set text(
  font: "Palatino",
  size: 11.5pt,
  fill: rgb("#575279"),
)

#set par(justify: true, leading: 0.7em)
#set heading(numbering: none)

// — colors (rosé pine dawn) —
#let love = rgb("#b4637a")
#let pine = rgb("#286983")
#let iris = rgb("#907aa9")
#let foam = rgb("#56949f")
#let subtle = rgb("#797593")
#let muted = rgb("#9893a5")
#let rose = rgb("#d7827e")
#let overlay = rgb("#f2e9e1")

// — helpers —
#let accent(body) = text(fill: love, body)
#let note(body) = text(fill: pine, style: "italic", body)
#let mono(body) = text(font: "Menlo", size: 0.85em, fill: subtle, body)
#let qed = h(1fr) + text(fill: love, sym.square.filled)

#let styled-eq(body) = {
  set text(size: 1.05em)
  block(
    width: 100%,
    inset: (x: 1.2em, y: 0.9em),
    radius: 4pt,
    fill: overlay,
    stroke: 0.5pt + iris.transparentize(80%),
    align(center, body),
  )
}

// ─────────────────────────────────────────
// Title
// ─────────────────────────────────────────

#v(0.3in)

#align(center)[
  #text(size: 28pt, weight: "bold", fill: love)[
    $e^(i pi) + 1 = 0$
  ]
  #v(0.3em)
  #text(size: 10pt, tracking: 0.18em, fill: muted, font: "Menlo", weight: "light")[
    A PROOF OF EULER'S IDENTITY
  ]
  #v(0.15em)
  #mono[eipi.boo]
]

#v(0.4in)

#line(length: 100%, stroke: 0.4pt + iris.transparentize(85%))

#v(0.25in)

// ─────────────────────────────────────────
// Prologue
// ─────────────────────────────────────────

#text(fill: subtle, style: "italic", size: 10.5pt)[
  Five fundamental constants — #accent[$0$], #accent[$1$], #accent[$e$], #accent[$i$], #accent[$pi$] — from
  arithmetic, analysis, and geometry, woven into a single identity by a
  single stroke of Euler's pen. Here is why it is true.
]

#v(0.3in)

// ─────────────────────────────────────────
// §1  The exponential series
// ─────────────────────────────────────────

== #text(fill: pine)[§1] #h(0.3em) The exponential series

The exponential function is defined, for every $z in CC$, by its
Taylor series centered at the origin:

#styled-eq[$
  e^z = sum_(n=0)^oo z^n / n! = 1 + z + z^2/2! + z^3/3! + dots.c
$]

This series converges absolutely for all $z in CC$. Its radius of
convergence is infinite, which makes $e^z$ an #emph[entire] function —
analytic everywhere in the complex plane.

#v(0.2in)

// ─────────────────────────────────────────
// §2  The trigonometric series
// ─────────────────────────────────────────

== #text(fill: pine)[§2] #h(0.3em) The trigonometric series

The cosine and sine functions, for real $x$, admit their own Taylor
expansions:

#styled-eq[$
  cos x = sum_(n=0)^oo (-1)^n x^(2n) / (2n)! = 1 - x^2/2! + x^4/4! - dots.c
$]

#styled-eq[$
  sin x = sum_(n=0)^oo (-1)^n x^(2n+1) / (2n+1)! = x - x^3/3! + x^5/5! - dots.c
$]

Both converge absolutely for all real $x$. Note the pattern: cosine
collects the #emph[even] powers, sine the #emph[odd] — and the signs
alternate.

#v(0.2in)

// ─────────────────────────────────────────
// §3  Euler's formula
// ─────────────────────────────────────────

== #text(fill: pine)[§3] #h(0.3em) Euler's formula

Now substitute $z = i x$ (where $x in RR$) into the exponential series.
The key observation is the cyclic behavior of powers of $i$:

#align(center)[
  #table(
    columns: 4,
    align: center,
    stroke: none,
    inset: (x: 1em, y: 0.45em),
    table.header(
      ..([$n$], [$i^n$], [$n$], [$i^n$]).map(h => text(fill: iris, weight: "bold", h)),
    ),
    table.hline(stroke: 0.4pt + iris.transparentize(80%)),
    [$0$], [$1$], [$4$], [$1$],
    [$1$], [$i$], [$5$], [$i$],
    [$2$], [$-1$], [$6$], [$-1$],
    [$3$], [$-i$], [$7$], [$-i$],
  )
]

Expanding $e^(i x)$ term by term:

#styled-eq[$
  e^(i x) &= sum_(n=0)^oo (i x)^n / n! \
         &= 1 + i x - x^2/2! - i x^3/3! + x^4/4! + i x^5/5! - dots.c
$]

Group the real and imaginary parts:

#styled-eq[$
  e^(i x) = underbrace(1 - x^2/2! + x^4/4! - dots.c, cos x)
           + i underbrace((x - x^3/3! + x^5/5! - dots.c), sin x)
$]

This is #accent[*Euler's formula*]:

#styled-eq[$
  #text(size: 1.1em)[$e^(i x) = cos x + i sin x$]
$]

#note[
  Valid for all $x in RR$. The exponential series, when fed a purely imaginary
  argument, decomposes into the cosine and sine series — a bridge between
  the exponential and the circular.
]

#v(0.2in)

// ─────────────────────────────────────────
// §4  The identity
// ─────────────────────────────────────────

== #text(fill: pine)[§4] #h(0.3em) The identity

Set $x = pi$ in Euler's formula:

#styled-eq[$
  e^(i pi) = cos pi + i sin pi
$]

We know from elementary geometry:

#align(center)[
  #grid(
    columns: 2,
    column-gutter: 2em,
    row-gutter: 0.4em,
    text(fill: foam)[$cos pi = -1$],
    text(fill: foam)[$sin pi = #h(0.35em) 0$],
  )
]

Therefore:

#styled-eq[$
  e^(i pi) = -1 + i dot 0 = -1
$]

Adding $1$ to both sides:

#v(0.3em)

#block(
  width: 100%,
  inset: (x: 1.2em, y: 1em),
  radius: 4pt,
  fill: love.transparentize(94%),
  stroke: 1pt + love.transparentize(70%),
  align(center)[
    #text(size: 1.6em, weight: "bold", fill: love)[
      $e^(i pi) + 1 = 0$
    ]
  ],
)

#v(0.15em)
#qed

#v(0.3in)

// ─────────────────────────────────────────
// Epilogue
// ─────────────────────────────────────────

#line(length: 100%, stroke: 0.4pt + iris.transparentize(85%))

#v(0.2in)

#text(fill: subtle, style: "italic", size: 10.5pt)[
  Five constants, three operations (addition, multiplication,
  exponentiation), and one relation (equality) — nothing more. The
  identity connects the additive identity~($0$), the multiplicative
  identity~($1$), the base of natural logarithms~($e$), the ratio of a
  circle's circumference to its diameter~($pi$), and the fundamental
  imaginary unit~($i$). It is, as Feynman wrote in his notebooks,
  "the most remarkable formula in mathematics."
]

#v(0.4in)

#align(center)[
  #mono[~ boo ~]
  #v(0.5em)
  #text(size: 0.75em, fill: muted)[
    #link("https://pwnwriter.me")[pwnwriter.me]
  ]
]

#v(0.1in)
