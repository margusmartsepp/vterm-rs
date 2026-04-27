use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::sync::Arc;
use tokio::runtime::Runtime;
use vterm_rs::client::OrchestratorClient;
use vterm_rs::{SkillCommand, SpawnArgs};

#[pyclass]
struct VTermClient {
    rt: Arc<Runtime>,
    client: OrchestratorClient,
}

#[pymethods]
impl VTermClient {
    #[new]
    fn new() -> PyResult<Self> {
        let rt = Arc::new(Runtime::new().map_err(|e| PyRuntimeError::new_err(format!("Failed to create tokio runtime: {}", e)))?);
        
        let client = rt.block_on(async {
            OrchestratorClient::connect().await
        }).map_err(|e| PyRuntimeError::new_err(format!("Failed to connect to orchestrator: {}", e)))?;
        
        Ok(Self { rt, client })
    }

    /// Spawns a new terminal session. Returns the terminal ID.
    #[pyo3(signature = (title, command=None, timeout_ms=None, max_lines=None, visible=None))]
    fn spawn(&self, py: Python<'_>, title: String, command: Option<String>, timeout_ms: Option<u64>, max_lines: Option<u32>, visible: Option<bool>) -> PyResult<u32> {
        let args = SpawnArgs { title, command, timeout_ms, max_lines, visible };
        let cmd = SkillCommand::Spawn(args);
        
        py.allow_threads(|| {
            self.rt.block_on(async {
                let res = self.client.execute(cmd).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Spawn failed: {}", e)))?;
                Ok(res.id.unwrap_or(0))
            })
        })
    }

    /// Returns a Spawn command dictionary without executing it.
    #[pyo3(signature = (title, command=None, timeout_ms=None, max_lines=None, visible=None))]
    fn spawn_op(&self, py: Python<'_>, title: String, command: Option<String>, timeout_ms: Option<u64>, max_lines: Option<u32>, visible: Option<bool>) -> PyResult<PyObject> {
        let cmd = SkillCommand::Spawn(SpawnArgs { title, command, timeout_ms, max_lines, visible });
        pythonize::pythonize(py, &cmd).map_err(|e| PyRuntimeError::new_err(e.to_string())).map(|obj| obj.unbind())
    }

    /// Returns a Write command dictionary.
    fn write_op(&self, py: Python<'_>, id: u32, text: String) -> PyResult<PyObject> {
        let cmd = SkillCommand::ScreenWrite { id, text };
        pythonize::pythonize(py, &cmd).map_err(|e| PyRuntimeError::new_err(e.to_string())).map(|obj| obj.unbind())
    }

    /// Returns a Read command dictionary.
    fn read_op(&self, py: Python<'_>, id: u32) -> PyResult<PyObject> {
        let cmd = SkillCommand::ScreenRead { id };
        pythonize::pythonize(py, &cmd).map_err(|e| PyRuntimeError::new_err(e.to_string())).map(|obj| obj.unbind())
    }

    /// Returns a WaitUntil command dictionary.
    fn wait_until_op(&self, py: Python<'_>, id: u32, pattern: String, timeout_ms: u64) -> PyResult<PyObject> {
        let cmd = SkillCommand::WaitUntil { id, pattern, timeout_ms };
        pythonize::pythonize(py, &cmd).map_err(|e| PyRuntimeError::new_err(e.to_string())).map(|obj| obj.unbind())
    }

    /// Returns a Close command dictionary.
    fn close_op(&self, py: Python<'_>, id: u32) -> PyResult<PyObject> {
        let cmd = SkillCommand::ScreenClose { id: Some(id), target: "single".into() };
        pythonize::pythonize(py, &cmd).map_err(|e| PyRuntimeError::new_err(e.to_string())).map(|obj| obj.unbind())
    }

    /// Writes keystrokes or text to a terminal.
    fn write(&self, py: Python<'_>, id: u32, text: String) -> PyResult<()> {
        py.allow_threads(|| {
            self.rt.block_on(async {
                self.client.execute(SkillCommand::ScreenWrite { id, text }).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Write failed: {}", e)))?;
                Ok(())
            })
        })
    }

    /// Reads the current contents of the terminal screen.
    fn read(&self, py: Python<'_>, id: u32) -> PyResult<String> {
        py.allow_threads(|| {
            self.rt.block_on(async {
                let res = self.client.execute(SkillCommand::ScreenRead { id }).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Read failed: {}", e)))?;
                Ok(res.content.unwrap_or_default())
            })
        })
    }

    /// Waits until a regex pattern appears on the screen.
    fn wait_until(&self, py: Python<'_>, id: u32, pattern: String, timeout_ms: u64) -> PyResult<String> {
        py.allow_threads(|| {
            self.rt.block_on(async {
                let res = self.client.execute(SkillCommand::WaitUntil { id, pattern, timeout_ms }).await
                    .map_err(|e| PyRuntimeError::new_err(format!("WaitUntil failed: {}", e)))?;
                Ok(res.content.unwrap_or_default())
            })
        })
    }

    /// Closes a terminal session.
    fn close(&self, py: Python<'_>, id: u32) -> PyResult<()> {
        py.allow_threads(|| {
            self.rt.block_on(async {
                self.client.execute(SkillCommand::ScreenClose { id: Some(id), target: "single".into() }).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Close failed: {}", e)))?;
                Ok(())
            })
        })
    }

    /// Low-level entry point for any SkillCommand. 
    /// Takes a dictionary and returns a dictionary.
    fn execute(&self, py: Python<'_>, command: Bound<'_, PyAny>) -> PyResult<PyObject> {
        let cmd: SkillCommand = pythonize::depythonize(&command)
            .map_err(|e| PyRuntimeError::new_err(format!("Invalid command format: {}", e)))?;
            
        let res = py.allow_threads(|| {
            self.rt.block_on(async {
                self.client.execute(cmd).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Execution failed: {}", e)))
            })
        })?;

        pythonize::pythonize(py, &res)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to pythonize result: {}", e)))
            .map(|obj| obj.into())
    }

    /// Batch multiple commands together.
    #[pyo3(signature = (commands, stop_on_error=None))]
    fn batch(&self, py: Python<'_>, commands: Vec<Bound<'_, PyAny>>, stop_on_error: Option<bool>) -> PyResult<PyObject> {
        let mut rust_cmds = Vec::with_capacity(commands.len());
        for c in commands {
            let cmd: SkillCommand = pythonize::depythonize(&c)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid command in batch: {}", e)))?;
            rust_cmds.push(cmd);
        }

        let batch_args = vterm_rs::BatchArgs {
            commands: rust_cmds,
            stop_on_error,
            visible: None,
        };

        let res = py.allow_threads(|| {
            self.rt.block_on(async {
                self.client.execute(SkillCommand::Batch(batch_args)).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Batch failed: {}", e)))
            })
        })?;

        pythonize::pythonize(py, &res)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to pythonize result: {}", e)))
            .map(|obj| obj.into())
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn vterm_python(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VTermClient>()?;
    Ok(())
}
