mod socket;

use pyo3::prelude::*;
use crate::socket::create_udp_socket;
use std::net::UdpSocket;

#[pyclass]
pub struct PsWrapper {
    target_addr: String,
    is_sender: bool,
    msg_type: PyObject,
    sock: UdpSocket,
}

#[pymethods]
impl PsWrapper {
    #[new]
    fn new(py: Python, target_ip: &str, port: u16, msg_type: PyObject, is_sender: Option<bool>) -> PyResult<Self> {
        let is_sender = is_sender.unwrap_or(true);
        let addr = format!("{}:{}", target_ip, port);

        let sock = if is_sender {
            create_udp_socket(192, format!("0.0.0.0")).unwrap()
        } else {
            UdpSocket::bind(addr.clone()).unwrap()
        };
        
        if is_sender {
            sock.set_nonblocking(true).unwrap();
        }
        else {
            sock.set_nonblocking(false).unwrap();
        }
        
        Ok(PsWrapper {
            target_addr: addr,
            is_sender,
            msg_type,
            sock: sock,
        })
    }

    fn to_message(&self, py: Python, data_bytes: &[u8]) -> PyResult<PyObject> {
        let msg_type = self.msg_type.as_ref(py).downcast::<pyo3::types::PyType>()?;
            if msg_type.name()? == "Int8" {
                let value = i8::from_be_bytes([data_bytes[0]]);
                Ok(value.to_object(py))
            } else if msg_type.name()? == "Float32MultiArray" {
                let data_len = u32::from_be_bytes(data_bytes[0..4].try_into().unwrap()) as usize;
                let mut data = Vec::with_capacity(data_len);
                for i in 0..data_len {
                    let start = 4 + i * 4;
                    let end = start + 4;
                    let value = f32::from_be_bytes(data_bytes[start..end].try_into().unwrap());
                    data.push(value);
                }
                Ok(data.to_object(py))
            } else {
                Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Unhandled msg_type {} with data: {:?}",
                    msg_type.name()?,
                    data_bytes
                )))
            }
    } 

    fn to_bytes(&self, py: Python, data: PyObject) -> PyResult<Vec<u8>> {
        let msg_type = self.msg_type.as_ref(py).downcast::<pyo3::types::PyType>()?;
        if msg_type.name()? == "Float32MultiArray" {
            let data: Vec<f32> = data.extract(py)?;
            let mut data_bytes = Vec::with_capacity(4 + data.len() * 4);
            data_bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
            for value in data {
                data_bytes.extend_from_slice(&value.to_be_bytes());
            }
            Ok(data_bytes)
        } else if msg_type.name()? == "Int8" {
            let value: i8 = data.extract(py)?;
            Ok(vec![value.to_be_bytes()[0]])
        } else {
            Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unhandled msg_type {} with data: {:?}",
                msg_type.name()?,
                data
            )))
        }
    }

    pub fn publish(&self, py: Python, message: PyObject) -> PyResult<()> {
        if self.is_sender {
            let data_bytes = self.to_bytes(py, message)?;
            self.sock.send_to(&data_bytes, self.target_addr.clone())?;
        } else {
            println!("[WARNING]: configured to subscriber, will not publish");
        }
        Ok(())
    }

    pub fn listen(&self, py: Python) -> PyResult<PyObject> {
        let mut buf = [0; 100];
        loop {
            if let Ok((len, _src_addr)) = self.sock.recv_from(&mut buf) {
                return self.to_message(py, &buf[..len]);
            }
        }
    }
}

#[pymodule]
fn ps_wrapper(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PsWrapper>()?;
    Ok(())
}