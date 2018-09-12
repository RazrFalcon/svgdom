### Performance

There will be no comparisons with other XML parsers since they do not parse SVG data.
And no comparisons with other SVG parsers, since there are no such.

Note that most of the time is spent during string to number and number to string conversion.

```
test parse_large  ... bench:  21,645,223 ns/iter (+/- 373,222)
test parse_medium ... bench:   3,566,053 ns/iter (+/- 7,969)
test parse_small  ... bench:      73,006 ns/iter (+/- 84)
test write_large  ... bench:  13,144,638 ns/iter (+/- 29,829)
test write_medium ... bench:   1,207,664 ns/iter (+/- 1,221)
test write_small  ... bench:      25,186 ns/iter (+/- 68)
```

Tested on i5-3570k 3.4GHz.
