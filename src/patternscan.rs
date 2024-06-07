#[cfg(feature = "patternscan")]
use {crate::module::Module, anyhow::Result, patternscan::scan_first_match, std::io::Cursor};

#[cfg(feature = "patternscan")]
impl Module {
    pub fn find_pattern(&self, pattern: &str) -> Result<Option<usize>> {
        let module_data = unsafe { self.get_module_data() };
        let reader = Cursor::new(module_data);
        let first_match = scan_first_match(reader, pattern)?;
        Ok(first_match)
    }
}
