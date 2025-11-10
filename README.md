# weniger_krumm_rust

Implementation of the "Weniger Krumme Touren" task from the second round of the BWINF (2022?).

## compile and run

```bash
RUST_LOG=info cargo r --release -- --path ./src/assets/wenigerkrumm5.txt 1000
```

Vary the example file by adjusting the path.
The number is the amount of different paths that are checked for each combination of 3 valid start paths.
Computation time will increase when increasing this value, but results might improve.

It is recommended to try out different compiler flags to improve time.

For a short help message run:

```bash
cargo r -- --help
```


The programm will try to use all available threads.

