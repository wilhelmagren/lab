Violations of 1NF:
  - Using row order to convey information
  - Mixing datatypes within the same column
  - Designing a table without a primary key
  - Repeating groups (struct/array type columns)

Violations of 2NF:
  - delete anomaly
  - update anomaly
  - insert anomaly

**Each non-key attribute must depend on the entire primary key.**

Violations of 3NF:
  - dependency of a non-key attribute on a another non-key attribute

(Boyce-Codd normal form)

**Every non-key attribute in a table should depend on the key, the whole key, and nothing but the key.**

