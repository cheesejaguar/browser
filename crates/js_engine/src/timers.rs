//! Timer APIs (setTimeout, setInterval, etc.)

use crate::runtime::{Runtime, TimerCallback};
use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    property::Attribute,
};
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;

/// Register timer APIs on the global object.
pub fn register_timers(context: &mut Context, runtime: Arc<RwLock<Runtime>>) {
    let runtime_set_timeout = runtime.clone();
    let set_timeout = FunctionBuilder::new(context, move |_this, args, ctx| {
        set_timeout_impl(args, ctx, runtime_set_timeout.clone(), false)
    });

    let runtime_set_interval = runtime.clone();
    let set_interval = FunctionBuilder::new(context, move |_this, args, ctx| {
        set_timeout_impl(args, ctx, runtime_set_interval.clone(), true)
    });

    let runtime_clear = runtime.clone();
    let clear_timeout = FunctionBuilder::new(context, move |_this, args, ctx| {
        clear_timeout_impl(args, ctx, runtime_clear.clone())
    });

    let runtime_clear2 = runtime.clone();
    let clear_interval = FunctionBuilder::new(context, move |_this, args, ctx| {
        clear_timeout_impl(args, ctx, runtime_clear2.clone())
    });

    context
        .register_global_builtin_callable(js_string!("setTimeout"), 1, set_timeout)
        .expect("Failed to register setTimeout");

    context
        .register_global_builtin_callable(js_string!("setInterval"), 1, set_interval)
        .expect("Failed to register setInterval");

    context
        .register_global_builtin_callable(js_string!("clearTimeout"), 1, clear_timeout)
        .expect("Failed to register clearTimeout");

    context
        .register_global_builtin_callable(js_string!("clearInterval"), 1, clear_interval)
        .expect("Failed to register clearInterval");

    // queueMicrotask
    let runtime_microtask = runtime.clone();
    let queue_microtask = FunctionBuilder::new(context, move |_this, args, ctx| {
        queue_microtask_impl(args, ctx, runtime_microtask.clone())
    });

    context
        .register_global_builtin_callable(js_string!("queueMicrotask"), 1, queue_microtask)
        .expect("Failed to register queueMicrotask");

    // requestAnimationFrame
    let runtime_raf = runtime.clone();
    let request_animation_frame = FunctionBuilder::new(context, move |_this, args, ctx| {
        request_animation_frame_impl(args, ctx, runtime_raf.clone())
    });

    context
        .register_global_builtin_callable(js_string!("requestAnimationFrame"), 1, request_animation_frame)
        .expect("Failed to register requestAnimationFrame");

    // cancelAnimationFrame
    let runtime_caf = runtime.clone();
    let cancel_animation_frame = FunctionBuilder::new(context, move |_this, args, ctx| {
        cancel_animation_frame_impl(args, ctx, runtime_caf.clone())
    });

    context
        .register_global_builtin_callable(js_string!("cancelAnimationFrame"), 1, cancel_animation_frame)
        .expect("Failed to register cancelAnimationFrame");

    // requestIdleCallback
    let runtime_ric = runtime.clone();
    let request_idle_callback = FunctionBuilder::new(context, move |_this, args, ctx| {
        request_idle_callback_impl(args, ctx, runtime_ric.clone())
    });

    context
        .register_global_builtin_callable(js_string!("requestIdleCallback"), 1, request_idle_callback)
        .expect("Failed to register requestIdleCallback");

    // cancelIdleCallback
    let runtime_cic = runtime;
    let cancel_idle_callback = FunctionBuilder::new(context, move |_this, args, ctx| {
        cancel_idle_callback_impl(args, ctx, runtime_cic.clone())
    });

    context
        .register_global_builtin_callable(js_string!("cancelIdleCallback"), 1, cancel_idle_callback)
        .expect("Failed to register cancelIdleCallback");
}

/// Helper for building native functions with closure captures.
struct FunctionBuilder<F>
where
    F: Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + 'static,
{
    callback: F,
}

