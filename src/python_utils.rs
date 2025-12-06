use pyo3::ffi::c_str;
use pyo3::prelude::*;

pub fn enumerate_cameras() -> Result<Vec<(i32, String)>, PyErr> {
    Python::attach(|py| {
        let python_ffi = PyModule::from_code(
            py,
            c_str!(include_str!("python-utils.py")),
            c"python-utils.py",
            c"python-utils",
        )?;
        let r = python_ffi.getattr("enumerate_cameras")?.call1(())?;
        r.extract::<Vec<(i32, String)>>()
    })
}
