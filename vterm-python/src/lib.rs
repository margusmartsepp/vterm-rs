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
    fn spawn(&self, py: Python<'_>, title: String, command: Option<String>, timeout_ms: Option<u64>, max_lines: Option<u32>) -> PyResult<u32> {
        let req = SpawnArgs {
            title,
            command,
            timeout_ms,
            max_lines,
            visible: Some(false),
        };
        
        py.allow_threads(|| {
            self.rt.block_on(async {
                let res = self.client.execute(SkillCommand::Spawn(req)).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Spawn failed: {}", e)))?;
                Ok(res.id.unwrap_or(0))
            })
        })
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
}

/// A Python module implemented in Rust.
#[pymodule]
fn vterm_python(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VTermClient>()?;
    Ok(())
}
