extern crate napi;
#[macro_use]
extern crate napi_derive;

mod watcher;
use watcher::Watcher;

use futures::StreamExt;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    CallContext, Env, JsFunction, JsObject, JsString, JsUndefined, Property, Result,
};
use std::thread;

#[js_function(1)]
fn watch(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;
    let watch_path = ctx.get::<JsString>(0)?.into_utf8()?;

    watcher_instance.watch(watch_path.as_str()?);

    ctx.env.get_undefined()
}

#[js_function(1)]
fn unwatch(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;
    let watch_path = ctx.get::<JsString>(0)?.into_utf8()?;

    // TODO: Add an unwatch
    // watcher_instance.watch(watch_path.as_str()?);

    ctx.env.get_undefined()
}

#[js_function(1)]
fn listen(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;

    let func = ctx.get::<JsFunction>(0)?;
    let tsfn: ThreadsafeFunction<Vec<String>> = ctx.env.create_threadsafe_function(
        &func,
        0,
        |ctx: ThreadSafeCallContext<Vec<String>>| {
            ctx.value
                .iter()
                .map(|v| ctx.env.create_string(v.as_str()))
                .collect::<Result<Vec<JsString>>>()
        },
    )?;

    thread::spawn(move || {
        futures::executor::block_on(async {
            while let Some(res) = watcher_instance.notify_rx.next().await {
                tsfn.call(Ok(vec![res]), ThreadsafeFunctionCallMode::Blocking);
            }
        })
    });

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
    let unwatch_method = Property::new(&env, "unwatch")?.with_method(watch);
    let watcher_class = env.define_class("Watcher", constructor, &[watch_method])?;
    exports.set_named_property("Watcher", watcher_class)?;
    Ok(())
}
