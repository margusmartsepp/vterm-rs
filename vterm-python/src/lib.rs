use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use std::sync::Arc;
use tokio::runtime::Runtime;
use vterm_rs::client::OrchestratorClient;
use vterm_protocol::{SkillCommand, SpawnArgs, BatchArgs};
use std::collections::HashMap;

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

    /// Spawns a new terminal session. Returns the terminal ID (or full result if wait=true).
    #[pyo3(signature = (title, command=None, timeout_ms=None, max_lines=None, visible=None, cols=None, rows=None, env=None, wait=None, semantic=None, extract_pattern=None, callback=None))]
    fn spawn(&self, py: Python<'_>, title: String, command: Option<String>, timeout_ms: Option<u64>, max_lines: Option<u32>, visible: Option<bool>, cols: Option<u16>, rows: Option<u16>, env: Option<HashMap<String, String>>, wait: Option<bool>, semantic: Option<bool>, extract_pattern: Option<String>, callback: Option<PyObject>) -> PyResult<PyObject> {
        let args = SpawnArgs { title, command, timeout_ms, max_lines, visible, cols, rows, env, wait, semantic, extract_pattern };
        let cmd = SkillCommand::Spawn(args);
        
        let res = py.allow_threads(|| {
            self.rt.block_on(async {
                self.client.execute(cmd).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Spawn failed: {}", e)))
            })
        })?;

        // If a callback is provided, run it on the result and serialize to JSON
        if let Some(cb) = callback {
            if !cb.bind(py).is_callable() {
                return Err(pyo3::exceptions::PyTypeError::new_err("Provided callback is not callable"));
            }

            let py_res = pythonize::pythonize(py, &res).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
            let cb_res = match cb.call1(py, (py_res,)) {
                Ok(result) => result,
                Err(e) => {
                    e.print_and_set_sys_last_vars(py);
                    return Err(pyo3::exceptions::PyRuntimeError::new_err("Python callback execution failed"));
                }
            };

            // Serialize the callback result to JSON using Python's native json module
            let json_module = py.import_bound("json")?;
            let json_str: String = json_module
                .call_method1("dumps", (cb_res,))?
                .extract()?;
            
            return Ok(json_str.into_py(py));
        }

        if wait.unwrap_or(false) {
            pythonize::pythonize(py, &res).map_err(|e| PyRuntimeError::new_err(e.to_string())).map(|obj| obj.unbind())
        } else {
            Ok(res.id.unwrap_or(0).into_py(py))
        }
    }

    /// Returns version and build metadata for this vterm-rs instance.
    #[pyo3(signature = (assurance=false))]
    fn get_info(&self, py: Python<'_>, assurance: bool) -> PyResult<PyObject> {
        let res = py.allow_threads(|| {
            self.rt.block_on(async {
                self.client.execute(SkillCommand::Inspect { assurance }).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Get info failed: {}", e)))
            })
        })?;
        pythonize::pythonize(py, &res).map_err(|e| PyRuntimeError::new_err(e.to_string())).map(|obj| obj.unbind())
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
    #[pyo3(signature = (id, history=false))]
    fn read(&self, py: Python<'_>, id: u32, history: bool) -> PyResult<String> {
        py.allow_threads(|| {
            self.rt.block_on(async {
                let res = self.client.execute(SkillCommand::ScreenRead { id, history }).await
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

    /// Blocks until the screen buffer remains unchanged for `stable_ms`.
    fn wait_until_stable(&self, py: Python<'_>, id: u32, stable_ms: u64, timeout_ms: u64) -> PyResult<String> {
        py.allow_threads(|| {
            self.rt.block_on(async {
                let res = self.client.execute(SkillCommand::WaitUntilStable { id, stable_ms, timeout_ms }).await
                    .map_err(|e| PyRuntimeError::new_err(format!("WaitUntilStable failed: {}", e)))?;
                Ok(res.content.unwrap_or_default())
            })
        })
    }

    /// Returns only the visual delta since the last observation.
    fn screen_diff(&self, py: Python<'_>, id: u32) -> PyResult<String> {
        py.allow_threads(|| {
            self.rt.block_on(async {
                let res = self.client.execute(SkillCommand::ScreenDiff { id }).await
                    .map_err(|e| PyRuntimeError::new_err(format!("ScreenDiff failed: {}", e)))?;
                Ok(res.content.unwrap_or_default())
            })
        })
    }

    /// Extracts structured data from terminal history using a regex with named groups.
    #[pyo3(signature = (id, pattern, history=true))]
    fn extract(&self, py: Python<'_>, id: u32, pattern: String, history: bool) -> PyResult<PyObject> {
        let cmd = SkillCommand::Extract { id, pattern, history };
        let res = py.allow_threads(|| {
            self.rt.block_on(async {
                self.client.execute(cmd).await
                    .map_err(|e| PyRuntimeError::new_err(format!("Extract failed: {}", e)))
            })
        })?;
        
        pythonize::pythonize(py, &res.extracted.unwrap_or_default())
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to pythonize result: {}", e)))
            .map(|obj| obj.into())
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
    #[pyo3(signature = (commands, stop_on_error=None, parallel=None, visible=None))]
    fn batch(&self, py: Python<'_>, commands: Vec<Bound<'_, PyAny>>, stop_on_error: Option<bool>, parallel: Option<bool>, visible: Option<bool>) -> PyResult<PyObject> {
        let mut rust_cmds = Vec::with_capacity(commands.len());
        for c in commands {
            let cmd: SkillCommand = pythonize::depythonize(&c)
                .map_err(|e| PyRuntimeError::new_err(format!("Invalid command in batch: {}", e)))?;
            rust_cmds.push(cmd);
        }

        let batch_args = BatchArgs {
            commands: rust_cmds,
            stop_on_error,
            parallel,
            visible,
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

#[pymodule]
fn vterm_python(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VTermClient>()?;
    Ok(())
}
