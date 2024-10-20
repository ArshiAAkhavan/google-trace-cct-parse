import re
import sys
from collections import OrderedDict  # Ensure this import is at the top of your script

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
from matplotlib.ticker import MaxNLocator  # Import MaxNLocator

# Regular expression pattern to parse each line
pattern = re.compile(
    r"^(?P<mode>.+)\((?P<trace>[^)]+)\) \[(?P<num_thread>\d+)\]::(?P<optimization_level>\w+) :\s*(?P<file_read>\d+),\s*(?P<cct_parse>\d+)$"
)


def parse_data(data_file):
    data_records = []

    with open(data_file, "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue  # Skip empty lines
            match = pattern.match(line)
            if match:
                record = match.groupdict()
                # Convert appropriate fields to integers
                record["num_thread"] = int(record["num_thread"])
                record["file_read"] = int(record["file_read"])
                record["cct_parse"] = int(record["cct_parse"])
                data_records.append(record)
            else:
                print(f"Line did not match the pattern: {line}")

    df = pd.DataFrame(data_records)
    # Ensure numeric columns are of type int
    df[["num_thread", "file_read", "cct_parse"]] = df[
        ["num_thread", "file_read", "cct_parse"]
    ].astype(int)

    df["trace"] = df["trace"].str.replace(r"^\./data/", "", regex=True)
    df["trace"] = df["trace"].str.replace(r"\.json$", "", regex=True)
    return df


def compute_statistics(df):
    """
    Computes min, max, and average times for each experiment and returns an aggregated DataFrame.
    """
    agg_df = (
        df.groupby(["mode", "trace", "num_thread", "optimization_level"])
        .agg(
            file_read_min=("file_read", "min"),
            file_read_max=("file_read", "max"),
            file_read_mean=("file_read", "mean"),
            cct_parse_min=("cct_parse", "min"),
            cct_parse_max=("cct_parse", "max"),
            cct_parse_mean=("cct_parse", "mean"),
        )
        .reset_index()
    )

    # Compute total average time (file_read_mean + cct_parse_mean)
    agg_df["total_time_mean"] = agg_df["file_read_mean"] + agg_df["cct_parse_mean"]
    print(
        agg_df[
            (agg_df["trace"] == "trace-100%")
            & (agg_df["optimization_level"] == "Release")
        ]
    )
    return agg_df


def plot_for_each_num_threads_on_release(agg_df):
    """
    Generates bar charts for each num_thread in 'Release' optimization level.
    """
    release_df = agg_df[agg_df["optimization_level"] == "Release"]
    num_threads = release_df["num_thread"].unique()

    for num_thread in sorted(num_threads):
        subset_df = release_df[release_df["num_thread"] == num_thread]
        pivot_df = subset_df.pivot(
            index="trace", columns="mode", values="total_time_mean"
        )

        # Plotting the bar chart
        ax = pivot_df.plot(kind="bar", figsize=(12, 6))
        ax.set_ylabel("Time (ms)")
        ax.set_title(f"total time in Release mode, using {num_thread} threads")
        ax.set_xticklabels(pivot_df.index, rotation=45, ha="right")
        # Increase the number of Y-axis ticks for more precision
        ax.yaxis.set_major_locator(MaxNLocator(integer=False, nbins=10))
        plt.tight_layout()
        plt.show()
        # If you want to save the figure, uncomment the following line
        # plt.savefig(f'bar_chart_release_{num_thread}.png')


def plot_num_threads_when_opt_level_is(agg_df, optimization_level):
    """
    Generates bar charts where the X-axis is num_thread,
    and each bar represents different modes on trace="trace-100" for each optimization level.
    """
    trace_name = "trace-100%"

    subset_df = agg_df[
        (agg_df["trace"] == trace_name)
        & (agg_df["optimization_level"] == optimization_level)
    ]

    if subset_df.empty:
        print(
            f"No data for optimization level '{optimization_level}' and trace '{trace_name}'"
        )
        return

    pivot_df = subset_df.pivot(
        index="num_thread", columns="mode", values="total_time_mean"
    )
    pivot_df = pivot_df.sort_index()

    # Plotting the bar chart
    ax = pivot_df.plot(kind="bar", figsize=(10, 6))
    ax.set_xlabel("Number of Threads")
    ax.set_ylabel("Total Time (ms)")
    ax.set_title(f"total time in {optimization_level} mode")
    ax.set_xticklabels(pivot_df.index.astype(str), rotation=0)
    # Increase the number of Y-axis ticks for more precision
    ax.yaxis.set_major_locator(MaxNLocator(integer=False, nbins=10))
    plt.legend(title="Mode")
    plt.tight_layout()
    plt.show()
    # To save the figure, uncomment the following line
    # plt.savefig(f'bar_chart_trace_{trace_name}_{optimization_level}.png')


def plot_optimization_level(agg_df, num_thread):
    """
    Generates a stacked bar chart where the X-axis is the mode,
    each mode has bars for each optimization level,
    bars are stacked to show file_read_mean and cct_parse_mean,
    filtered by trace="trace-100" and num_thread=16.
    Each mode has a unique set of colors for its bars.
    """
    # Filter the data
    trace_name = "trace-100%"
    subset_df = agg_df[
        (agg_df["trace"] == trace_name) & (agg_df["num_thread"] == num_thread)
    ]

    if subset_df.empty:
        print(f"No data for trace '{trace_name}' and num_thread {num_thread}")
        return

    # Pivot the data to have modes on the x-axis and optimization levels as columns
    # We need to have 'file_read_mean' and 'cct_parse_mean' as stacked components
    pivot_df = subset_df.pivot(
        index="mode",
        columns="optimization_level",
        values=["file_read_mean", "cct_parse_mean"],
    )

    # Ensure the columns are sorted for consistent plotting
    pivot_df = pivot_df.sort_index(axis=1, level=1)

    # Prepare data for plotting
    modes = pivot_df.index.tolist()
    optimization_levels = pivot_df.columns.levels[1].tolist()
    x = np.arange(len(modes))  # X locations for the groups
    total_width = 0.8  # Total width for all bars at one x position
    bar_width = total_width / len(optimization_levels)  # Width of each bar

    fig, ax = plt.subplots(figsize=(12, 6))

    # Generate colors for each mode
    # We'll create a color map for each mode, assigning unique colors
    # For simplicity, we'll use a colormap
    import matplotlib.cm as cm

    # Create a color map
    cmap = cm.get_cmap("tab20")  # Using 'tab20' colormap which has 20 distinct colors

    # Assign colors to each mode
    mode_colors = {}
    num_modes = len(modes)
    for mode in modes:
        color1 = cmap(0)
        color2 = cmap(1)
        color3 = cmap(2)
        color4 = cmap(3)
        mode_colors[mode] = {
            "file_read": [
                color1,
                color3,
            ],  # Colors for file_read_mean for each optimization level
            "cct_parse": [
                color2,
                color4,
            ],  # Colors for cct_parse_mean for each optimization level
        }

    # Plotting
    for i, opt_level in enumerate(optimization_levels):
        offset = (i - len(optimization_levels) / 2) * bar_width + bar_width / 2

        for j, mode in enumerate(modes):
            fr_value = pivot_df.loc[mode, ("file_read_mean", opt_level)]
            cp_value = pivot_df.loc[mode, ("cct_parse_mean", opt_level)]

            fr_color = mode_colors[mode]["file_read"][i]
            cp_color = mode_colors[mode]["cct_parse"][i]

            # Plot the file_read_mean bars
            ax.bar(
                x[j] + offset, fr_value, bar_width, color=fr_color, edgecolor="black"
            )
            # Plot the cct_parse_mean bars on top of the file_read_mean bars
            ax.bar(
                x[j] + offset,
                cp_value,
                bar_width,
                bottom=fr_value,
                color=cp_color,
                edgecolor="black",
            )

    # Create custom legend
    from matplotlib.patches import Patch

    legend_patches = []
    for mode in modes:
        for i, opt_level in enumerate(optimization_levels):
            fr_label = f"File Read ({opt_level}, {mode})"
            cp_label = f"CCT Parse ({opt_level}, {mode})"
            fr_patch = Patch(
                facecolor=mode_colors[mode]["file_read"][i],
                edgecolor="black",
                label=fr_label,
            )
            cp_patch = Patch(
                facecolor=mode_colors[mode]["cct_parse"][i],
                edgecolor="black",
                label=cp_label,
            )
            legend_patches.extend([fr_patch, cp_patch])

    # Remove duplicate labels in legend
    by_label = OrderedDict((patch.get_label(), patch) for patch in legend_patches)
    ax.legend(
        by_label.values(),
        by_label.keys(),
        title="Components (Optimization Level, Mode)",
        bbox_to_anchor=(1.05, 1),
        loc="upper left",
    )

    # Customize the plot
    ax.set_xlabel("Mode")
    ax.set_ylabel("Time (ms)")
    ax.set_title(f"total time using {num_thread} threads")
    ax.set_xticks(x)
    ax.set_xticklabels(modes, rotation=45, ha="right")
    # Increase the number of Y-axis ticks for more precision
    ax.yaxis.set_major_locator(MaxNLocator(integer=False, nbins=10))

    plt.tight_layout()
    plt.show()
    # To save the figure, uncomment the following line
    # plt.savefig(f'stacked_bar_chart_trace_{trace_name}_num_threads_{num_thread_value}_unique_colors.png')


def main():
    # Replace 'data.txt' with the path to your data file
    data_file = sys.argv[1]

    # Parse the data file
    df = parse_data(data_file)

    # Compute statistics
    agg_df = compute_statistics(df)
    no_baseline_df = agg_df[agg_df["mode"] != "baseline"]

    # Generate bar charts
    plot_num_threads_when_opt_level_is(agg_df, "Release")
    plot_num_threads_when_opt_level_is(agg_df, "Debug")
    plot_for_each_num_threads_on_release(agg_df)
    plot_optimization_level(agg_df, 1)
    plot_optimization_level(no_baseline_df, 2)
    plot_optimization_level(no_baseline_df, 4)
    plot_optimization_level(no_baseline_df, 8)
    plot_optimization_level(no_baseline_df, 16)
    plot_optimization_level(no_baseline_df, 32)


if __name__ == "__main__":
    main()
