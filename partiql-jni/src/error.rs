use jni::JNIEnv;
use partiql_eval::EngineError;

#[derive(Debug)]
pub enum JniError {
    InvalidHandle,
    EngineError(EngineError),
    Jni(jni::errors::Error),
}

impl From<EngineError> for JniError {
    fn from(err: EngineError) -> Self {
        JniError::EngineError(err)
    }
}

impl From<jni::errors::Error> for JniError {
    fn from(err: jni::errors::Error) -> Self {
        JniError::Jni(err)
    }
}

pub fn throw_jni_error(env: &mut JNIEnv<'_>, err: JniError) {
    let (exception_class, message) = match err {
        JniError::InvalidHandle => (
            "org/partiql/jni/exceptions/PartiQLException",
            "Invalid native handle".to_string(),
        ),
        JniError::EngineError(engine_err) => match engine_err {
            EngineError::TypeError(msg) => ("org/partiql/jni/exceptions/TypeException", msg),
            EngineError::NotImplemented => (
                "org/partiql/jni/exceptions/NotImplementedException",
                "Feature not implemented".to_string(),
            ),
            EngineError::IllegalState(msg) => {
                ("org/partiql/jni/exceptions/IllegalStateException", msg)
            }
            EngineError::InvalidPlan(msg) => ("org/partiql/jni/exceptions/PlanningException", msg),
            _ => (
                "org/partiql/jni/exceptions/PartiQLException",
                engine_err.to_string(),
            ),
        },
        JniError::Jni(jni_err) => (
            "org/partiql/jni/exceptions/PartiQLException",
            format!("JNI error: {}", jni_err),
        ),
    };

    let _ = env.throw_new(exception_class, message);
}

// Macro for exception handling in JNI functions that return a value
#[macro_export]
macro_rules! jni_guard {
    ($env:expr, $body:expr) => {
        match (|| -> Result<_, $crate::error::JniError> { $body })() {
            Ok(v) => v,
            Err(e) => {
                $crate::error::throw_jni_error(&mut $env, e);
                return Default::default();
            }
        }
    };
}

// Macro for exception handling in JNI functions that return void
#[macro_export]
macro_rules! jni_guard_void {
    ($env:expr, $body:expr) => {
        match { $body } {
            Ok(_) => {}
            Err(e) => {
                $crate::error::throw_jni_error(&mut $env, e);
            }
        }
    };
}
