# @harmonize

Convert values between measurement systems — weight, distance, temperature, volume, speed, and currency.

---

## Simple Example

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'weight_lbs': [150.0, 180.0, 200.0],
})

result = add.transform('@harmonize', df, columns='weight_lbs',
    strategy={'from': 'lbs', 'to': 'kg'},
)
print(result)
```

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `columns` | `str` or `list[str]` | *(required)* | Numeric column(s) to convert |
| `strategy` | `dict` | *(required)* | Conversion specification with `from` and `to` units |

---

## Unit Categories

### Weight / Mass

Convert between kilograms, grams, pounds, ounces, and tons.

| Unit | Aliases |
|------|---------|
| Kilogram | `kg`, `kilogram`, `kilograms` |
| Gram | `g`, `gram`, `grams` |
| Pound | `lbs`, `lb`, `pound`, `pounds` |
| Ounce | `oz`, `ounce`, `ounces` |
| Ton | `ton`, `tons` |

```python
result = add.transform('@harmonize', df, columns='weight',
    strategy={'from': 'lbs', 'to': 'kg'})
```

### Distance / Length

Convert between meters, kilometers, centimeters, millimeters, miles, feet, and inches.

| Unit | Aliases |
|------|---------|
| Meter | `m`, `meter`, `meters` |
| Kilometer | `km`, `kilometer`, `kilometers` |
| Centimeter | `cm`, `centimeter`, `centimeters` |
| Millimeter | `mm`, `millimeter`, `millimeters` |
| Mile | `mi`, `mile`, `miles` |
| Foot | `ft`, `foot`, `feet` |
| Inch | `in`, `inch`, `inches` |

```python
result = add.transform('@harmonize', df, columns='distance',
    strategy={'from': 'mi', 'to': 'km'})
```

### Temperature

Convert between Celsius, Fahrenheit, and Kelvin.

| Unit | Aliases |
|------|---------|
| Celsius | `C`, `celsius` |
| Fahrenheit | `F`, `fahrenheit` |
| Kelvin | `K`, `kelvin` |

```python
result = add.transform('@harmonize', df, columns='temp',
    strategy={'from': 'F', 'to': 'C'})
```

### Volume

Convert between liters, gallons, and milliliters.

| Unit | Aliases |
|------|---------|
| Liter | `L`, `liter`, `liters` |
| Gallon | `gal`, `gallon`, `gallons` |
| Milliliter | `mL`, `milliliter`, `milliliters` |

```python
result = add.transform('@harmonize', df, columns='volume',
    strategy={'from': 'gal', 'to': 'L'})
```

### Speed

Convert between km/h, mph, and m/s.

| Unit | Aliases |
|------|---------|
| km/h | `km/h`, `kph` |
| mph | `mph` |
| m/s | `m/s`, `mps` |

```python
result = add.transform('@harmonize', df, columns='speed',
    strategy={'from': 'mph', 'to': 'km/h'})
```

### Currency

Provide your own exchange rates for currency conversion:

```python
result = add.transform('@harmonize', df, columns='price',
    strategy={'from': 'USD', 'to': 'EUR', 'rate': 0.92})
```

!!! note "No built-in exchange rates"
    Currency conversion requires a user-provided `rate` in the strategy. Additory does not fetch live exchange rates (no internet access by design).

---

## Multiple Columns

Convert several columns at once:

```python
df = pl.DataFrame({
    'height_in': [68.0, 72.0, 65.0],
    'weight_lbs': [150.0, 180.0, 130.0],
})

result = add.transform('@harmonize', df, columns=['height_in', 'weight_lbs'],
    strategy={'from': ['in', 'lbs'], 'to': ['cm', 'kg']})
```

---

## Practical Scenarios

### International data standardization

```python
import additory as add
import polars as pl

us_data = pl.DataFrame({
    'city': ['New York', 'Chicago', 'Phoenix'],
    'temp_f': [75.0, 68.0, 105.0],
    'distance_mi': [12.5, 8.3, 15.7],
})

# Convert to metric
result = add.transform('@harmonize', us_data, columns='temp_f',
    strategy={'from': 'F', 'to': 'C'})
result = add.transform('@harmonize', result, columns='distance_mi',
    strategy={'from': 'mi', 'to': 'km'})
```

---

## Next Steps

- [@calc](calc.md) — calculate derived columns after conversion
- [@aggregate](aggregate.md) — summarize harmonized data
- [add.transform()](../functions/transform.md) — all 12 transform modes
