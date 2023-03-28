import subprocess
import pathlib
import functools
from rich import print
from rich.panel import Panel
from rich.table import Table

from typing import Protocol, Union, Literal

TARGET = pathlib.Path(__file__).parent / "target" / "release"
RANK_SELECT = str((TARGET / "rank_select_experiment").absolute())
SPARSE_ARRAY = str((TARGET / "sparse_experiment").absolute())
LOGS = pathlib.Path(__file__).parent / "logs"


class Experiment(Protocol):
    def __call__(self, outfile: Union[str, pathlib.Path], *args, **kwargs):
        ...


RESULT_FOLDER = pathlib.Path(__file__).parent.absolute() / "new-results"


def run_command(
    cmd: list[str], logfile: Union[str, pathlib.Path] = None, **kwargs
) -> subprocess.CompletedProcess:
    if logfile is None:
        return subprocess.run(cmd, **kwargs)

    with open(str(logfile), "wb") as f:
        kwargs["stdout"] = f
        return subprocess.run(cmd, **kwargs)


def average_runs(f: Experiment = None, *, runs=3) -> Experiment:
    if f is None:
        return functools.partial(average_runs, runs=runs)

    @functools.wraps(f)
    def inner(outfile: Union[str, pathlib.Path], *args, **kwargs):
        original_path = pathlib.Path(outfile)
        run_directory = original_path.parent / original_path.stem
        run_directory.mkdir(exist_ok=True, parents=True)
        for run in range(1, runs + 1):
            print(f"Executing run [cyan]{run:02}[/cyan] of {runs}")
            run_outfile = run_directory / f"run-{run:02}{original_path.suffix}"
            f(run_outfile, *args, **kwargs)

    return inner


def print_command_block(name: str, **kwargs):
    grid = Table.grid()
    grid.add_column(min_width=15)
    grid.add_column(justify="left")
    for k, v in kwargs.items():
        grid.add_row(f"{k}", f"[cyan]{v}")
    print(Panel.fit(grid, title=f"[bold][green]{name}"))


def run_rank_select_experiment(
    outfile: Union[str, pathlib.Path],
    query_mode: Literal["rank", "select"],
    min_size: int = 1000,
    max_size: int = 1_000_000,
    step_size: int = 3_000,
    query_size: int = 1_000,
    block_size: Literal["dynamic", "fixed"] = "dynamic",
):
    print_command_block(
        f"{query_mode.upper()} EXPERIMENT",
        min_size=min_size,
        max_size=max_size,
        step_size=step_size,
        query_size=query_size,
        block_size=block_size,
    )
    run_command(
        [
            RANK_SELECT,
            str(outfile),
            query_mode,
            f"--min-size={min_size}",
            f"--max-size={max_size}",
            f"--query-size={query_size}",
            f"--block-size={block_size}",
        ],
        LOGS / f"{query_mode}.log",
    )


def run_sparsity_experiment(
    outfile: Union[str, pathlib.Path],
    vary_by: Literal["length", "sparsity"],
    query_mode: Literal["num-elem-at", "get-at-index", "get-index-of"],
    fixed_value: int,
    min_size: int,
    max_size: int,
    step_size: int,
    query_size: int = 1_000,
):
    print_command_block(
        f"Sparse Array Experiment: {query_mode} by {vary_by}",
        fixed_value=fixed_value,
        min_value=min_size,
        max_value=max_size,
        step_size=step_size,
        query_size=query_size,
    )
    fixed = "length" if vary_by == "sparsity" else "sparsity"
    run_command(
        [
            SPARSE_ARRAY,
            str(outfile),
            vary_by,
            query_mode,
            f"--min-{vary_by}={min_size}",
            f"--max-{vary_by}={max_size}",
            f"--{fixed}={fixed_value}",
            f"--query-size={query_size}",
        ],
        LOGS / "sparse.log",
    )


def build():
    if not TARGET.exists():
        run_command(["cargo", "build", "--release"])


if __name__ == "__main__":
    import click

    PARAMETER_RANGES = {
        "length": (1_000, 1_000_000, 3_000, 15),
        "sparsity": (0, 100, 1, 500_000),
    }

    @click.command()
    @click.option("-q", "--query-size", type=int)
    @click.option("-r", "--runs", default=3, type=int)
    @click.option("-m", "--max-size", type=int)
    @click.option("-m", "--step-size", type=int)
    def main(**kwargs):
        kwargs = {k: v for k, v in kwargs.items() if v is not None}
        build()
        r = average_runs(runs=kwargs["runs"])
        run_kwargs = {k: v for k, v in kwargs.items() if k in ["query_size", "max_size", "step_size"]}
        for experiment in ["rank", "select"]:
        # for experiment in ["rank", "select"]:
            # for block_size in ["dynamic", "fixed"]:
            for block_size in ["dynamic"]:
                r(run_rank_select_experiment)(
                    f"runs/{experiment}-support-{block_size}-block-size-wide.json",
                    experiment,
                    block_size=block_size,
                    **run_kwargs,
                )
        # for vary_by in ["length", "sparsity"]:
        #     for query_mode in ["num-elem-at", "get-at-index", "get-index-of"]:
        #         min_value, max_value, step_size, fixed = PARAMETER_RANGES[vary_by]
        #         outfile = f"runs/sparse-array-{query_mode}-vary-by-{vary_by}.json"
        #         r(run_sparsity_experiment)(
        #             outfile,
        #             vary_by,
        #             query_mode,
        #             fixed,
        #             min_size=min_value,
        #             max_size=max_value,
        #             step_size=step_size,
        #             **run_kwargs,
        #         )

    main()
