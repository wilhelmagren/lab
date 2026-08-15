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

# Compute speedup
speedup = pandas / polars

"""
# --- Plot 1: Read times ---
plt.figure(figsize=(10, 6))
plt.loglog(nrows, polars, "o-", label="Polars", linewidth=2)
plt.loglog(nrows, pandas, "s--", label="Pandas", linewidth=2)

# Add OOM annotation for Pandas
for x in oom_nrows:
    plt.text(x, 40, "OOM", color="tab:red", fontsize=10, ha="center", va="bottom")

# Extend Polars line to huge datasets
plt.loglog(
    [10_000_000, 50_000_000, 100_000_000],
    [5.9219563780006865, 28.512838358999943, 61.52426286500031],
    "o-",
    color="tab:blue",
    linewidth=2,
)

plt.xlabel("Number of Rows (log scale)")
plt.ylabel("Read Time (seconds, log scale)")
plt.title("NDJSON Read Performance: Polars vs Pandas")
plt.legend()
plt.grid(True, which="both", linestyle="--", alpha=0.6)
plt.tight_layout()
"""

# --- Plot 2: Speedup + Polars timings ---
fig, ax1 = plt.subplots(figsize=(10, 5))

ax1.semilogx(
    nrows,
    speedup,
    "x-",
    color="tab:green",
    linewidth=1,
    label="Speedup (Pandas / Polars)",
)
ax1.set_xlabel("Number of Rows (log scale)")
ax1.set_ylabel("Speedup (Pandas / Polars)", color="tab:green")
ax1.tick_params(axis="y", labelcolor="tab:green")
ax1.grid(True, which="both", linestyle="--", alpha=0.4)

# Secondary y-axis for Polars timings
ax2 = ax1.twinx()
ax2.semilogx(
    nrows, polars, "+-", color="tab:blue", linewidth=1, label="Polars Time (s)"
)
# Extend Polars to huge datasets
ax2.semilogx(
    [10_000_000, 50_000_000, 100_000_000],
    [5.9219563780006865, 28.512838358999943, 61.52426286500031],
    "+-",
    color="tab:blue",
    linewidth=1,
)
ax2.set_ylabel("Polars Read Time (seconds)", color="tab:blue")
ax2.tick_params(axis="y", labelcolor="tab:blue")

# Highlight OOM region for Pandas
ax1.axvspan(10_000_000, 100_000_000, color="red", alpha=0.1)
ax1.text(
    20_000_000, max(speedup) * 0.8, "Pandas OOM region", color="tab:red", fontsize=10
)

# Combine legends
lines1, labels1 = ax1.get_legend_handles_labels()
lines2, labels2 = ax2.get_legend_handles_labels()
ax1.legend(lines1 + lines2, labels1 + labels2, loc="upper left")

plt.title("Polars Speedup & Absolute Timings (NDJSON Read)")
plt.tight_layout()
plt.savefig("speedup3.png")
