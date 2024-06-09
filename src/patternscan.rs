use {crate::module::Module, anyhow::Result, patternscan::scan_first_match, std::io::Cursor};

fn scan_data_for_pattern(data: Vec<u8>, pattern: &str) -> Result<Option<usize>> {
    let reader = Cursor::new(data);
    let first_match = scan_first_match(reader, pattern)?;
    Ok(first_match)
}

impl Module {
    #[cfg(feature = "external")]
    pub fn find_pattern(&self, pattern: &str) -> Result<Option<usize>> {
        let module_data = self.get_module_data()?;
        scan_data_for_pattern(module_data, pattern)
    }

    #[cfg(feature = "internal")]
    pub fn find_pattern(&self, pattern: &str) -> Result<Option<usize>> {
        let module_data = unsafe { self.get_module_data() };
        scan_data_for_pattern(module_data, pattern)
    }
}
