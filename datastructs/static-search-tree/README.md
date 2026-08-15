# Static Search Tree

The static search tree is a data structure that enables 40x faster binary search (only at
the cost of a 6% space overhead) [1].

**Input:** a sorted list $E, |E| = n$.

**Output:** a data structure $D$ that supports queries $q$ such that $D : q -> e$, where
$e \in E, min(E >= q)$ if such $e$ exists in $E$, else max(E).

**Metric:** We optimize *throughput*. That is, the number of (independent) queries $q$
that can be answered per second.

**Benchmarking setup:** We will assume that both the input and queries are 31 bit integers sampled
uniformly at random.

## References

- [1] Ragnar Groot Koerkamp, https://curiouscoding.nl/posts/static-search-tree/

