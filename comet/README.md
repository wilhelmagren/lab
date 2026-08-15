# Apache Datafusion Comet

Apache Datafusion Comet is a high-performance accelerator for Apache Spark, built on top
of the Apache Datafusion query engine. Comet is designed to significantly enhance the
performance of Apache Spark workloads while leveraging commodity hardware and seamlessly
integrating with the Spark ecosystem without requiring any code changes.


## Building Comet From Source

We will be building from the github repo, first step is to clone it:

```
git clone https://github.com/apache/datafusion-comet.git
```

```
cd datafusion-comet
make release PROFILES="-Pspark-3.5.5 -Pscala-2.12.18"
```

and now you can run spark shell with comet enabled:

```

```
