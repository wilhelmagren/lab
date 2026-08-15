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

# OOM cases for pandas
oom_nrows = np.array([50_000_000, 100_000_000])

# Compute speedup (for valid data)
speedup = pandas / polars

# --- Plot 1: Read times ---
plt.figure(figsize=(10, 6))
plt.loglog(nrows, polars, "o-", label="Polars", linewidth=2)
plt.loglog(nrows, pandas, "s--", label="Pandas", linewidth=2)

# Add OOM annotations
plt.scatter(oom_nrows, [np.nan, np.nan], marker="x", color="red")  # placeholders
for x in oom_nrows:
    plt.text(
        x, 40, "OOM", color="red", fontsize=9, ha="center", va="bottom", rotation=0
    )

# Extend Polars line to show it keeps going
plt.loglog(
    [10_000_000, 50_000_000, 100_000_000],
    [5.9219563780006865, 28.512838358999943, 61.52426286500031],
    "o-",
    color="C0",
    linewidth=2,
)

plt.xlabel("Number of Rows (log scale)")
plt.ylabel("Read Time (seconds, log scale)")
plt.title("NDJSON Read Performance: Polars vs Pandas")
plt.legend()
plt.grid(True, which="both", linestyle="--", alpha=0.6)
plt.tight_layout()

# --- Plot 2: Speedup ratio ---
plt.figure(figsize=(10, 4))
plt.semilogx(nrows, speedup, "d-", color="green", linewidth=2)
plt.xlabel("Number of Rows (log scale)")
plt.ylabel("Speedup (Pandas / Polars)")
plt.title("Polars Speedup over Pandas (Higher = Faster)")
plt.grid(True, which="both", linestyle="--", alpha=0.6)

# Annotate the OOM region
plt.axvspan(10_000_000, 100_000_000, color="red", alpha=0.1)
plt.text(20_000_000, max(speedup) * 0.8, "Pandas OOM region", color="red", fontsize=10)

plt.tight_layout()
plt.savefig("speedup2.png")
