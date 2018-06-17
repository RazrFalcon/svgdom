### Performance

There will be no comparisons with other XML parsers since they do not parse SVG data.
And no comparisons with other SVG parsers, since there are no such.

Note that most of the time is spent during string to number and number to string conversion.

```
test parse_large  ... bench:  16,089,315 ns/iter (+/- 370,141)
test parse_medium ... bench:   2,869,603 ns/iter (+/- 3,144)
test parse_small  ... bench:      57,240 ns/iter (+/- 80)
test write_large  ... bench:  13,055,774 ns/iter (+/- 55,332)
test write_medium ... bench:   1,309,112 ns/iter (+/- 1,722)
test write_small  ... bench:      26,017 ns/iter (+/- 87)
```

Tested on i5-3570k 3.4GHz.
