//! This crate helps "store" and "retrieve" the current Java Virtual Machine in a safe way, plus it provides routines to help you get the JNIEnv for the current thread you're running on. This way, you set the JVM once and ask for the JNIEnv you need when you need it. Attaching and detaching from the JavaVM should be left to the library, it is none of your business :)
//! Note that you have to set the JavaVM once before you use the functions in this library. Failure to do that will make your program panic!
//! # Example
//! ```
//! javavm::set_jvm(None); // Pass a valid JavaVM instance here (hint: use the jni crate, which this crate already depends on)
//! // ... Other code goes here
//!
//! // When you need the JNIEnv
//! let env = javavm::get_env(); // Note: this will currently panic as we do not have a JavaVM set
//! ```

use jni::JNIEnv;
use jni::JavaVM;

static mut VM: Option<JavaVM> = None;

/// Sets the current JavaVM. All JNIEnv instances will come from this JavaVM
/// [Panic]
pub fn set_jvm(vm: Option<JavaVM>) {
    unsafe {
        VM = vm;
    }
}

/// Retrieve the current JVM as set by set_jvm
pub fn jvm() -> Option<&'static JavaVM> {
    if let Some(vm) = unsafe { VM.as_ref() } {
        return Some(vm);
    }

    None
}

/// Retrieves the current JNIEnv from the JavaVM.
/// # Panics
/// This function will panic if a JavaVM has not already been set
pub fn get_env() -> JNIEnv<'static> {
    let vm = unsafe { VM.as_ref().unwrap() };
    vm.attach_current_thread_as_daemon().unwrap()
}

/// Retrieves the current JNIEnv from the JavaVM.
/// Does not panic if there is no JavaVM currently set and returns None instead
pub fn get_env_safe() -> Option<JNIEnv<'static>> {
    let vm = unsafe { VM.as_ref()? };
    match vm.attach_current_thread_as_daemon() {
        Ok(env) => Some(env),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    #[should_panic]
    fn no_vm() {
        let _ = jvm().unwrap();
    }

    #[test]
    #[should_panic]
    fn no_env() {
        let _ = get_env();
    }

    #[test]
    fn no_env_safe() {
        assert!(get_env_safe().is_none());
    }
}
