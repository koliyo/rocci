//! Probe `handle` modes for measuring yield-around-Roc vs nested C occupancy.

use std::future::Future;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

/// Stand-in routes for the three occupancy classes in the WASI HTTP plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeMode {
    /// Adapter `await` of clocks (SSE Wait analog). Yields the instance.
    AdapterAwait,
    /// `extern "C"` CPU-only stub standing in for `roc_respond_for_host`.
    CpuC,
    /// `extern "C"` hosted sleep standing in for `hosted_sleep_millis`.
    HostedSleepC,
}

impl ProbeMode {
    pub fn from_path(path: &str) -> Option<Self> {
        match path {
            "/adapter-await" => Some(Self::AdapterAwait),
            "/cpu-c" => Some(Self::CpuC),
            "/hosted-sleep-c" => Some(Self::HostedSleepC),
            _ => None,
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Self::AdapterAwait => "/adapter-await",
            Self::CpuC => "/cpu-c",
            Self::HostedSleepC => "/hosted-sleep-c",
        }
    }

    fn wasm_export(self) -> &'static str {
        match self {
            Self::AdapterAwait => "adapter_await",
            Self::CpuC => "cpu_c",
            Self::HostedSleepC => "hosted_sleep",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeRequest {
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeResponse {
    pub status: u16,
}

/// Wall-clock overlap of two concurrent `handle` calls on one current-thread runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlapReport {
    pub wait: Duration,
    pub wall: Duration,
    pub overlapped: bool,
}

impl OverlapReport {
    pub fn from_wall(wait: Duration, wall: Duration) -> Self {
        Self {
            wait,
            wall,
            overlapped: wall < wait + wait / 2,
        }
    }
}

/// One probe request: wait ~`wait`, then 200.
pub async fn handle_probe(request: &ProbeRequest, wait: Duration) -> Result<ProbeResponse> {
    let Some(mode) = ProbeMode::from_path(&request.path) else {
        return Ok(ProbeResponse { status: 404 });
    };
    run_mode(mode, wait).await;
    Ok(ProbeResponse { status: 200 })
}

async fn run_mode(mode: ProbeMode, wait: Duration) {
    match mode {
        ProbeMode::AdapterAwait => tokio::time::sleep(wait).await,
        ProbeMode::CpuC => busy_ms(wait),
        ProbeMode::HostedSleepC => std::thread::sleep(wait),
    }
}

fn busy_ms(wait: Duration) {
    let start = Instant::now();
    while start.elapsed() < wait {
        std::hint::black_box(start.elapsed());
    }
}

/// Overlap two native probe handles on the caller's runtime (use current-thread).
pub async fn overlap_native(mode: ProbeMode, wait: Duration) -> OverlapReport {
    let path = mode.path().to_string();
    overlap_two(wait, || {
        let path = path.clone();
        async move {
            let request = ProbeRequest { path };
            handle_probe(&request, wait).await.expect("probe handle");
        }
    })
    .await
}

async fn overlap_two<F, Fut>(wait: Duration, mut run: F) -> OverlapReport
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ()>,
{
    let first = run();
    let second = run();
    let start = Instant::now();
    tokio::join!(first, second);
    OverlapReport::from_wall(wait, start.elapsed())
}

const PROBE_WAT: &str = r#"
(module
  (import "host" "async_sleep_ms" (func $async_sleep (param i32)))
  (import "host" "sync_sleep_ms" (func $sync_sleep (param i32)))
  (import "host" "busy_ms" (func $busy (param i32)))
  (func (export "adapter_await") (param $ms i32)
    (call $async_sleep (local.get $ms)))
  (func (export "cpu_c") (param $ms i32)
    (call $busy (local.get $ms)))
  (func (export "hosted_sleep") (param $ms i32)
    (call $sync_sleep (local.get $ms)))
)
"#;

/// Overlap two Wasmtime guest exports that call host imports.
///
/// `AdapterAwait` uses `func_wrap_async` (fiber park). `CpuC` and `HostedSleepC`
/// use sync imports. Two Stores share one current-thread host runtime, matching
/// one OS thread multiplexing two instances / tasks.
fn wasmtime_err(err: wasmtime::Error) -> anyhow::Error {
    anyhow::anyhow!("{err:#}")
}

pub async fn overlap_wasmtime(mode: ProbeMode, wait: Duration) -> Result<OverlapReport> {
    let wasm = wat::parse_str(PROBE_WAT).context("parse probe WAT")?;
    let engine = wasmtime::Engine::new(&wasmtime::Config::new()).map_err(wasmtime_err)?;
    let module = wasmtime::Module::new(&engine, &wasm).map_err(wasmtime_err)?;
    let linker = probe_linker(&engine)?;
    let ms = u32::try_from(wait.as_millis()).unwrap_or(u32::MAX);
    let export = mode.wasm_export();

    let first = call_probe_export(&engine, &module, &linker, export, ms);
    let second = call_probe_export(&engine, &module, &linker, export, ms);
    let start = Instant::now();
    let (a, b) = tokio::join!(first, second);
    a?;
    b?;
    Ok(OverlapReport::from_wall(wait, start.elapsed()))
}

fn probe_linker(engine: &wasmtime::Engine) -> Result<wasmtime::Linker<()>> {
    let mut linker = wasmtime::Linker::new(engine);
    linker
        .func_wrap_async(
            "host",
            "async_sleep_ms",
            |_caller: wasmtime::Caller<'_, ()>, (ms,): (i32,)| {
                Box::new(async move {
                    tokio::time::sleep(Duration::from_millis(ms.max(0) as u64)).await;
                })
            },
        )
        .map_err(wasmtime_err)?;
    linker
        .func_wrap("host", "sync_sleep_ms", |ms: i32| {
            std::thread::sleep(Duration::from_millis(ms.max(0) as u64));
        })
        .map_err(wasmtime_err)?;
    linker
        .func_wrap("host", "busy_ms", |ms: i32| {
            busy_ms(Duration::from_millis(ms.max(0) as u64));
        })
        .map_err(wasmtime_err)?;
    Ok(linker)
}

async fn call_probe_export(
    engine: &wasmtime::Engine,
    module: &wasmtime::Module,
    linker: &wasmtime::Linker<()>,
    export: &str,
    ms: u32,
) -> Result<()> {
    let mut store = wasmtime::Store::new(engine, ());
    let instance = linker
        .instantiate_async(&mut store, module)
        .await
        .map_err(wasmtime_err)
        .with_context(|| format!("instantiate probe for {export}"))?;
    let func = instance
        .get_typed_func::<i32, ()>(&mut store, export)
        .map_err(wasmtime_err)
        .with_context(|| format!("missing export {export}"))?;
    func.call_async(&mut store, ms as i32)
        .await
        .map_err(wasmtime_err)
        .with_context(|| format!("call {export}"))?;
    Ok(())
}

/// Print a 200ms overlap table for research (run with `--ignored` or as a binary helper).
pub async fn measure_table(wait: Duration) -> Result<String> {
    let mut lines = vec![format!(
        "wait={}ms (current-thread runtime; two concurrent handles)",
        wait.as_millis()
    )];
    for mode in [
        ProbeMode::AdapterAwait,
        ProbeMode::CpuC,
        ProbeMode::HostedSleepC,
    ] {
        let native = overlap_native(mode, wait).await;
        let wasm = overlap_wasmtime(mode, wait).await?;
        lines.push(format!(
            "{mode:?}: native wall={}ms overlapped={}; wasmtime wall={}ms overlapped={}",
            native.wall.as_millis(),
            native.overlapped,
            wasm.wall.as_millis(),
            wasm.overlapped
        ));
    }
    Ok(lines.join("\n"))
}

pub fn confirm_policy(native: &[(ProbeMode, OverlapReport)]) -> Result<()> {
    for (mode, report) in native {
        match mode {
            ProbeMode::AdapterAwait => {
                if !report.overlapped {
                    bail!("adapter-await should overlap concurrent handles: {report:?}");
                }
            }
            ProbeMode::CpuC | ProbeMode::HostedSleepC => {
                if report.overlapped {
                    bail!("{mode:?} nested C should serialize other handles: {report:?}");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const WAIT: Duration = Duration::from_millis(40);

    #[tokio::test(flavor = "current_thread")]
    async fn unknown_path_is_404() {
        let response = handle_probe(
            &ProbeRequest {
                path: "/nope".into(),
            },
            WAIT,
        )
        .await
        .unwrap();
        assert_eq!(response.status, 404);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn known_routes_return_200() {
        for mode in [
            ProbeMode::AdapterAwait,
            ProbeMode::CpuC,
            ProbeMode::HostedSleepC,
        ] {
            let response = handle_probe(
                &ProbeRequest {
                    path: mode.path().into(),
                },
                Duration::from_millis(1),
            )
            .await
            .unwrap();
            assert_eq!(response.status, 200, "{}", mode.path());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn adapter_await_overlaps_native() {
        let report = overlap_native(ProbeMode::AdapterAwait, WAIT).await;
        assert!(report.overlapped, "adapter-await should yield: {report:?}");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cpu_c_serializes_native() {
        let report = overlap_native(ProbeMode::CpuC, WAIT).await;
        assert!(
            !report.overlapped,
            "CPU-C should occupy the instance: {report:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn hosted_sleep_c_serializes_native() {
        let report = overlap_native(ProbeMode::HostedSleepC, WAIT).await;
        assert!(
            !report.overlapped,
            "hosted-sleep C should block the host thread: {report:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn adapter_await_overlaps_wasmtime() {
        let report = overlap_wasmtime(ProbeMode::AdapterAwait, WAIT)
            .await
            .unwrap();
        assert!(
            report.overlapped,
            "async host import should park the guest fiber: {report:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cpu_c_serializes_wasmtime() {
        let report = overlap_wasmtime(ProbeMode::CpuC, WAIT).await.unwrap();
        assert!(
            !report.overlapped,
            "sync busy import should serialize: {report:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn hosted_sleep_c_serializes_wasmtime() {
        let report = overlap_wasmtime(ProbeMode::HostedSleepC, WAIT)
            .await
            .unwrap();
        assert!(
            !report.overlapped,
            "sync thread::sleep import does not apply fibers: {report:?}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn policy_matches_native_table() {
        let table = [
            (
                ProbeMode::AdapterAwait,
                overlap_native(ProbeMode::AdapterAwait, WAIT).await,
            ),
            (ProbeMode::CpuC, overlap_native(ProbeMode::CpuC, WAIT).await),
            (
                ProbeMode::HostedSleepC,
                overlap_native(ProbeMode::HostedSleepC, WAIT).await,
            ),
        ];
        confirm_policy(&table).unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    #[ignore = "200ms timings for the research table; run during Phase 0"]
    async fn measure_200ms_table() {
        let table = measure_table(Duration::from_millis(200)).await.unwrap();
        eprintln!("{table}");
    }
}
