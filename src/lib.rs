extern crate napi;
#[macro_use]
extern crate napi_derive;
use napi::{CallContext, Env, JsObject, JsString, JsStringUtf8, JsUndefined, Property, Result};

mod watcher;
use watcher::Watcher;

#[js_function(2)]
fn watch(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let watcher_instance: &mut Watcher = ctx.env.unwrap(&this)?;
    let watch_options = ctx.get::<JsObject>(0)?;
    let watch_directory: JsStringUtf8 = watch_options
        .get_named_property::<JsString>("directory")?
        .into_utf8()?;

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
