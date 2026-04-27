#[repr(C)]
struct LogRedactor {
    ptr : *mut LogRedactorInner,
}

struct LogRedactorInner {

}
#[unsafe(no_mangle)]
pub extern "C" fn init_log_redactor() -> LogRedactor {
    LogRedactor {
        ptr: Box::into_raw(Box::new(LogRedactorInner {}))
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn  drop_log_redactor(redactor: LogRedactor) {
    unsafe {
        Box::from_raw(redactor.ptr);
    }
}