use crate::prelude::*;
use crate::unit::{CompositeUnit, CompositeUnitClass, Unit, UnitClass};
use crate::util::{ItemStorage, StorageHolder};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment {
    unit_classes: ItemStorage<UnitClass>,
    units: ItemStorage<Unit>,
    global_symbols: HashMap<Symbol, Value>,
}

/// Stuff to format values.
impl Environment {
    pub fn format_base_unit(&self, base_unit: &CompositeUnitClass) -> String {
        let mut numerator = "".to_owned();
        let mut denominator = "".to_owned();
        for (unit_class_id, power) in base_unit.components.iter() {
            assert!(*power != 0);
            if *power > 0 {
                numerator += &format!("{}^{}", self.borrow(*unit_class_id).0, power);
            } else {
                denominator += &format!("{}^{}", self.borrow(*unit_class_id).0, -power);
            }
        }
        format!("{} / {}", numerator, denominator)
    }

    pub fn format_unit(&self, unit: &CompositeUnit) -> String {
        let mut numerator = "".to_owned();
        let mut denominator = "".to_owned();
        for (unit_id, power) in unit.components.iter() {
            assert!(*power != 0);
            if *power > 0 {
                numerator += &format!("{}^{}", self.borrow(*unit_id).name, power);
            } else {
                denominator += &format!("{}^{}", self.borrow(*unit_id).name, -power);
            }
        }
        format!("{} / {}", numerator, denominator)
    }

    pub fn format_scalar_detailed(&self, scalar: &Scalar) -> String {
        let ratio = self.base_conversion_ratio_of(&scalar.display_unit);
        assert!(scalar.precision > 0);
        format!(
            "{1:.0$e} {2} ({3:.0$e} {4})",
            scalar.precision as usize - 1,
            scalar.base_value / ratio,
            self.format_unit(&scalar.display_unit),
            scalar.base_value,
            self.format_base_unit(&scalar.base_unit)
        )
    }

    pub fn format_value_detailed(&self, value: &Value) -> String {
        match value {
            Value::Scalar(scalar) => self.format_scalar_detailed(scalar),
            Value::Vector => unimplemented!(),
        }
    }

    pub fn format_formula_detailed(&self, formula: &Formula) -> String {
        self.format_formula_detailed_impl(formula, 0)
    }

    fn format_formula_detailed_impl(&self, formula: &Formula, indent: usize) -> String {
        match formula {
            Formula::Value(value) => self.format_value_detailed(value),
            Formula::PlainFunction { fun, args } => {
                let mut result = format!("{}[\n", fun.debug_name());
                for arg in args {
                    let arg = self.format_formula_detailed_impl(arg, indent + 4);
                    result += &format!("{0:>1$}{2},\n", "", indent + 4, arg);
                }
                result += &format!("{0:>1$}]", "", indent);
                result
            }
            Formula::Symbol(symbol) => format!("{:?}", symbol),
        }
    }
}

/// Non-formatting stuff.
impl Environment {
    pub fn new() -> Self {
        let mut result = Self {
            unit_classes: ItemStorage::new(),
            units: ItemStorage::new(),
            global_symbols: HashMap::new(),
        };
        crate::unit::add_default_units(&mut result);
        crate::constants::add_default_symbols(&mut result);
        result
    }

    /// Returns the base unit of the given unit. For example, Meters^2*Seconds^-1 will return
    /// Length^2*Time^-1. Hz*Area^-1 will return Time^-1*Length^-2.
    pub fn base_unit_of(&self, unit: &CompositeUnit) -> CompositeUnitClass {
        let mut complete_base = CompositeUnitClass::unitless();
        for (unit_id, power) in unit.components.iter() {
            let component_base = &self.borrow(*unit_id).base_class;
            assert!(*power != 0);
            if *power > 0 {
                for _ in 0..*power {
                    complete_base = complete_base * component_base.clone();
                }
            } else {
                for _ in 0..-*power {
                    complete_base = complete_base / component_base.clone();
                }
            }
        }
        complete_base
    }

    /// Calculate the ratio to convert a value of this unit to the base unit. Multiplying the value
    /// times this ratio will give the value expressed in terms of the base unit. Dividing a value
    /// expressed in the base unit by the ratio will give the value expressed in terms of the
    /// inputted unit.
    pub fn base_conversion_ratio_of(&self, unit: &CompositeUnit) -> f64 {
        let mut ratio = 1.0;
        for (unit_id, power) in unit.components.iter() {
            // E.G. if Feet^2 is a component, we will want the ratio to multiply values by
            // (Feet to Base Unit) * (Feet to Base Unit).
            ratio *= self.borrow(*unit_id).base_ratio.powi(*power);
        }
        ratio
    }

    pub fn make_scalar(&self, value: f64, unit: CompositeUnit, precision: u32) -> Scalar {
        let base_unit = self.base_unit_of(&unit);
        let base_value = value * self.base_conversion_ratio_of(&unit);
        Scalar::new(base_value, base_unit, unit, precision)
    }

    pub fn add_global_symbol(&mut self, symbol: Symbol, value: Value) {
        self.global_symbols.insert(symbol, value);
    }

    pub fn borrow_global_symbols(&self) -> SymbolTable<'_> {
        SymbolTable::new(&self.global_symbols)
    }

    pub fn find_global_symbol(&self, symbol: &Symbol) -> Option<&Value> {
        self.global_symbols.get(symbol)
    }
}

impl StorageHolder<UnitClass> for Environment {
    fn borrow_storage(&self) -> &ItemStorage<UnitClass> {
        &self.unit_classes
    }

    fn borrow_storage_mut(&mut self) -> &mut ItemStorage<UnitClass> {
        &mut self.unit_classes
    }
}

impl StorageHolder<Unit> for Environment {
    fn borrow_storage(&self) -> &ItemStorage<Unit> {
        &self.units
    }

    fn borrow_storage_mut(&mut self) -> &mut ItemStorage<Unit> {
        &mut self.units
    }
}
