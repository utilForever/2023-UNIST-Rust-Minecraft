use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

#[macro_export]
macro_rules! gl_call {
    ($fn_call:expr) => {
        if !cfg!(feature = "gl_debug") {
            unsafe { $fn_call }
        } else {
            let ret = unsafe { $fn_call };

            loop {
                let err = unsafe { gl::GetError() };

                match err {
                    gl::NO_ERROR => break,
                    _ => {
                        println!(
                            "{}:{}\t{} = {:?}",
                            file!(),
                            line!(),
                            stringify!($fn_call),
                            ret
                        );
                        println!("OpenGL Error: ");

                        let err = match err {
                            gl::INVALID_ENUM => "INVALID_ENUM".into(),
                            gl::INVALID_VALUE => "INVALID_VALUE".into(),
                            gl::INVALID_OPERATION => "INVALID_OPERATION".into(),
                            gl::STACK_OVERFLOW => "STACK_OVERFLOW".into(),
                            gl::STACK_UNDERFLOW => "STACK_UNDERFLOW".into(),
                            gl::OUT_OF_MEMORY => "OUT_OF_MEMORY".into(),
                            gl::INVALID_FRAMEBUFFER_OPERATION => {
                                "INVALID_FRAMEBUFFER_OPERATION".into()
                            }
                            _ => format!("code {}", err),
                        };

                        println!("{}", err);
                    }
                }
            }

            ret
        }
    };
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "system" fn debug_message_callback(
    source: u32,
    error_type: u32,
    id: u32,
    severity: u32,
    _length: i32,
    message: *const c_char,
    _user_param: *mut c_void,
) {
    let source = match source {
        gl::DEBUG_SOURCE_API => "Source: API".into(),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Source: Window System".into(),
        gl::DEBUG_SOURCE_SHADER_COMPILER => "Source: Shader Compiler".into(),
        gl::DEBUG_SOURCE_THIRD_PARTY => "Source: Third Party".into(),
        gl::DEBUG_SOURCE_APPLICATION => "Source: Application".into(),
        gl::DEBUG_SOURCE_OTHER => "Source: Other".into(),
        _ => format!("Source: *undocumented* {source}"),
    };
    let error_type = match error_type {
        gl::DEBUG_TYPE_ERROR => "Type: Error".into(),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Type: Deprecated Behaviour".into(),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Type: Undefined Behaviour".into(),
        gl::DEBUG_TYPE_PORTABILITY => "Type: Portability".into(),
        gl::DEBUG_TYPE_PERFORMANCE => "Type: Performance".into(),
        gl::DEBUG_TYPE_MARKER => "Type: Marker".into(),
        gl::DEBUG_TYPE_PUSH_GROUP => "Type: Push Group".into(),
        gl::DEBUG_TYPE_POP_GROUP => "Type: Pop Group".into(),
        gl::DEBUG_TYPE_OTHER => "Type: Other".into(),
        _ => format!("Type: *undocumented* {error_type}"),
    };
    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => "Severity: High".into(),
        gl::DEBUG_SEVERITY_MEDIUM => "Severity: Medium".into(),
        gl::DEBUG_SEVERITY_LOW => "Severity: Low".into(),
        gl::DEBUG_SEVERITY_NOTIFICATION => "Severity: Notification".into(),
        _ => format!("Severity: *undocumented* {severity}"),
    };

    eprintln!("---------------");
    unsafe {
        eprintln!(
            "Debug message ({id}): {}",
            CStr::from_ptr(message).to_str().unwrap()
        )
    };
    eprintln!("{source}");
    eprintln!("{error_type}");
    eprintln!("{severity}");
    eprintln!();
}
