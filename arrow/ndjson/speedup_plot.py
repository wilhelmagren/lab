import matplotlib.pyplot as plt
import numpy as np

# Data
nrows = np.array(
    [
        10,
        100,
        1000,
        5000,
        10000,
        15000,
        20000,
        50000,
        100000,
        200000,
        500000,
        1000000,
        5000000,
        10000000,
    ]
)
polars = np.array(
    [
        0.007355659000495507,
        0.0010151719998248154,
        0.0011897929998667678,
        0.003459230999396823,
        0.006393096000465448,
        0.009729254000376386,
        0.012915016999613727,
        0.03575382399958471,
        0.06168411700036813,
        0.13832516799993755,
        0.3287481580000531,
        0.6282745099997555,
        3.120174565000525,
        5.9219563780006865,
    ]
)
pandas = np.array(
    [
        0.004064355999616964,
        0.0016479510004501208,
        0.0030234559999371413,
        0.010151022000172816,
        0.01679692800007615,
        0.024701235000065935,
        0.03341553700010991,
        0.083116173000235,
        0.16723211800035642,
        0.34223719199962943,
        0.8960139920000074,
        1.7710612209993997,
        8.824555703999977,
        18.19564802900004,
    ]
)

# Compute speedup
speedup = pandas / polars

# Plot 1: Read times
plt.figure(figsize=(10, 6))
plt.loglog(nrows, polars, "o-", label="Polars", linewidth=2)
plt.loglog(nrows, pandas, "s--", label="Pandas", linewidth=2)
plt.xlabel("Number of Rows (log scale)")
plt.ylabel("Read Time (seconds, log scale)")
plt.title("NDJSON Read Performance: Polars vs Pandas")
plt.legend()
plt.grid(True, which="both", linestyle="--", alpha=0.6)
plt.tight_layout()

# Plot 2: Speedup ratio
plt.figure(figsize=(10, 4))
plt.semilogx(nrows, speedup, "d-", color="green", linewidth=2)
plt.xlabel("Number of Rows (log scale)")
plt.ylabel("Speedup (Pandas / Polars)")
plt.title("Polars Speedup over Pandas (Higher = Faster)")
plt.grid(True, which="both", linestyle="--", alpha=0.6)
plt.tight_layout()
plt.savefig("speedup.png")
