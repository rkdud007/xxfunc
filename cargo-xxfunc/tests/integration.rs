#[cfg(test)]
mod tests {
    use cargo_xxfunc::xxfunc;

    #[xxfunc]
    fn custom_function(data: &[u8]) -> i64 {
        data.iter().sum::<u8>() as i64
    }

    #[test]
    fn test_custom_function() {
        let data = [1, 2, 3, 2, 4, 5];
        let result = custom_function(data.as_ptr(), data.len());
        assert_eq!(result, 17);
    }
}
