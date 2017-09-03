#!/usr/bin/python3

import urllib.request
import hashlib

items = [
    ("https://upload.wikimedia.org/wikipedia/commons/0/02/SVG_logo.svg",
     "small.svg", "71bbb30ab760b8f4da07639f4eeb32d6"),
    ("http://www.clker.com/cliparts/6/8/2/1/12344034191822607300Inkscape_skull_corneum.svg",
     "medium.svg", "5fcb9790c90b9362ff77daf062f963c9"),
    ("https://openclipart.org/download/231513/Colorful-Geometric-Line-Art-2.svg",
     "large.svg", "7d44ac52a89feca40f27d5024141d6cf"),
]

for item in items:
    urllib.request.urlretrieve(item[0], item[1])
    md5 = hashlib.md5(open(item[1], 'rb').read()).hexdigest()
    if md5 != item[2]:
        raise Exception("invalid hash")

