```bash
PI<const float>=3.14159265358979323846

circumference(
    radius<float>
    [--diameter]
)
    result<float>=
    if $--diameter 
        result=$radius * PI
    else
        result=$radius * 2 * PI
    -> $result

# or

circumference(
    radius<float>
    [--diameter]
)
    if $--diameter; -> $radius * PI; else -> $radius * 2 * PI

.calculate(
    value<float>
    [--radius]
    [--diameter]
)
    if $--radius
        -> circumference $value
    elif $--diameter
        -> circumference $value --diameter

    -> circumference $value
```



