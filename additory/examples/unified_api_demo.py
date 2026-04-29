"""
Unified API Demo

Demonstrates the new unified additory API with:
- Expression resolution via @calc
- @knn imputation
- Configuration management
"""

import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import additory as add
import polars as pl

def demo_expression_resolution():
    """Demo: Using builtin expressions via @calc"""
    print("\n" + "="*60)
    print("DEMO 1: Expression Resolution")
    print("="*60)
    
    # Create sample data
    df = pl.DataFrame({
        'name': ['Alice', 'Bob', 'Charlie'],
        'weight': [70.0, 85.0, 92.0],  # kg
        'height': [1.75, 1.80, 1.68],  # meters
    })
    
    print("\nInput DataFrame:")
    print(df)
    
    # Use builtin expression (if Rust bindings available)
    if add.RUST_AVAILABLE:
        print("\n✓ Rust bindings available")
        print("Calculating BMI using 'inbuilt:bmi' expression...")
        
        try:
            result = add.transform('@calc', df, expression='inbuilt:bmi', as='bmi')
            print("\nResult:")
            print(result)
        except Exception as e:
            print(f"\n✗ Error: {e}")
    else:
        print("\n✗ Rust bindings not available")
        print("Expression resolution requires Rust bindings")


def demo_knn_imputation():
    """Demo: KNN imputation via @knn mode"""
    print("\n" + "="*60)
    print("DEMO 2: KNN Imputation")
    print("="*60)
    
    # Create sample data with missing values
    df = pl.DataFrame({
        'name': ['Alice', 'Bob', 'Charlie', 'David', 'Eve'],
        'age': [25, None, 35, 40, 28],
        'salary': [50000, 60000, None, 80000, 55000],
    })
    
    print("\nInput DataFrame (with missing values):")
    print(df)
    
    # Use @knn imputation
    print("\nPerforming KNN imputation (k=2)...")
    
    try:
        result = add.transform('@knn', df, 
            fetch=['age', 'salary'],
            strategy={'k': 2, 'weights': 'distance', 'metric': 'euclidean'}
        )
        print("\nResult (imputed):")
        print(result)
    except Exception as e:
        print(f"\n✗ Error: {e}")


def demo_configuration():
    """Demo: Configuration management"""
    print("\n" + "="*60)
    print("DEMO 3: Configuration Management")
    print("="*60)
    
    # Set configuration
    print("\nSetting expressions folder...")
    add.set(expressions='/path/to/my_expressions')
    
    # Get configuration
    folder = add.get('expressions')
    print(f"Expressions folder: {folder}")
    
    namespace = add.get('expressions_namespace')
    print(f"Expressions namespace: {namespace}")
    
    # Enable logging
    print("\nEnabling logging...")
    add.set(logging=True)
    
    logging_enabled = add.get('logging')
    print(f"Logging enabled: {logging_enabled}")


def demo_pandas_support():
    """Demo: Pandas DataFrame support"""
    print("\n" + "="*60)
    print("DEMO 4: Pandas DataFrame Support")
    print("="*60)
    
    import pandas as pd
    
    # Create pandas DataFrame
    df_pandas = pd.DataFrame({
        'name': ['Alice', 'Bob', 'Charlie'],
        'age': [25, None, 35],
        'salary': [50000, 60000, None],
    })
    
    print("\nInput DataFrame (pandas):")
    print(df_pandas)
    print(f"Type: {type(df_pandas)}")
    
    # Use @knn imputation
    print("\nPerforming KNN imputation...")
    
    try:
        result = add.transform('@knn', df_pandas, 
            fetch=['age', 'salary'],
            strategy={'k': 2}
        )
        print("\nResult (imputed):")
        print(result)
        print(f"Type: {type(result)}")
        print("\n✓ Result is same type as input (pandas)")
    except Exception as e:
        print(f"\n✗ Error: {e}")


def main():
    """Run all demos"""
    print("\n" + "="*60)
    print("ADDITORY UNIFIED API DEMO")
    print("="*60)
    print(f"\nVersion: {add.__version__}")
    print(f"Rust bindings available: {add.RUST_AVAILABLE}")
    
    # Run demos
    demo_expression_resolution()
    demo_knn_imputation()
    demo_configuration()
    demo_pandas_support()
    
    print("\n" + "="*60)
    print("DEMO COMPLETE")
    print("="*60)


if __name__ == '__main__':
    main()
