# SVG DOM preprocessing

Unlike the usual XML DOM implementations, `svgdom` will preprocess the document/tree a lot.

## XML parsing

`svgdom` uses [`roxmltree`](https://github.com/RazrFalcon/roxmltree) as an XML parser. 
You can find its parsing details [here](https://github.com/RazrFalcon/roxmltree/blob/master/docs/parsing.md). 

## Non-SVG elements and attributes

Only SVG 1.1 elements and attributes will be parsed.
But it's possible to write custom elements and attributes.

## `style` attributes splitting

From:

```xml
<svg style="fill:black;stroke:green;"/>
```

to:

```xml
<svg fill="black" stroke="green"/>
```

## Text unescaping

All [character references](https://www.w3.org/TR/xml/#NT-CharRef) will be resolved.
Not only simple one like `&amp;` but also any hexadecimal and decimal.

Also, all whitespaces will be replaced with the Space character. Even escaped one.

The text data from the SVG like this:

```xml
<svg>
    <text> &#x20;&amp;&#64;&#x40;&amp;&#x20; <text>
</svg>
```

will be represented as `'@@'` and saved as:

```xml
<svg>
    <text>&amp;@@&amp;</text>
</svg>
```

## Whitespaces trimming

The text data from the SVG like this:

```xml
<svg>
    <text>
        Text
    </text>
</svg>
```

will be represented just as `Text` and not as `␣␣␣␣␣␣␣␣␣Text␣␣␣␣␣`.

`svgdom` also supports `xml:space` attribute. So the text data from the SVG like this:

```xml
<svg>
    <text>
        Text
        <tspan xml:space="preserve">  Text  </tspan>
        Text
    </text>
</svg>
```

will be represented as `Text␣␣␣Text␣␣Text`. And saved as:

```xml
<svg>
    <text>Text <tspan xml:space="preserve">  Text  </tspan>Text</text>
</svg>
```

Note that nested `xml:space` is mostly an undefined behavior and every XML DOM implementation
will process it differently. `svgdom` follows the Chrome behavior.
But Firefox, for example, will process the SVG above as `Text␣␣␣Text␣␣␣Text`.

## CSS resolving

`svgdom` supports only a tiny fraction of the CSS 2.1 features.
If the SVG contains an unsupported CSS, it will lead to a parsing error unless
the `ParseOptions::skip_invalid_css` is set.

After the preprocessing the `style` elements will removed.

From:

```xml
<svg>
    <style type="text/css">
        .fil1 { fill:green }
    </style>
    <rect class="fil1"/>
</svg>
```

to:

```xml
<svg>
    <rect fill="green"/>
</svg>
```

The proper style resolving order is supported too.

From:

```xml
<svg>
    <style type="text/css">
        .fil1 { fill:blue }
    </style>
    <rect fill="red" style="fill:green" class="fil1"/>
</svg>
```

to:

```xml
<svg>
    <rect fill="green"/>
</svg>
```

## Paint fallback resolving

SVG allows specifying the paint fallback value in case of an invalid FuncIRI.

For example:

```xml
<svg>
    <rect fill="url(#gradient1) green"/>
</svg>
```

`svgdom` will convert it into:

```xml
<svg>
    <rect fill="green"/>
</svg>
```

because there are no elements with a `gradient1` ID.
But more complex cases, like a reference to an invalid element, should be resolved manually.

For example:

```xml
<svg>
    <linearGradient id="gradient1"/>
    <rect fill="url(#gradient1) green"/>
</svg>
```

This will be represented as is, even though that `linearGradient` is invalid
(because have no children).

## Crosslink resolving

If an element is linked to itself, directly or indirectly, it may lead to a recursion/endless loop.

`svgdom` will resolve some simple cases by default.

In the example below the `lg1` gradient is linked to itself indirectly
via the `xlink:href` attribute.

From:

```xml
<svg>
    <linearGradient id="lg1" xlink:href="#lg2"/>
    <linearGradient id="lg2" xlink:href="#lg1"/>
</svg>
```

to:

```xml
<svg>
    <linearGradient id="lg1" xlink:href="#lg2"/>
    <linearGradient id="lg2"/>
</svg>
```

More complex cases should be resolved manually. Like:

```xml
<svg>
    <pattern id="patt1">
        <rect fill="url(#patt1)"/>
    </pattern>
</svg>
```

## Namespaces

There is only one namespace: http://www.w3.org/2000/svg .
All namespaces will be resolved and removed.

So SVG like this:

```xml
<svg:svg svg:x='0' xmlns:svg='http://www.w3.org/2000/svg'/>
```

will became:

```xml
<svg xmlns='http://www.w3.org/2000/svg' x='0'/>
```

`xmlns` and `xmlns:xlink` attributes/namespaces will be added automatically.

http://www.w3.org/1999/xlink is a special case and will be handled too.
