# Suffix Array

In computer science, a **suffix array** is a sorted array of all suffixes of a string.

What is a suffix? A string $s$ is a suffix of a string $t$ if there exists a string $p$
such that $t = ps$. It is essentially a special case of a substring.

The string `nana` is a suffix of the string `banana`:

```
banana
  ||||
  nana
```

The suffix array is thus a data structure that lists the start positions of the suffixes
in alpabetically sorted order. It is a simpler alternative to the suffix tree.

## Definition

Let $S = S\[1\]S\[2\]...S\[n\]$ be an $n$-string and let $S\[i, j\]$ denote the substring
of $S$ ranging from $i$ to $j$ inclusive.

The suffix array $A$ of $S$ is now defined to be an array of integers providing the
starting positions of suffixes of $S$ in lexicographical order. This mean, an entry
$A\[i\]$ contains the starting position of the $i^{\text{th}}$ smallest suffix in $S$ and
thus for all $1 \leq i \leq n: S\[A\[i - 1], n] < S\[A\[i\], n]$.

Each suffix of $S$ shows up in $A$ exactly once.


## Example

Consider the text $S=$`banana$` to be indexed:

|**i**|1|2|3|4|5|6|7|
|--|--|--|--|--|--|--|--|
|$S\[i\]$|b|a|n|a|n|a|$|

The text ends with the special sentinel letter `$` that is unique and lexographically
smaller than any other character. The text has the following suffixes:

|**Suffix**|**i**|
|--|--|
|banana$|1|
|anana$|2|
|nana$|3|
|ana$|4|
|na$|5|
|a$|6|
|$|7|

These suffixes can be sorted in ascending (alphabetical) order:

|**Suffix**|**i**|
|--|--|
|$|7|
|a$|6|
|ana$|4|
|anana$|2|
|banana$|1|
|na$|5|
|nana$|3|

The suffix array $A$ contains the starting positions of these sorted suffixes:

|**i=**|1|2|3|4|5|6|7|
|--|--|--|--|--|--|--|--|
|$A\[i\]$**=**|7|6|4|2|1|5|3|

And the suffix array with the suffixes written out vertically underneath for clarity:

|**i=**|1|2|3|4|5|6|7|
|--|--|--|--|--|--|--|--|
|$A\[i\]$**=**|7|6|4|2|1|5|3|
|1|$|a|a|a|b|n|n|
|2| |$|n|n|a|a|a|
|3| | |a|a|n|$|n|
|4| | |$|n|a| |a|
|5| | | |$|n| |$|
|6| | | | |a| | |
|7| | | | |$| | |

So for example, $A\[3\]=4$, and therefore refers to the suffix starting at position 4
within $S$, which is the suffix `ana$`.


