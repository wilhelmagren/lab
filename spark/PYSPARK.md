## PySpark course


### Spark Architecture

Master and slave.

Driver program <-> Cluster Manager <-> Worker Node 1, ..., Worker Node N

The driver program requests the cluster manager to create N worker nodes.

The driver program holds the current SparkContext of the query.

Each Worker Node has an executor with a cache and 1-N tasks.
The driver program divides the job into multiple tasks - split on the worker nodes.


### Why?

- In-memory computation
    - Hadoop was using disk memory (slow communication)
    - Spark all data in-memory
- Lazy evaluation
    - Transformations are not performed directly
    - A logical plan is derived and the transforms are only executed when we need the result from
    them.
    - DataFrame -> transformation1, ..., transformationN -> Logical Plan
        - this might perform the transformations in another order if it will lead to the same result
        - which can drastically improve performance
        - only executed when we need the result (perform an action)
            - `.show()`
            - `.display()`
            - `.collect()`
- Fault tolerant
- Partitioning
    - Distribute data to the cluster.

Spark jobs are divided into stages and each stage can have an arbitrary
amount of tasks.

```
Job/
   - stage1/
        - task1
        - task2
    - stage2/
        - task1
    ...
    - stageN/
        - task1
        - task2
        ...
        - taskM
```

