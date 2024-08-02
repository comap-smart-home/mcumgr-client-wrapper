use mcumgr_client;
use mcumgr_client::SerialSpecs;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
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
#[pyclass(module = "mcumgr_client")]
struct SerialSession {
    specs: SerialSpecs,
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
        mtu: usize,
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

    #[pyo3(signature = (hash, confirm=None))]
    fn test(&self, hash: Vec<u8>, confirm: Option<bool>) -> PyResult<()> {
        mcumgr_client::test(&self.specs, hash, confirm)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))
    }

    #[pyo3(signature = (slot=None))]
    fn erase(&self, slot: Option<u32>) -> PyResult<()> {
        mcumgr_client::erase(&self.specs, slot)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))
    }

    /// List the properties of the images in a device.
    ///
    /// Returns:
    ///     list: A list of dicts containing the properties of the images
    ///
    fn list(&self, py: Python) -> PyResult<PyObject> {
        let result = mcumgr_client::list(&self.specs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;

        let py_list = PyList::empty_bound(py);

        for entry in &result.images {
            let py_dict = PyDict::new_bound(py);
            py_dict.set_item("image", entry.image.clone())?;
            py_dict.set_item("slot", entry.slot.clone())?;
            py_dict.set_item("version", entry.version.clone())?;
            py_dict.set_item("hash", entry.hash.clone())?;
            py_dict.set_item("bootable", entry.bootable)?;
            py_dict.set_item("pending", entry.pending)?;
            py_dict.set_item("confirmed", entry.confirmed)?;
            py_dict.set_item("active", entry.active)?;
            py_dict.set_item("permanent", entry.permanent)?;
            py_list.append(py_dict)?;
        }

        Ok(py_list.into())
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
    fn upload(
        &self,
        py: Python,
        filename: &str,
        slot: u8,
        progress: Option<PyObject>,
    ) -> PyResult<()> {
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
            }
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
