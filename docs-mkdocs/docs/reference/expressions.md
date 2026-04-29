# Expression Catalog

All built-in expressions shipped with additory, organized by category. Use these with `add.transform('@calc', df, expression='inbuilt:<name>')` or the dynamic API `add.<name>(df)`.

---

## Core

General-purpose expressions for common calculations.

| Expression | Formula | Required Inputs | Output |
|-----------|---------|-----------------|--------|
| `bmi` | `weight / (height ** 2)` | weight (kg), height (m) | Body Mass Index |
| `bsa_mosteller` | `0.007184 * (height ** 0.725) * (weight ** 0.425)` | height (cm), weight (kg) | Body Surface Area (Mosteller) |
| `age_years` | `(today() - birth_date).days / 365.25` | birth_date | Age in years |
| `age_from_dob` | `(today() - date_of_birth).days / 365.25` | date_of_birth | Age in years |
| `profit` | `revenue - cost` | revenue, cost | Profit |
| `profit_margin_pct` | `(revenue - cost) / revenue * 100` | revenue, cost | Profit margin % |
| `total_price` | `price * quantity` | price, quantity | Total price |
| `discount_amount` | `price * discount_rate` | price, discount_rate | Discount amount |
| `net_price` | `price - (price * discount_rate)` | price, discount_rate | Net price after discount |

---

## Finance

Financial ratios, interest calculations, and profitability metrics.

| Expression | Formula | Required Inputs | Output |
|-----------|---------|-----------------|--------|
| `roi` | `((revenue - cost) / cost) * 100` | revenue, cost | Return on Investment % |
| `profit_margin` | `((revenue - cost) / revenue) * 100` | revenue, cost | Profit margin % |
| `gross_profit` | `revenue - cost_of_goods_sold` | revenue, cost_of_goods_sold | Gross profit |
| `net_profit` | `revenue - total_expenses` | revenue, total_expenses | Net profit |
| `break_even` | `fixed_costs / (price - variable_cost)` | fixed_costs, price, variable_cost | Break-even units |
| `markup` | `((selling_price - cost) / cost) * 100` | selling_price, cost | Markup % |
| `compound_interest` | `principal * ((1 + rate) ** periods)` | principal, rate, periods | Compound interest |
| `simple_interest` | `principal * rate * time` | principal, rate, time | Simple interest |
| `debt_to_equity` | `total_debt / total_equity` | total_debt, total_equity | Debt-to-equity ratio |
| `current_ratio` | `current_assets / current_liabilities` | current_assets, current_liabilities | Current ratio |

---

## Medical

Clinical formulas, vital sign calculations, and body composition metrics.

| Expression | Formula | Required Inputs | Output |
|-----------|---------|-----------------|--------|
| `heart_rate_max` | `220 - age` | age | Maximum heart rate |
| `heart_rate_reserve` | `heart_rate_max - resting_heart_rate` | heart_rate_max, resting_heart_rate | Heart rate reserve |
| `target_heart_rate_low` | `((heart_rate_max - resting_heart_rate) * 0.5) + resting_heart_rate` | heart_rate_max, resting_heart_rate | Target HR (50%) |
| `target_heart_rate_high` | `((heart_rate_max - resting_heart_rate) * 0.85) + resting_heart_rate` | heart_rate_max, resting_heart_rate | Target HR (85%) |
| `bmi_category` | `if_else(bmi < 18.5, 'underweight', ...)` | bmi | BMI category |
| `ideal_body_weight_male` | `50 + 0.91 * (height - 152.4)` | height (cm) | Ideal weight (male) |
| `ideal_body_weight_female` | `45.5 + 0.91 * (height - 152.4)` | height (cm) | Ideal weight (female) |
| `creatinine_clearance` | `((140 - age) * weight) / (72 * serum_creatinine)` | age, weight, serum_creatinine | Creatinine clearance (male) |
| `map` | `(systolic + 2 * diastolic) / 3` | systolic, diastolic | Mean Arterial Pressure |
| `pulse_pressure` | `systolic - diastolic` | systolic, diastolic | Pulse pressure |
| `bsa_dubois` | `0.007184 * (height ** 0.725) * (weight ** 0.425)` | height (cm), weight (kg) | BSA (DuBois) |
| `bsa_haycock` | `0.024265 * (height ** 0.3964) * (weight ** 0.5378)` | height (cm), weight (kg) | BSA (Haycock) |
| `bmr_male` | `10 * weight + 6.25 * height - 5 * age + 5` | weight (kg), height (cm), age | BMR (male, Mifflin-St Jeor) |
| `bmr_female` | `10 * weight + 6.25 * height - 5 * age - 161` | weight (kg), height (cm), age | BMR (female, Mifflin-St Jeor) |
| `waist_to_hip_ratio` | `waist_circumference / hip_circumference` | waist_circumference, hip_circumference | Waist-to-hip ratio |

