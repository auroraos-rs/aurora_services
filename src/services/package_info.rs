pub fn package_name() -> Option<String> {
    std::env::var("AURORA_PACKAGE_NAME").ok()
}

pub fn app_id() -> Option<String> {
    std::env::var("AURORA_APP_ID").ok()
}

pub fn app_instance_id() -> Option<String> {
    std::env::var("AURORA_APP_INSTANCE_ID").ok()
}

pub fn runtime_dir() -> Option<String> {
    std::env::var("XDG_RUNTIME_DIR").ok()
}
