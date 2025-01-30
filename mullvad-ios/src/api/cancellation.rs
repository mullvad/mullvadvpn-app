use tokio::task::JoinHandle;

use super::completion::SwiftCompletionHandler;


#[repr(C)]
pub struct SwiftCancelHandle {
    ptr: *mut RequestCancelHandle,
}

struct RequestCancelHandle {
    task: JoinHandle<()>,
    completion: SwiftCompletionHandler,
}


