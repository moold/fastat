# Performance comparison between `fastat`, `kseq`, `seqkit` and `cat/zcat | wc-l`

## Requirement

- [kseq](https://github.com/lh3/biofast/blob/master/fqcnt/fqcnt_c1_kseq.c)
- [seqkit](https://bioinf.shenwei.me/seqkit/)
- [hyperfine](https://github.com/sharkdp/hyperfine)

## Command

```bash
infile=$1

if [[ ${infile} = *gz ]]
then
    cat="zcat"
else
    cat="cat"
fi

hyperfine --warmup 3 --style basic "fastat ${infile}"  "kseq ${infile}" "seqkit sum -a ${infile}" "${cat} ${infile}|wc -l"
```

## Result

### ngs.fq (~24G file size, ~11G bases)
```
Benchmark 1: ./fastat ngs.fq
  Time (mean ± σ):     136.185 s ±  0.166 s    [User: 17.600 s, System: 31.711 s]
  Range (min … max):   135.989 s … 136.542 s    10 runs

Benchmark 2: ./kseq ngs.fq
  Time (mean ± σ):     135.225 s ±  0.108 s    [User: 30.367 s, System: 23.946 s]
  Range (min … max):   135.098 s … 135.475 s    10 runs
 
Benchmark 3: ./seqkit sum -a ngs.fq
  Time (mean ± σ):     149.518 s ±  0.525 s    [User: 98.866 s, System: 34.372 s]
  Range (min … max):   148.858 s … 150.403 s    10 runs
 
Benchmark 4: cat ngs.fq|wc -l
  Time (mean ± σ):     134.882 s ±  0.102 s    [User: 6.619 s, System: 37.668 s]
  Range (min … max):   134.761 s … 135.041 s    10 runs
 
Summary
  'cat ngs.fq|wc -l' ran
    1.00 ± 0.00 times faster than './kseq ngs.fq'
    1.01 ± 0.00 times faster than './fastat ngs.fq'
    1.11 ± 0.00 times faster than './seqkit sum -a ngs.fq'
```
### ngs.fq.gz (~7G file size, ~11G bases)
```
Benchmark 1: ./fastat ngs.fq.gz
  Time (mean ± σ):     121.867 s ±  0.405 s    [User: 133.355 s, System: 6.874 s]
  Range (min … max):   120.914 s … 122.289 s    10 runs

Benchmark 2: ./kseq ngs.fq.gz
  Time (mean ± σ):     223.623 s ±  0.221 s    [User: 221.073 s, System: 2.546 s]
  Range (min … max):   223.045 s … 223.818 s    10 runs
 
Benchmark 3: ./seqkit sum -a ngs.fq.gz
  Time (mean ± σ):     1121.473 s ±  2.371 s    [User: 1359.907 s, System: 30.739 s]
  Range (min … max):   1117.466 s … 1124.147 s    10 runs
 
Benchmark 4: zcat ngs.fq.gz|wc -l
  Time (mean ± σ):     311.279 s ±  0.290 s    [User: 315.216 s, System: 23.951 s]
  Range (min … max):   310.701 s … 311.603 s    10 runs
 
Summary
  './fastat ngs.fq.gz' ran
    1.83 ± 0.01 times faster than './kseq ngs.fq.gz'
    2.55 ± 0.01 times faster than 'zcat ngs.fq.gz|wc -l'
    9.20 ± 0.04 times faster than './seqkit sum -a ngs.fq.gz'
```
### pb.fa (~25G file size, ~26G bases)
```
Benchmark 1: ./fastat pb.fa
  Time (mean ± σ):     144.066 s ±  0.197 s    [User: 8.775 s, System: 31.587 s]
  Range (min … max):   143.847 s … 144.327 s    10 runs

Benchmark 2: ./kseq pb.fa
  Time (mean ± σ):     144.000 s ±  0.172 s    [User: 8.846 s, System: 25.695 s]
  Range (min … max):   143.830 s … 144.276 s    10 runs
 
Benchmark 3: ./seqkit sum -a pb.fa
  Time (mean ± σ):     144.312 s ±  0.138 s    [User: 110.546 s, System: 19.056 s]
  Range (min … max):   144.185 s … 144.662 s    10 runs
 
Benchmark 4: cat pb.fa|wc -l
  Time (mean ± σ):     143.828 s ±  0.130 s    [User: 2.775 s, System: 48.876 s]
  Range (min … max):   143.681 s … 144.155 s    10 runs
 
Summary
  'cat pb.fa|wc -l' ran
    1.00 ± 0.00 times faster than './kseq pb.fa'
    1.00 ± 0.00 times faster than './fastat pb.fa'
    1.00 ± 0.00 times faster than './seqkit sum -a pb.fa'
```
### pb.fa.gz (~7G file size, ~26G bases)
```
Benchmark 1: ./fastat pb.fa.gz
  Time (mean ± σ):     122.476 s ±  0.554 s    [User: 129.043 s, System: 5.837 s]
  Range (min … max):   121.585 s … 123.124 s    10 runs

Benchmark 2: ./kseq pb.fa.gz
  Time (mean ± σ):     199.804 s ±  0.206 s    [User: 197.258 s, System: 2.542 s]
  Range (min … max):   199.431 s … 200.140 s    10 runs
 
Benchmark 3: ./seqkit sum -a pb.fa.gz
  Time (mean ± σ):     266.866 s ±  0.442 s    [User: 397.405 s, System: 6.876 s]
  Range (min … max):   265.979 s … 267.366 s    10 runs
 
Benchmark 4: zcat pb.fa.gz|wc -l
  Time (mean ± σ):     336.985 s ±  0.291 s    [User: 334.698 s, System: 31.423 s]
  Range (min … max):   336.454 s … 337.392 s    10 runs
 
Summary
  './fastat pb.fa.gz' ran
    1.63 ± 0.01 times faster than './kseq pb.fa.gz'
    2.18 ± 0.01 times faster than './seqkit sum -a pb.fa.gz'
    2.75 ± 0.01 times faster than 'zcat pb.fa.gz|wc -l'
```