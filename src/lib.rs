extern crate napi;
#[macro_use]
extern crate napi_derive;

mod watcher;
use watcher::Watcher;

use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    CallContext, Env, JsFunction, JsObject, JsString, JsStringUtf8, JsUndefined, Property, Result,
};
use std::thread;

#[js_function(2)]
fn watch(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;
    let watch_options = ctx.get::<JsObject>(0)?;
    let watch_directory: JsStringUtf8 = watch_options
        .get_named_property::<JsString>("directory")?
        .into_utf8()?;

    let func = ctx.get::<JsFunction>(1)?;
    let tsfn: ThreadsafeFunction<Vec<String>> =
        ctx.env
            .create_threadsafe_function(&func, 0, |ctx: ThreadSafeCallContext<Vec<String>>| {
                ctx.value
                    .iter()
                    .map(|v| ctx.env.create_string(v.as_str()))
                    .collect::<Result<Vec<JsString>>>()
            })?;

    thread::spawn(move || {
        // It's okay to call a threadsafe function multiple times.
        tsfn.call(Ok(vec![String::from("test")]), ThreadsafeFunctionCallMode::Blocking);
    });

    watcher_instance.watch(watch_directory.as_str()?);

    ctx.env.get_undefined()
}

#[js_function()]
fn constructor(ctx: CallContext) -> Result<JsUndefined> {
    let mut this: JsObject = ctx.this_unchecked();
    ctx.env.wrap(&mut this, Watcher::new())?;
    ctx.env.get_undefined()
}

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
    let watch_method = Property::new(&env, "watch")?.with_method(watch);
    let watcher_class = env.define_class("Watcher", constructor, &[watch_method])?;
    exports.set_named_property("Watcher", watcher_class)?;
    Ok(())
}
