### Performance

There will be no comparisons with other XML parsers since they do not parse SVG data.
And no comparisons with other SVG parsers, since there are no such.

Note that most of the time is spent during string to number and number to string conversion.

```
test parse_large  ... bench:  20,839,388 ns/iter (+/- 169,014)
test parse_medium ... bench:   2,969,591 ns/iter (+/- 42,878)
test parse_small  ... bench:      60,118 ns/iter (+/- 72)
test write_large  ... bench:  12,549,372 ns/iter (+/- 14,983)
test write_medium ... bench:   1,173,646 ns/iter (+/- 2,062)
test write_small  ... bench:      24,401 ns/iter (+/- 85)
```

Tested on i5-3570k 3.4GHz.
