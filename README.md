**This script has been integrated into [fxTools](https://github.com/moold/fxTools): `fxTools stat`.**

# fastat
ultrafast to get bases count/reads count/length distribution from fasta or fastq files

## Installation

#### Dependencies

`fastat` is written in rust, try below commands (no root required) or see [here](https://www.rust-lang.org/tools/install) to install `Rust` first.
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Download and install

```
git clone https://github.com/moold/fastat.git
cd fastat && cargo build --release
```

## Usage
`./target/release/fastat input1.fastq.gz input2.fastq.gz > input.fastq.stat`

## Parameters
Use `./target/release/fastat -h` to see options.

## Benchmarking

### ngs.fq.gz (~7G file size, ~11G bases)
```
Summary
  './fastat ngs.fq.gz' ran
    1.83 ± 0.01 times faster than './kseq ngs.fq.gz'
    2.55 ± 0.01 times faster than 'zcat ngs.fq.gz|wc -l'
    9.20 ± 0.04 times faster than './seqkit sum -a ngs.fq.gz'
```

### pb.fa.gz (~7G file size, ~26G bases)
```
Summary
  './fastat pb.fa.gz' ran
    1.63 ± 0.01 times faster than './kseq pb.fa.gz'
    2.18 ± 0.01 times faster than './seqkit sum -a pb.fa.gz'
    2.75 ± 0.01 times faster than 'zcat pb.fa.gz|wc -l'
```
See [details](./bench.md)

## Output demo 
```
[length histogram ('*' =~ 7116 reads)]
   45 14021   10321 *
14022 14363    4550 
14364 14705    8018 *
14706 15047   13673 *
15048 15389   21615 ***
15390 15731   32226 ****
15732 16073   44952 ******
16074 16415   58196 ********
16416 16757   72486 **********
16758 17099   84376 ***********
17100 17441   93906 *************
17442 17783   99349 *************
17784 18125  101953 **************
18126 18467  100981 **************
18468 18809   96074 *************
18810 19151   89310 ************
19152 19493   81196 ***********
19494 19835   71249 **********
19836 20177   62419 ********
20178 20519   53440 *******
20520 20861   44890 ******
20862 21203   37396 *****
21204 21545   31179 ****
21546 21887   25294 ***
21888 22229   19734 **
22230 22571   15935 **
22572 22913   12273 *
22914 23255    9657 *
23256 23597    7175 *
23598 46910   19453 **



[length stat]
Types Count (#) Length (bp)
N10    115930      21468   
N20    241978      20392   
N30    373509      19663   
N40    509438      19079   
N50    649305      18563   
N60    792987      18073   
N70    940626      17576   
N80    1092710     17023   
N90    1250454     16303   

Min.      -         45     
Max.      -        46910   
Ave.      -        18491   
Total  1423276  26318110120
```
