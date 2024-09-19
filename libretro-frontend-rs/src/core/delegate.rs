use libloading::Library;

pub struct CoreDelegate {
    corelib: Library,

}

impl CoreDelegate {
    pub fn new(corelib: Library) -> Self {
        Self {
            corelib,
        }
    }

}
