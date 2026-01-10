//! Console API implementation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, JsValue, NativeFunction,
    js_string,
    object::ObjectInitializer,
    property::Attribute,
};
use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Console log level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

impl LogLevel {
    fn prefix(&self) -> &'static str {
        match self {
            LogLevel::Log => "",
            LogLevel::Info => "[INFO] ",
            LogLevel::Warn => "[WARN] ",
            LogLevel::Error => "[ERROR] ",
            LogLevel::Debug => "[DEBUG] ",
            LogLevel::Trace => "[TRACE] ",
        }
    }
}

/// Console state for timers and counters.
static TIMER_START: AtomicU64 = AtomicU64::new(0);

/// Register the console API on the global object.
pub fn register_console(context: &mut Context) {
    let console = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(console_log), js_string!("log"), 0)
        .function(NativeFunction::from_fn_ptr(console_info), js_string!("info"), 0)
        .function(NativeFunction::from_fn_ptr(console_warn), js_string!("warn"), 0)
        .function(NativeFunction::from_fn_ptr(console_error), js_string!("error"), 0)
        .function(NativeFunction::from_fn_ptr(console_debug), js_string!("debug"), 0)
        .function(NativeFunction::from_fn_ptr(console_trace), js_string!("trace"), 0)
        .function(NativeFunction::from_fn_ptr(console_assert), js_string!("assert"), 0)
        .function(NativeFunction::from_fn_ptr(console_clear), js_string!("clear"), 0)
        .function(NativeFunction::from_fn_ptr(console_count), js_string!("count"), 0)
        .function(NativeFunction::from_fn_ptr(console_count_reset), js_string!("countReset"), 0)
        .function(NativeFunction::from_fn_ptr(console_group), js_string!("group"), 0)
        .function(NativeFunction::from_fn_ptr(console_group_collapsed), js_string!("groupCollapsed"), 0)
        .function(NativeFunction::from_fn_ptr(console_group_end), js_string!("groupEnd"), 0)
        .function(NativeFunction::from_fn_ptr(console_time), js_string!("time"), 0)
        .function(NativeFunction::from_fn_ptr(console_time_log), js_string!("timeLog"), 0)
        .function(NativeFunction::from_fn_ptr(console_time_end), js_string!("timeEnd"), 0)
        .function(NativeFunction::from_fn_ptr(console_table), js_string!("table"), 0)
        .function(NativeFunction::from_fn_ptr(console_dir), js_string!("dir"), 0)
        .function(NativeFunction::from_fn_ptr(console_dirxml), js_string!("dirxml"), 0)
        .build();

    context
        .register_global_property(js_string!("console"), console, Attribute::all())
        .expect("Failed to register console");
}

/// Format arguments for console output.
fn format_args(args: &[JsValue], context: &mut Context) -> String {
    let mut output = String::new();

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            output.push(' ');
        }
        let _ = write!(output, "{}", format_value(arg, context, 0));
    }

    output
}

/// Format a single value.
fn format_value(value: &JsValue, context: &mut Context, depth: usize) -> String {
    if depth > 3 {
        return "[...]".to_string();
    }

    match value {
        JsValue::Undefined => "undefined".to_string(),
        JsValue::Null => "null".to_string(),
        JsValue::Boolean(b) => b.to_string(),
        JsValue::Integer(i) => i.to_string(),
        JsValue::Rational(r) => {
            if r.is_nan() {
                "NaN".to_string()
            } else if r.is_infinite() {
                if *r > 0.0 {
                    "Infinity".to_string()
                } else {
                    "-Infinity".to_string()
                }
            } else {
                r.to_string()
            }
        }
        JsValue::String(s) => s.to_std_string_escaped(),
        JsValue::Symbol(s) => format!("Symbol({})", s.description().map(|d| d.to_std_string_escaped()).unwrap_or_default()),
        JsValue::BigInt(b) => format!("{}n", b.to_string()),
        JsValue::Object(obj) => {
            // Check if it's an array
            if obj.is_array() {
                let length = obj
                    .get(js_string!("length"), context)
                    .ok()
                    .and_then(|v| v.to_length(context).ok())
                    .unwrap_or(0);

                let mut items = Vec::new();
                for i in 0..length.min(10) {
                    if let Ok(item) = obj.get(i, context) {
                        items.push(format_value(&item, context, depth + 1));
                    }
                }

                if length > 10 {
                    items.push(format!("... {} more items", length - 10));
                }

                format!("[{}]", items.join(", "))
            } else if obj.is_callable() {
                // Function
                let name = obj
                    .get(js_string!("name"), context)
                    .ok()
                    .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
                    .unwrap_or_default();

                if name.is_empty() {
                    "[Function (anonymous)]".to_string()
                } else {
                    format!("[Function: {}]", name)
                }
            } else {
                // Regular object
                let keys = obj.own_property_keys(context).unwrap_or_default();
                let mut pairs = Vec::new();

                for (i, key) in keys.iter().enumerate() {
                    if i >= 5 {
                        pairs.push(format!("... {} more", keys.len() - 5));
                        break;
                    }

                    if let Ok(value) = obj.get(key.clone(), context) {
                        let key_str = key.to_string();
                        pairs.push(format!("{}: {}", key_str, format_value(&value, context, depth + 1)));
                    }
                }

                format!("{{ {} }}", pairs.join(", "))
            }
        }
    }
}

