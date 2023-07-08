//! Implementation of JavaScript iteration order

use std::str::FromStr;

use indexmap::IndexMap;

/// Get the keys in the order that JavaScript would iterate over them.  
/// The key being generic allows you to do things like `Rc<str>` and the like to avoid expensive
/// cloning.
///   
/// https://tc39.es/ecma262/#sec-ordinaryownpropertykeys
pub fn js_iter_order<'a, S, T>(data: &'a IndexMap<S, T>) -> Vec<S>
where
    S: Clone + AsRef<str>,
{
    // let mut order = Vec::with_capacity(data.len());

    let mut number_order = Vec::new();
    let mut string_order = Vec::new();

    for key in data.keys() {
        if let Ok(number) = usize::from_str(key.as_ref()) {
            // TODO: this is slightly wrong as it should allow +0 and a smaller range for u32s
            number_order.push((number, key));
        } else {
            string_order.push(key);
        }
    }

    // We don't need to handle symbols

    number_order.sort_by_key(|(number, _)| *number);

    let mut order = Vec::with_capacity(data.len());

    order.extend(number_order.into_iter().map(|(_, key)| key.clone()));
    order.extend(string_order.into_iter().cloned());

    order
}
