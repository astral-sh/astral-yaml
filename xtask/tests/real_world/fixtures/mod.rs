#![allow(clippy::too_many_lines, reason = "Expected trees are inline snapshots")]

// Keep the module list and manifest coverage check in sync.
macro_rules! fixtures {
    ($($name:ident),* $(,)?) => {
        $(mod $name;)*

        pub(super) const NAMES: &[&str] = &[$(stringify!($name)),*];
    };
}

fixtures!(
    airflow_compose,
    black_diff_shades,
    cibuildwheel_test,
    compose_ci,
    cpython_build,
    gh_deployment,
    kubernetes_guestbook,
    openapi_petstore,
    ruff_ci,
    runner_release,
    sentry_compose,
    uv_ci,
);