/// console.log()
fn console_log(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = format_args(args, context);
    println!("{}", message);
    Ok(JsValue::undefined())
}

/// console.info()
fn console_info(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = format_args(args, context);
    println!("{}{}", LogLevel::Info.prefix(), message);
    Ok(JsValue::undefined())
}

/// console.warn()
fn console_warn(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = format_args(args, context);
    eprintln!("{}{}", LogLevel::Warn.prefix(), message);
    Ok(JsValue::undefined())
}

/// console.error()
fn console_error(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = format_args(args, context);
    eprintln!("{}{}", LogLevel::Error.prefix(), message);
    Ok(JsValue::undefined())
}

/// console.debug()
fn console_debug(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = format_args(args, context);
    println!("{}{}", LogLevel::Debug.prefix(), message);
    Ok(JsValue::undefined())
}

/// console.trace()
fn console_trace(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let message = format_args(args, context);
    println!("{}{}", LogLevel::Trace.prefix(), message);
    println!("  (stack trace not available)");
    Ok(JsValue::undefined())
}

/// console.assert()
fn console_assert(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let condition = args.get_or_undefined(0).to_boolean();

    if !condition {
        let message = if args.len() > 1 {
            format_args(&args[1..], context)
        } else {
            "Assertion failed".to_string()
        };
        eprintln!("Assertion failed: {}", message);
    }

    Ok(JsValue::undefined())
}

/// console.clear()
fn console_clear(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    // Clear terminal (ANSI escape code)
    print!("\x1B[2J\x1B[H");
    Ok(JsValue::undefined())
}

/// console.count()
fn console_count(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let label = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let label = if label == "undefined" { "default" } else { &label };

    // In a real implementation, we'd track counts per label
    println!("{}: 1", label);
    Ok(JsValue::undefined())
}

/// console.countReset()
fn console_count_reset(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let label = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let label = if label == "undefined" { "default" } else { &label };

    println!("{}: 0", label);
    Ok(JsValue::undefined())
}

/// console.group()
fn console_group(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let label = format_args(args, context);
    println!("▼ {}", if label.is_empty() { "console.group" } else { &label });
    Ok(JsValue::undefined())
}

/// console.groupCollapsed()
fn console_group_collapsed(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let label = format_args(args, context);
    println!("▶ {}", if label.is_empty() { "console.group" } else { &label });
    Ok(JsValue::undefined())
}

/// console.groupEnd()
fn console_group_end(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    // In a real implementation, we'd track group nesting
    Ok(JsValue::undefined())
}

/// console.time()
fn console_time(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _label = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();

    // Store start time (simplified - would use a HashMap in real impl)
    let now = Instant::now();
    let nanos = now.elapsed().as_nanos() as u64;
    TIMER_START.store(nanos, Ordering::SeqCst);

    Ok(JsValue::undefined())
}

/// console.timeLog()
fn console_time_log(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let label = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let label = if label == "undefined" { "default" } else { &label };

    // Calculate elapsed time (simplified)
    println!("{}: 0ms", label);

    if args.len() > 1 {
        let extra = format_args(&args[1..], context);
        println!("  {}", extra);
    }

    Ok(JsValue::undefined())
}

/// console.timeEnd()
fn console_time_end(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let label = args
        .get_or_undefined(0)
        .to_string(context)?
        .to_std_string_escaped();
    let label = if label == "undefined" { "default" } else { &label };

    println!("{}: 0ms", label);
    Ok(JsValue::undefined())
}

/// console.table()
fn console_table(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let data = args.get_or_undefined(0);

    if let Some(obj) = data.as_object() {
        if obj.is_array() {
            println!("┌─────────┬────────────────────┐");
            println!("│ (index) │       Values       │");
            println!("├─────────┼────────────────────┤");

            let length = obj
                .get(js_string!("length"), context)
                .ok()
                .and_then(|v| v.to_length(context).ok())
                .unwrap_or(0);

            for i in 0..length.min(20) {
                if let Ok(item) = obj.get(i, context) {
                    let value = format_value(&item, context, 0);
                    println!("│ {:>7} │ {:>18} │", i, &value[..value.len().min(18)]);
                }
            }

            println!("└─────────┴────────────────────┘");
        } else {
            // Object
            println!("{}", format_value(data, context, 0));
        }
    } else {
        println!("{}", format_value(data, context, 0));
    }

    Ok(JsValue::undefined())
}

/// console.dir()
fn console_dir(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = args.get_or_undefined(0);
    println!("{}", format_value(obj, context, 0));
    Ok(JsValue::undefined())
}

/// console.dirxml()
fn console_dirxml(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // In a browser, this would show XML representation of DOM nodes
    // For now, just format as regular object
    let obj = args.get_or_undefined(0);
    println!("{}", format_value(obj, context, 0));
    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_primitive() {
        let mut context = Context::default();

        assert_eq!(format_value(&JsValue::undefined(), &mut context, 0), "undefined");
        assert_eq!(format_value(&JsValue::Null, &mut context, 0), "null");
        assert_eq!(format_value(&JsValue::from(true), &mut context, 0), "true");
        assert_eq!(format_value(&JsValue::from(42), &mut context, 0), "42");
    }
}
