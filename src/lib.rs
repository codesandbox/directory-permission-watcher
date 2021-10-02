extern crate napi;
#[macro_use]
extern crate napi_derive;

mod watcher;
use watcher::Watcher;

mod error;

use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction},
    CallContext, Env, JsFunction, JsObject, JsString, JsUndefined, Property, Result,
};

#[js_function(1)]
fn watch(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;
    let watch_path = ctx.get::<JsString>(0)?.into_utf8()?;

    watcher_instance.watch(watch_path.as_str()?)?;

    ctx.env.get_undefined()
}

// TODO: Implement unwatch
#[js_function(1)]
fn unwatch(ctx: CallContext) -> Result<JsUndefined> {
    // let this: JsObject = ctx.this_unchecked();
    // let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;
    // let watch_path = ctx.get::<JsString>(0)?.into_utf8()?;
    // watcher_instance.watch(watch_path.as_str()?);
    ctx.env.get_undefined()
}

#[js_function(2)]
fn constructor(ctx: CallContext) -> Result<JsUndefined> {
    let mut this: JsObject = ctx.this_unchecked();
    let func = ctx.get::<JsFunction>(1)?;
    let tsfn: ThreadsafeFunction<String> =
        ctx.env
            .create_threadsafe_function(&func, 0, |ctx: ThreadSafeCallContext<String>| {
                ctx.env
                    .create_string_from_std(ctx.value)
                    .map(|js_string| vec![js_string])
            })?;
    ctx.env.wrap(&mut this, Watcher::new(tsfn)?)?;
    ctx.env.get_undefined()
}

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
    let watch_method = Property::new(&env, "watch")?.with_method(watch);
    let watcher_class = env.define_class("Watcher", constructor, &[watch_method])?;
    exports.set_named_property("Watcher", watcher_class)?;
    Ok(())
}