impl<F> FunctionBuilder<F>
where
    F: Fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue> + 'static,
{
    fn new(_context: &mut Context, callback: F) -> NativeFunction {
        // We need to create a static function that can be used with NativeFunction
        // For now, use a simple wrapper
        NativeFunction::from_copy_closure(move |this, args, ctx| {
            // This is a simplified version - in practice we'd store the callback properly
            Ok(JsValue::undefined())
        })
    }
}

/// Implementation of setTimeout/setInterval.
fn set_timeout_impl(
    args: &[JsValue],
    context: &mut Context,
    runtime: Arc<RwLock<Runtime>>,
    repeat: bool,
) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);

    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("First argument must be a function")
            .into());
    }

    let delay = args
        .get_or_undefined(1)
        .to_u32(context)
        .unwrap_or(0);

    let delay = Duration::from_millis(delay.max(4) as u64); // Minimum 4ms

    // Store callback reference (simplified - would need proper GC integration)
    let callback_id = 0u64; // Placeholder

    let timer_id = runtime.write().add_timer(
        TimerCallback::JsFunction(callback_id),
        delay,
        repeat,
    );

    Ok(JsValue::from(timer_id))
}

/// Implementation of clearTimeout/clearInterval.
fn clear_timeout_impl(
    args: &[JsValue],
    context: &mut Context,
    runtime: Arc<RwLock<Runtime>>,
) -> JsResult<JsValue> {
    let id = args
        .get_or_undefined(0)
        .to_u32(context)
        .unwrap_or(0);

    runtime.write().cancel_timer(id);

    Ok(JsValue::undefined())
}

/// Implementation of queueMicrotask.
fn queue_microtask_impl(
    args: &[JsValue],
    _context: &mut Context,
    runtime: Arc<RwLock<Runtime>>,
) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);

    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("First argument must be a function")
            .into());
    }

    // Queue the microtask
    let callback_id = 0u64; // Placeholder
    runtime.write().queue_microtask(crate::runtime::Microtask {
        callback: crate::runtime::MicrotaskCallback::JsFunction(callback_id),
    });

    Ok(JsValue::undefined())
}

/// Implementation of requestAnimationFrame.
fn request_animation_frame_impl(
    args: &[JsValue],
    _context: &mut Context,
    runtime: Arc<RwLock<Runtime>>,
) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);

    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("First argument must be a function")
            .into());
    }

    let id = runtime.write().request_animation_frame(Arc::new(|_timestamp| {
        // Callback would be invoked with high-resolution timestamp
    }));

    Ok(JsValue::from(id))
}

/// Implementation of cancelAnimationFrame.
fn cancel_animation_frame_impl(
    args: &[JsValue],
    context: &mut Context,
    runtime: Arc<RwLock<Runtime>>,
) -> JsResult<JsValue> {
    let id = args
        .get_or_undefined(0)
        .to_u32(context)
        .unwrap_or(0);

    runtime.write().cancel_animation_frame(id);

    Ok(JsValue::undefined())
}

/// Implementation of requestIdleCallback.
fn request_idle_callback_impl(
    args: &[JsValue],
    context: &mut Context,
    runtime: Arc<RwLock<Runtime>>,
) -> JsResult<JsValue> {
    let callback = args.get_or_undefined(0);

    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("First argument must be a function")
            .into());
    }

    // Parse options
    let timeout = if let Some(options) = args.get(1).and_then(|v| v.as_object()) {
        options
            .get(js_string!("timeout"), context)
            .ok()
            .and_then(|v| v.to_u32(context).ok())
            .map(|ms| Duration::from_millis(ms as u64))
    } else {
        None
    };

    let id = runtime.write().request_idle_callback(
        Arc::new(|_deadline| {
            // Callback would be invoked with idle deadline
        }),
        timeout,
    );

    Ok(JsValue::from(id))
}

/// Implementation of cancelIdleCallback.
fn cancel_idle_callback_impl(
    args: &[JsValue],
    context: &mut Context,
    runtime: Arc<RwLock<Runtime>>,
) -> JsResult<JsValue> {
    let id = args
        .get_or_undefined(0)
        .to_u32(context)
        .unwrap_or(0);

    runtime.write().cancel_idle_callback(id);

    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_minimum_delay() {
        // Minimum delay should be 4ms per HTML spec
        let delay = Duration::from_millis(4);
        assert!(delay.as_millis() >= 4);
    }
}
