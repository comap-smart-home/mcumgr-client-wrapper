use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyBytes};
use mcumgr_client;
use mcumgr_client::SerialSpecs;
use serde_cbor::Value;
use std::path::PathBuf;

/// A session allows sending MCUmgr commands to a device over a serial port.
/// 
/// The serial port is configured during initialization of the Session. It stores the configuration
/// and manages the serial port.
/// 
/// Args:
///     device (str): Name of the device used for serial communication (/dev/ttyUSBx, COMx,
///     etc.).
///     baudrate (int): Baudrate of the serial device. Defaults to 115200.
///     initial_timeout_s (int): Timeout in seconds to receive a first response to a request.
///     Defaults to 60.
///     subsequent_timeout_ms (int): Timeout in milliseconds for the subsequent requests.
///     Defaults to 200.
///     nb_retry (int):
///     linelength (int):
///     mtu (int):
#[pyclass(module="mcumgr_client")]
struct SerialSession {
    specs: SerialSpecs
}

fn cbor_to_py(py: Python, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.into_py(py)),
        Value::Integer(num) => Ok(num.into_py(py)),
        Value::Float(num) => Ok(num.into_py(py)),
        Value::Bytes(bytes) => Ok(PyBytes::new_bound(py, bytes).into_py(py)),
        Value::Text(s) => Ok(s.into_py(py)),
        Value::Array(arr) => {
            let py_list = PyList::new_bound(py, arr.iter().map(|v| cbor_to_py(py, v)).collect::<Result<Vec<_>, _>>()?);
            Ok(py_list.into_py(py))
        },
        Value::Map(map) => {
            let py_dict = PyDict::new_bound(py);
            for (k, v) in map {
                let key = match k {
                    Value::Text(s) => s.clone(),
                    _ => return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Invalid map key")),
                };
                py_dict.set_item(key, cbor_to_py(py, v)?)?;
            }
            Ok(py_dict.into_py(py))
        }
        Value::Tag(_, boxed_value) => cbor_to_py(py, boxed_value),
        _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("unknown cbor type")),
    }
}

#[pymethods]
impl SerialSession {
    #[new]
    #[pyo3(signature = (device, baudrate=115200, initial_timeout_s=60, subsequent_timeout_ms=200,
        nb_retry=4, linelength=128, mtu=512))]
    fn new(
        device: &str,
        baudrate: u32,
        initial_timeout_s: u32,
        subsequent_timeout_ms: u32,
        nb_retry: u32,
        linelength: usize,
        mtu: usize
    ) -> Self {
        SerialSession {
            specs: SerialSpecs {
                device: device.to_string(),
                baudrate,
                initial_timeout_s,
                subsequent_timeout_ms,
                nb_retry,
                linelength,
                mtu,
            },
        }
    }

    /// List the properties of the images in a device.
    /// 
    /// Returns:
    ///     dict: A dictionnary containing the properties of the listed images
    /// 
    fn list(&self, py: Python) -> PyResult<PyObject> {
        let result = mcumgr_client::list(&self.specs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;

        let pydict = cbor_to_py(py, &result)?;

        Ok(pydict)
    }

    /// Reset a device
    fn reset(&self) -> PyResult<()> {
        mcumgr_client::reset(&self.specs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))
    }

    /// Upload an image on a device.
    ///
    /// Args:
    ///     filename (Union[str, bytes, os.PathLike]): Image to be uploaded on the device.
    ///     slot (int): Destination slot for the image.
    ///     progess (Optional[Callable[[int,int], None]]): a callable taking two integers
    ///     representing the current progress (offset, total size). This is called periodically
    ///     during the upload process.
    #[pyo3(signature = (filename, slot=0, progress=None))]
    fn upload(&self, py: Python, filename: &str, slot: u8, progress: Option<PyObject>) -> PyResult<()> {

        let path = PathBuf::from(filename);
        let callback = match progress {
            None => None,
            Some(pyfun) => {
                let pyfun = pyfun.clone_ref(py);
                
                Some(move |pos, total| {
                    Python::with_gil(|py| {
                        if let Err(e) = pyfun.call1(py, (pos, total)) {
                            e.print(py);
                        }
                    });
                })
            },
        };

        mcumgr_client::upload(&self.specs, &path, slot, callback)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))
    }
}


/// Python bindings for mcumgr-client, a rust library implementing MCUmgr protocol
/// 
/// Example:
/// ```
/// import mcumgr_client as mcu
/// 
/// s = mcu.SerialSession(device='/dev/ttyUSB0', baudrate=576000)
/// # Get a dictionnary of properties 
/// d = s.list()
/// print(d)
/// 
/// # Upload image to device
/// s.upload('/path/to/image/bin')
/// 
/// # Reset the device
/// s.reset()
/// ```
#[pymodule(name = "mcumgr_client")]
fn py_mcumgr_client(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SerialSession>()?;
    Ok(())
}