---

## Physics

Mechanics, energy, waves, and fluid dynamics.

| Expression | Formula | Required Inputs | Output |
|-----------|---------|-----------------|--------|
| `velocity` | `distance / time` | distance, time | Velocity |
| `acceleration` | `(final_velocity - initial_velocity) / time` | final_velocity, initial_velocity, time | Acceleration |
| `force` | `mass * acceleration` | mass, acceleration | Force (N) |
| `kinetic_energy` | `0.5 * mass * (velocity ** 2)` | mass, velocity | Kinetic energy |
| `potential_energy` | `mass * 9.81 * height` | mass, height | Gravitational PE |
| `work` | `force * distance` | force, distance | Work |
| `momentum` | `mass * velocity` | mass, velocity | Momentum |
| `pressure` | `force / area` | force, area | Pressure |
| `density` | `mass / volume` | mass, volume | Density |
| `frequency` | `1 / period` | period | Frequency |
| `wavelength` | `speed / frequency` | speed, frequency | Wavelength |

---

## Chemistry

Stoichiometry, gas laws, and solution calculations.

| Expression | Formula | Required Inputs | Output |
|-----------|---------|-----------------|--------|
| `molarity` | `moles / volume_liters` | moles, volume_liters | Molarity (mol/L) |
| `moles_from_mass` | `mass / molar_mass` | mass, molar_mass | Moles |
| `mass_from_moles` | `moles * molar_mass` | moles, molar_mass | Mass |
| `dilution_concentration` | `initial_concentration * initial_volume / final_volume` | initial_concentration, initial_volume, final_volume | Final concentration |
| `percent_yield` | `(actual_yield / theoretical_yield) * 100` | actual_yield, theoretical_yield | Percent yield |
| `percent_composition` | `(element_mass / compound_mass) * 100` | element_mass, compound_mass | Percent composition |
| `ideal_gas_pressure` | `moles * 8.314 * temperature / volume` | moles, temperature, volume | Pressure (ideal gas) |
| `ideal_gas_volume` | `moles * 8.314 * temperature / pressure` | moles, temperature, pressure | Volume (ideal gas) |

---

## Engineering

Electrical, mechanical, and fluid engineering calculations.

| Expression | Formula | Required Inputs | Output |
|-----------|---------|-----------------|--------|
| `power_electrical` | `voltage * current` | voltage, current | Electrical power |
| `resistance` | `voltage / current` | voltage, current | Resistance (Ohm's law) |
| `current` | `voltage / resistance` | voltage, resistance | Current (Ohm's law) |
| `efficiency_pct` | `(output_power / input_power) * 100` | output_power, input_power | Efficiency % |
| `stress` | `force / area` | force, area | Stress |
| `strain` | `change_in_length / original_length` | change_in_length, original_length | Strain |
| `youngs_modulus` | `stress / strain` | stress, strain | Young's modulus |
| `flow_rate_volume` | `area * velocity` | area, velocity | Volumetric flow rate |
| `reynolds_number` | `density * velocity * diameter / viscosity` | density, velocity, diameter, viscosity | Reynolds number |
| `capacitance` | `charge / voltage` | charge, voltage | Capacitance |

---

## Statistics

Descriptive statistics, regression, and error metrics.

| Expression | Formula | Required Inputs | Output |
|-----------|---------|-----------------|--------|
| `z_score` | `(value - mean) / std_dev` | value, mean, std_dev | Z-score |
| `coefficient_of_variation` | `(std_dev / mean) * 100` | std_dev, mean | CV % |
| `standard_error` | `std_dev / (count ** 0.5)` | std_dev, count | Standard error |
| `percent_error` | `((actual - expected) / expected) * 100` | actual, expected | Percent error |
| `relative_error_pct` | `((actual - predicted) / actual) * 100` | actual, predicted | Relative error % |
| `slope` | `covariance / variance_x` | covariance, variance_x | Regression slope |
| `intercept` | `mean_y - slope * mean_x` | mean_y, slope, mean_x | Regression intercept |
| `predicted_value` | `intercept + slope * x` | intercept, slope, x | Predicted value |
| `residual` | `actual - predicted` | actual, predicted | Residual |
| `r_squared` | `correlation ** 2` | correlation | R² |

---

## Usage

### With @calc mode

```python
import additory as add
import polars as pl

df = pl.DataFrame({'weight': [70, 85], 'height': [1.70, 1.80]})
result = add.transform('@calc', df, expression='inbuilt:bmi', name='bmi')
```

### With the dynamic API

```python
result = add.bmi(df)
```

### With explicit column mapping

```python
df = pl.DataFrame({'mass': [70, 85], 'stature': [1.70, 1.80]})
result = add.bmi(df, weight='mass', height='stature')
```

---

## Adding Custom Expressions

See the [Expression Files](../guides/expression-files.md) guide for writing your own `.add` files.
