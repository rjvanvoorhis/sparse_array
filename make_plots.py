import pathlib
import json
import itertools
import pandas as pd


from typing import Union, TypedDict


class Duration(TypedDict):
    secs: int
    nanos: int


class Run(TypedDict):
    overhead: int
    parameter: int
    setup_duration: Duration
    query_duration: Duration


class Result:
    params: dict
    runs: list[Run]


def convert_duration(duration: Duration) -> pd.Timedelta:
    return pd.Timedelta(seconds=duration["secs"], nanoseconds=duration["nanos"])


def collect_results(run_folder: Union[str, pathlib.Path]) -> list[Result]:
    return list(
        map(
            lambda file: json.loads(file.read_text()),
            pathlib.Path(run_folder).glob("*.json"),
        )
    )


def extract_runs(results: list[Result]):
    if "params" in results[0]:
        query_size = results[0]["params"]["query_size"]
    else:
        # ugly
        query_size = list(results[0]["args"]["command"].values())[0]["query_size"]
    return [
        {
            **result,
            **{
                "setup_duration": convert_duration(result["setup_duration"]),
                "query_duration": convert_duration(result["query_duration"])
                / query_size
                # k: convert_duration(result[k])
                # for k in ["setup_duration", "query_duration"]
            },
        }
        for result in itertools.chain.from_iterable(
            result["runs"] for result in results
        )
    ]


def combine_runs(runs: list[Run]) -> pd.DataFrame:
    df = pd.DataFrame(runs)
    return df.groupby(df["parameter"]).aggregate(
        {
            "parameter": "first",
            "overhead": "mean",
            "setup_duration": "mean",
            "query_duration": "mean",
        }
    )


if __name__ == "__main__":
    import click
    import numpy as np

    NAME_DICT = {
        "query_duration": "Average query duration", "overhead": "Overhead", "overhead_ratio": "Overhead Ratio"}
    UNIT_DICT = {"query_duration": "nanoseconds", "overhead": "bits", "overhead_ratio": None}

    @click.command()
    @click.argument(
        "directory",
        type=click.Path(exists=True),
        callback=lambda _ctx, _param, value: pathlib.Path(value),
    )
    @click.option("-x", default="parameter")
    @click.option(
        "-y",
        default="query_duration",
        type=click.Choice(
            ["query_duration", "overhead", "overhead_ratio", "setup_duration"]
        ),
    )
    @click.option("-l", "--low", type=int, default=None)
    @click.option("-h", "--high", type=int, default=None)
    @click.option("-o", "--outfile", type=click.Path(), default="plot.png")
    @click.option("-u", "--unit", default=None)
    @click.option("--title", type=str)
    @click.option("--logx", default=False)
    @click.option("--xlabel", default="length")
    def main(
        directory: str,
        x: str,
        y: str,
        low: int = None,
        high: int = None,
        outfile: str = "plot.png",
        unit: str = None,
        **plot_kwargs,
    ):
        result = collect_results(directory)
        runs = extract_runs(result)
        runs = combine_runs(runs)
        unit = UNIT_DICT[y] if unit is None else unit
        ylabel = f"{NAME_DICT[y]} ({unit})"
        runs["overhead"]
        runs["overhead_ratio"] = runs["overhead"] / runs["parameter"]
        ylim = [low, high] if (low is not None and high is not None) else None
        fig = runs.plot(
            x=x,
            y=y,
            ylim=ylim,
            ylabel=ylabel,
            **plot_kwargs,
        ).get_figure()
        fig.savefig(outfile)

    main()