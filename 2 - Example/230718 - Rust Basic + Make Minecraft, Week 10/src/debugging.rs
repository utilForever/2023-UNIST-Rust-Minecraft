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
