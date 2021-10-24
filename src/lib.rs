//! This crate helps "store" and "retrieve" the current Java Virtual Machine in a safe way, plus it provides routines to help you get the JNIEnv for the current thread you're running on. This way, you set the JVM once and ask for the JNIEnv you need when you need it. Attaching and detaching from the JavaVM should be left to the library, it is none of your business :)
//! Note that you have to set the JavaVM once before you use the functions in this library. Failure to do that will make your program panic!
//! # Example
//! ```
//! javavm::set_jvm(None); // Pass a valid JavaVM instance here (hint: use the jni crate, which this crate already depends on)
//! // ... Other code goes here
//!
//! let handle = std::thread::spawn(|| {
//!     // When you need the JNIEnv
//!     let _ = javavm::get_env();
//! });
//! ```

use std::cell::RefCell;
use std::collections::HashMap;

use jni::objects::JClass;
use jni::JNIEnv;
use jni::JavaVM;

static mut VM: Option<JavaVM> = None;
thread_local! {
    static CACHED_CLASSES: RefCell<HashMap<String, JClass<'static>>> = RefCell::new(HashMap::new());
}

/// Sets the current JavaVM. All JNIEnv instances will come from this JavaVM
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

/// Find and cache the JClass for the current thread. If the class has already been looked up, it returns the class and avoids any other expensive lookup without any other work on your part. The class will be automatically unloaded on program termination. If you want to unload a class before program termination, use `unload_cached_class(&str)`
pub fn load_class_cached(name: &str) -> Option<JClass<'static>> {
    let get_class = || -> Option<JClass> {
        return CACHED_CLASSES.with(|map| -> Option<JClass> {
            if map.borrow().contains_key(name) {
                let res = *map.borrow().get(name).unwrap();
                return Some(res);
            }

            None
        });
    };

    let class = get_class();

    if class.is_some() {
        return class;
    }

    let env = get_env();
    if let Ok(class) = env.find_class(name) {
        CACHED_CLASSES.with(|map| {
            map.borrow_mut().insert(name.to_string(), class);
        });
    }

    get_class()
}

/// Unloads a cached class (if one exists). Does nothing if the class has not already been cached
pub fn unload_cached_class(name: &str) {
    CACHED_CLASSES.with(|map| map.borrow_mut().remove(name));
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
        // Get the env from another thread. This is probably more appropriate for an example
        let handle = std::thread::spawn(|| {
            let _ = get_env();
        });

        handle.join().unwrap();
    }

    #[test]
    fn no_env_safe() {
        assert!(get_env_safe().is_none());
    }
}
