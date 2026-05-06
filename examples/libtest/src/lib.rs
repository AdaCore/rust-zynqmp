use adacore_zynqmp as _;

#[cfg(test)]
mod tests {
    #[test]
    fn vec_grows_across_reallocations() {
        let mut values = Vec::new();
        for value in 0..1000 {
            values.push(value);
        }
        assert_eq!(values.len(), 1000);
        assert_eq!(values[500], 500);
    }

    #[test]
    fn hashmap_insert_and_lookup() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("hello", 1);
        map.insert("world", 2);
        assert_eq!(map.get("hello"), Some(&1));
        assert_eq!(map.len(), 2);
    }
}
