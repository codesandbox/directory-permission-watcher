extern crate napi;
#[macro_use]
extern crate napi_derive;
use napi::{CallContext, Env, JsObject, JsUndefined, Property, Result};

mod watcher;
use watcher::Watcher;

#[js_function(2)]
fn watch(ctx: CallContext) -> Result<JsUndefined> {
    // let this: JsObject = ctx.this_unchecked();
    // let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;
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
